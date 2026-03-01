use proc_macro::TokenStream;

/// Holds the computed `Cache-Control` header value for a query handler.
#[derive(Debug)]
pub(crate) struct CacheConfig {
    pub(crate) cache_control: String,
}

/// Parsed attributes from `#[rpc_query(...)]` or `#[rpc_mutation(...)]`.
#[derive(Debug, Default)]
pub(crate) struct HandlerAttrs {
    pub(crate) cache_config: Option<CacheConfig>,
    pub(crate) init_fn: Option<String>,
    pub(crate) timeout_secs: Option<u64>,
    pub(crate) idempotent: bool,
}

/// Parses handler attributes from `#[rpc_query(...)]` or `#[rpc_mutation(...)]`.
///
/// Supports key-value pairs (`cache`, `stale`, `init`, `timeout`) and bare flags
/// (`idempotent`). Returns `HandlerAttrs` with all fields at their defaults when
/// the attribute is empty (backward compatible bare `#[rpc_query]`).
pub(crate) fn parse_handler_attrs(attr: TokenStream) -> Result<HandlerAttrs, syn::Error> {
    parse_handler_attrs_inner(attr.into())
}

/// Inner implementation that accepts `proc_macro2::TokenStream` for testability.
pub(crate) fn parse_handler_attrs_inner(
    attr: proc_macro2::TokenStream,
) -> Result<HandlerAttrs, syn::Error> {
    if attr.is_empty() {
        return Ok(HandlerAttrs::default());
    }

    let parsed = syn::parse::Parser::parse2(
        syn::punctuated::Punctuated::<syn::Meta, syn::token::Comma>::parse_terminated,
        attr,
    )?;

    let mut cache_value = None;
    let mut stale_value = None;
    let mut init_value = None;
    let mut timeout_value = None;
    let mut idempotent = false;

    for meta in &parsed {
        match meta {
            syn::Meta::Path(path) => {
                let ident = path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(path, "expected a simple identifier"))?;

                if ident == "idempotent" {
                    if idempotent {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "duplicate `idempotent` attribute",
                        ));
                    }
                    idempotent = true;
                } else {
                    return Err(syn::Error::new_spanned(
                        ident,
                        format!("unknown attribute `{ident}`"),
                    ));
                }
            }
            syn::Meta::NameValue(nv) => {
                let key = nv.path.get_ident().ok_or_else(|| {
                    syn::Error::new_spanned(&nv.path, "expected a simple identifier")
                })?;

                // Reject `idempotent = ...` before parsing the value — it's a bare flag.
                if key == "idempotent" {
                    return Err(syn::Error::new_spanned(
                        key,
                        "`idempotent` is a bare flag and does not accept a value; use `idempotent` instead of `idempotent = \"...\"`",
                    ));
                }

                let value = match &nv.value {
                    syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                        syn::Lit::Str(s) => s.value(),
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "expected a string literal",
                            ));
                        }
                    },
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "expected a string literal",
                        ));
                    }
                };

                if key == "cache" {
                    if cache_value.is_some() {
                        return Err(syn::Error::new_spanned(key, "duplicate `cache` attribute"));
                    }
                    cache_value = Some(value);
                } else if key == "stale" {
                    if stale_value.is_some() {
                        return Err(syn::Error::new_spanned(key, "duplicate `stale` attribute"));
                    }
                    stale_value = Some(value);
                } else if key == "init" {
                    if init_value.is_some() {
                        return Err(syn::Error::new_spanned(key, "duplicate `init` attribute"));
                    }
                    if value.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "init function path cannot be empty",
                        ));
                    }
                    init_value = Some(value);
                } else if key == "timeout" {
                    if timeout_value.is_some() {
                        return Err(syn::Error::new_spanned(
                            key,
                            "duplicate `timeout` attribute",
                        ));
                    }
                    if value.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "timeout duration cannot be empty",
                        ));
                    }
                    timeout_value = Some(value);
                } else {
                    return Err(syn::Error::new_spanned(
                        key,
                        format!("unknown attribute `{key}`"),
                    ));
                }
            }
            syn::Meta::List(list) => {
                return Err(syn::Error::new_spanned(
                    list,
                    "expected `key = \"value\"` or bare flag",
                ));
            }
        }
    }

    let cache_config = if cache_value.is_some() || stale_value.is_some() {
        let cache_value = cache_value.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required `cache` attribute",
            )
        })?;

        let cache_control = build_cache_control(&cache_value, stale_value.as_deref())
            .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?;

        Some(CacheConfig { cache_control })
    } else {
        None
    };

    let timeout_secs = timeout_value
        .map(|v| parse_duration(&v))
        .transpose()
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e))?;

    Ok(HandlerAttrs {
        cache_config,
        init_fn: init_value,
        timeout_secs,
        idempotent,
    })
}

/// Parses human-readable duration shorthand into seconds.
///
/// Supported suffixes: `s` (seconds), `m` (minutes), `h` (hours), `d` (days).
/// Zero durations are rejected.
pub(crate) fn parse_duration(s: &str) -> Result<u64, String> {
    if s.is_empty() {
        return Err("duration cannot be empty".into());
    }

    let (num_str, multiplier) = if let Some(n) = s.strip_suffix('s') {
        (n, 1)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3600)
    } else if let Some(n) = s.strip_suffix('d') {
        (n, 86400)
    } else {
        return Err(format!(
            "invalid duration suffix in `{s}`, expected s/m/h/d"
        ));
    };

    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number in duration `{s}`"))?;

    if num == 0 {
        return Err(format!("duration cannot be zero: `{s}`"));
    }

    Ok(num * multiplier)
}

/// Builds the `Cache-Control` header value from parsed `cache` and optional `stale` values.
///
/// - `"1h"` → `"public, max-age=0, s-maxage=3600"`
/// - `"private, 10m"` → `"private, max-age=600"`
/// - `"5m"` + stale `"1h"` → `"public, max-age=0, s-maxage=300, stale-while-revalidate=3600"`
pub(crate) fn build_cache_control(
    cache_value: &str,
    stale_value: Option<&str>,
) -> Result<String, String> {
    let (is_private, duration_str) = if let Some(rest) = cache_value.strip_prefix("private,") {
        (true, rest.trim())
    } else {
        (false, cache_value.trim())
    };

    let seconds = parse_duration(duration_str)?;
    let stale_seconds = stale_value.map(parse_duration).transpose()?;

    if is_private {
        let mut header = format!("private, max-age={seconds}");
        if let Some(stale) = stale_seconds {
            header.push_str(&format!(", stale-while-revalidate={stale}"));
        }
        Ok(header)
    } else {
        let mut header = format!("public, max-age=0, s-maxage={seconds}");
        if let Some(stale) = stale_seconds {
            header.push_str(&format!(", stale-while-revalidate={stale}"));
        }
        Ok(header)
    }
}
