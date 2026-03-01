use quote::quote;
use syn::{FnArg, ItemFn, ReturnType, Type};

use crate::attrs::HandlerAttrs;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandlerKind {
    Query,
    Mutation,
}

/// Transforms a user-defined async function into a complete Vercel lambda handler.
///
/// Generates `main()`, CORS helpers, input parsing, and response serialization.
/// The `kind` parameter determines whether the handler accepts GET (query) or POST (mutation).
#[expect(
    clippy::needless_pass_by_value,
    reason = "ItemFn is owned from parse_macro_input"
)]
pub(crate) fn build_handler(
    func: ItemFn,
    kind: HandlerKind,
    attrs: HandlerAttrs,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let HandlerAttrs {
        cache_config,
        init_fn,
        timeout_secs,
        idempotent: _,
    } = attrs;

    if func.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            func.sig.fn_token,
            "RPC handlers must be async functions",
        ));
    }

    let fn_name = &func.sig.ident;
    let fn_block = &func.block;
    let fn_output = &func.sig.output;

    // Separate typed parameters into input, headers, and state params.
    let typed_params: Vec<_> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat) => Some(pat),
            _ => None,
        })
        .collect();

    let mut input_param = None;
    let mut headers_param = None;
    let mut state_param = None;

    for param in &typed_params {
        if is_headers_type(&param.ty) {
            if headers_param.is_some() {
                return Err(syn::Error::new_spanned(
                    &func.sig.inputs,
                    "RPC handlers accept at most one Headers parameter",
                ));
            }
            headers_param = Some(*param);
        } else if let Type::Reference(r) = &*param.ty {
            if r.mutability.is_some() {
                return Err(syn::Error::new_spanned(
                    &param.ty,
                    "state parameter must be a shared reference (&T), not &mut T",
                ));
            }
            if state_param.is_some() {
                return Err(syn::Error::new_spanned(
                    &func.sig.inputs,
                    "RPC handlers accept at most one state parameter",
                ));
            }
            state_param = Some(*param);
        } else if input_param.is_some() {
            return Err(syn::Error::new_spanned(
                &func.sig.inputs,
                "RPC handlers accept at most one input parameter",
            ));
        } else {
            input_param = Some(*param);
        }
    }

    if state_param.is_some() && init_fn.is_none() {
        return Err(syn::Error::new_spanned(
            &func.sig.inputs,
            "state parameter requires init = \"...\" attribute",
        ));
    }

    let input_type = if let Some(param) = input_param {
        let ty = &param.ty;
        quote! { #ty }
    } else {
        quote! { () }
    };

    // Determine whether the handler returns Result<T, E> or plain T
    let (return_type, returns_result) = match fn_output {
        ReturnType::Default => (quote! { () }, false),
        ReturnType::Type(_, ty) => (quote! { #ty }, is_result_type(ty)),
    };

    let expected_method = match kind {
        HandlerKind::Query => "GET",
        HandlerKind::Mutation => "POST",
    };

    let parse_input = match kind {
        HandlerKind::Query => quote! {
            let __input: #input_type = {
                let __url = match ::metaxy::__private::url::Url::parse(
                    &format!("http://localhost{}", __req.uri())
                ) {
                    Ok(u) => u,
                    Err(e) => return __rpc_error_response(400, &format!("Invalid URL: {}", e)),
                };

                let __raw = __url
                    .query_pairs()
                    .find(|(k, _)| k == "input")
                    .map(|(_, v)| v.into_owned());

                match __raw {
                    Some(ref __s) => match ::metaxy::__private::serde_json::from_str(__s) {
                        Ok(v) => v,
                        Err(e) => return __rpc_error_response(400,
                            &format!("Failed to deserialize input: {}", e)),
                    },
                    None => match ::metaxy::__private::serde_json::from_value(::metaxy::__private::serde_json::Value::Null) {
                        Ok(v) => v,
                        Err(e) => return __rpc_error_response(400,
                            &format!("Missing required input parameter: {}", e)),
                    },
                }
            };
        },
        HandlerKind::Mutation => quote! {
            let __input: #input_type = {
                use ::metaxy::__private::http_body_util::BodyExt as _;
                let __collected = __req.into_body().collect().await
                    .map_err(|e| ::metaxy::__private::vercel_runtime::Error::from(
                        format!("Failed to read request body: {}", e)
                    ))?;
                let __bytes = __collected.to_bytes();

                if __bytes.is_empty() {
                    match ::metaxy::__private::serde_json::from_value(::metaxy::__private::serde_json::Value::Null) {
                        Ok(v) => v,
                        Err(e) => return __rpc_error_response(400,
                            &format!("Missing required request body: {}", e)),
                    }
                } else {
                    match ::metaxy::__private::serde_json::from_slice(&__bytes) {
                        Ok(v) => v,
                        Err(e) => return __rpc_error_response(400,
                            &format!("Failed to deserialize request body: {}", e)),
                    }
                }
            };
        },
    };

    // Build the result handling block depending on whether the fn returns Result<T, E>
    let result_handling = if returns_result {
        quote! {
            match __raw_result {
                Ok(__val) => {
                    let __data = ::metaxy::__private::serde_json::to_value(&__val)
                        .map_err(|e| ::metaxy::__private::vercel_runtime::Error::from(
                            format!("Failed to serialize response: {}", e)
                        ))?;
                    __rpc_ok_response(__data)
                }
                Err(__err) => {
                    __rpc_error_response(400, &format!("{}", __err))
                }
            }
        }
    } else {
        quote! {
            let __data = ::metaxy::__private::serde_json::to_value(&__raw_result)
                .map_err(|e| ::metaxy::__private::vercel_runtime::Error::from(
                    format!("Failed to serialize response: {}", e)
                ))?;
            __rpc_ok_response(__data)
        }
    };

    // Generate headers extraction and function parameters based on Headers presence.
    let extract_headers = if headers_param.is_some() {
        quote! { let __headers = __req.headers().clone(); }
    } else {
        quote! {}
    };

    // Generate state extraction when state param is present.
    let extract_state = if state_param.is_some() {
        quote! { let __state = __RPC_STATE.get().expect("BUG: init not called"); }
    } else {
        quote! {}
    };

    // Build the inner function parameters and call arguments preserving original order.
    let inner_fn_params: Vec<_> = typed_params
        .iter()
        .map(|param| {
            let pat = &param.pat;
            let ty = &param.ty;
            quote! { #pat: #ty }
        })
        .collect();

    let call_args: Vec<_> = typed_params
        .iter()
        .map(|param| {
            if is_headers_type(&param.ty) {
                quote! { __headers }
            } else if is_ref_type(&param.ty) {
                quote! { __state }
            } else {
                quote! { __input }
            }
        })
        .collect();

    let cache_header = match &cache_config {
        Some(config) => {
            let value = &config.cache_control;
            quote! { .header("Cache-Control", #value) }
        }
        None => quote! {},
    };

    // Generate OnceLock static when init returns state.
    let state_static = if let Some(sp) = state_param {
        let Type::Reference(r) = &*sp.ty else {
            unreachable!("state_param is always a reference");
        };
        let inner_ty = &r.elem;
        quote! {
            static __RPC_STATE: std::sync::OnceLock<#inner_ty> = std::sync::OnceLock::new();
        }
    } else {
        quote! {}
    };

    // Generate init call inside block_on.
    let init_call = if let Some(ref path) = init_fn {
        let path_ident: proc_macro2::TokenStream = path.parse().map_err(|_| {
            syn::Error::new_spanned(&func.sig, format!("invalid init function path: `{path}`"))
        })?;
        if state_param.is_some() {
            quote! {
                __RPC_STATE.set(#path_ident().await).expect("BUG: OnceLock already set");
            }
        } else {
            quote! {
                #path_ident().await;
            }
        }
    } else {
        quote! {}
    };

    let invoke_user_fn = if let Some(secs) = timeout_secs {
        quote! {
            match ::metaxy::__private::tokio::time::timeout(
                ::std::time::Duration::from_secs(#secs),
                #fn_name(#(#call_args),*),
            ).await {
                Ok(result) => result,
                Err(_) => return __rpc_error_response(504, "Handler timed out"),
            }
        }
    } else {
        quote! { #fn_name(#(#call_args),*).await }
    };

    let expanded = quote! {
        #state_static

        fn main() -> Result<(), ::metaxy::__private::vercel_runtime::Error> {
            ::metaxy::__private::tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| ::metaxy::__private::vercel_runtime::Error::from(
                    format!("Failed to build tokio runtime: {}", e)
                ))?
                .block_on(async {
                    #init_call
                    ::metaxy::__private::vercel_runtime::run(
                        ::metaxy::__private::vercel_runtime::service_fn(__rpc_handler),
                    ).await
                })
        }

        // Shared CORS headers applied to every response.
        fn __rpc_cors_headers() -> [(&'static str, &'static str); 4] {
            [
                ("Access-Control-Allow-Origin", "*"),
                ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
                ("Access-Control-Allow-Headers", "Content-Type, Authorization"),
                ("Access-Control-Max-Age", "86400"),
            ]
        }

        // Builds a successful JSON response with CORS headers.
        fn __rpc_ok_response(
            data: ::metaxy::__private::serde_json::Value,
        ) -> Result<::metaxy::__private::vercel_runtime::Response<::metaxy::__private::serde_json::Value>, ::metaxy::__private::vercel_runtime::Error> {
            let mut builder = ::metaxy::__private::vercel_runtime::Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                #cache_header;

            for (k, v) in __rpc_cors_headers() {
                builder = builder.header(k, v);
            }

            Ok(builder.body(::metaxy::__private::serde_json::json!({
                "result": {
                    "type": "response",
                    "data": data
                }
            }))?)
        }

        // Builds a JSON error response with CORS headers.
        fn __rpc_error_response(
            status: u16,
            message: &str,
        ) -> Result<::metaxy::__private::vercel_runtime::Response<::metaxy::__private::serde_json::Value>, ::metaxy::__private::vercel_runtime::Error> {
            let mut builder = ::metaxy::__private::vercel_runtime::Response::builder()
                .status(status)
                .header("Content-Type", "application/json");

            for (k, v) in __rpc_cors_headers() {
                builder = builder.header(k, v);
            }

            Ok(builder.body(::metaxy::__private::serde_json::json!({
                "error": {
                    "type": "error",
                    "message": message
                }
            }))?)
        }

        async fn __rpc_handler(
            __req: ::metaxy::__private::vercel_runtime::Request,
        ) -> Result<::metaxy::__private::vercel_runtime::Response<::metaxy::__private::serde_json::Value>, ::metaxy::__private::vercel_runtime::Error> {
            // Handle CORS preflight
            if __req.method() == "OPTIONS" {
                let mut builder = ::metaxy::__private::vercel_runtime::Response::builder()
                    .status(204);
                for (k, v) in __rpc_cors_headers() {
                    builder = builder.header(k, v);
                }
                return Ok(builder.body(::metaxy::__private::serde_json::Value::Null)?);
            }

            // Validate HTTP method
            if __req.method() != #expected_method {
                return __rpc_error_response(
                    405,
                    &format!(
                        "Method {} not allowed, expected {}",
                        __req.method(),
                        #expected_method,
                    ),
                );
            }

            #extract_headers

            #extract_state

            #parse_input

            async fn #fn_name(#(#inner_fn_params),*) -> #return_type
            #fn_block

            let __raw_result = #invoke_user_fn;

            #result_handling
        }
    };

    Ok(expanded)
}

/// Returns `true` if the type syntactically ends with `Headers`
/// (e.g. `Headers`, `metaxy::Headers`).
///
/// **Limitation:** this is a purely syntactic check, similar to
/// [`is_result_type`]. Type aliases will not be detected, and custom
/// types named `Headers` will be falsely identified.
pub(crate) fn is_headers_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Headers";
    }
    false
}

/// Returns `true` if the type is a shared (immutable) reference `&T`.
pub(crate) fn is_ref_type(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r) if r.mutability.is_none())
}

/// Returns `true` if the type syntactically ends with `Result`
/// (e.g. `Result<T, E>`, `std::result::Result<T, E>`).
///
/// **Limitation:** this is a purely syntactic check. Type aliases (e.g.
/// `type MyResult<T> = Result<T, MyError>`) will not be detected, and custom
/// types named `Result` will be falsely identified.
pub(crate) fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Result";
    }
    false
}
