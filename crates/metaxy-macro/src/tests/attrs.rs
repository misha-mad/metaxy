use quote::quote;

use super::helpers::parse_type;
use crate::attrs::{build_cache_control, parse_duration, parse_handler_attrs_inner};
use crate::codegen::{is_ref_type, is_result_type};

// --- is_result_type ---

#[test]
fn result_type_detected() {
    assert!(is_result_type(&parse_type("Result<String, Error>")));
}

#[test]
fn plain_type_not_result() {
    assert!(!is_result_type(&parse_type("String")));
}

#[test]
fn vec_type_not_result() {
    assert!(!is_result_type(&parse_type("Vec<u8>")));
}

#[test]
fn unit_type_not_result() {
    assert!(!is_result_type(&parse_type("()")));
}

// --- is_ref_type ---

#[test]
fn shared_ref_detected() {
    assert!(is_ref_type(&parse_type("&AppState")));
}

#[test]
fn mut_ref_not_shared() {
    assert!(!is_ref_type(&parse_type("&mut AppState")));
}

#[test]
fn owned_type_not_ref() {
    assert!(!is_ref_type(&parse_type("String")));
}

// --- parse_duration ---

#[test]
fn duration_seconds() {
    assert_eq!(parse_duration("30s").unwrap(), 30);
}

#[test]
fn duration_minutes() {
    assert_eq!(parse_duration("5m").unwrap(), 300);
}

#[test]
fn duration_hours() {
    assert_eq!(parse_duration("1h").unwrap(), 3600);
}

#[test]
fn duration_days() {
    assert_eq!(parse_duration("1d").unwrap(), 86400);
}

#[test]
fn duration_rejects_zero() {
    assert!(parse_duration("0s").unwrap_err().contains("zero"));
}

#[test]
fn duration_rejects_invalid_suffix() {
    assert!(parse_duration("10x").unwrap_err().contains("suffix"));
}

#[test]
fn duration_rejects_empty() {
    assert!(parse_duration("").unwrap_err().contains("empty"));
}

#[test]
fn duration_rejects_no_number() {
    assert!(parse_duration("h").unwrap_err().contains("invalid number"));
}

// --- build_cache_control ---

#[test]
fn cache_control_public_default() {
    let header = build_cache_control("1h", None).unwrap();
    assert_eq!(header, "public, max-age=0, s-maxage=3600");
}

#[test]
fn cache_control_public_with_stale() {
    let header = build_cache_control("5m", Some("1h")).unwrap();
    assert_eq!(
        header,
        "public, max-age=0, s-maxage=300, stale-while-revalidate=3600"
    );
}

#[test]
fn cache_control_private() {
    let header = build_cache_control("private, 10m", None).unwrap();
    assert_eq!(header, "private, max-age=600");
}

#[test]
fn cache_control_private_no_s_maxage() {
    let header = build_cache_control("private, 1h", None).unwrap();
    assert!(!header.contains("s-maxage"));
}

#[test]
fn cache_control_private_with_stale() {
    let header = build_cache_control("private, 5m", Some("1h")).unwrap();
    assert_eq!(header, "private, max-age=300, stale-while-revalidate=3600");
}

// --- parse_handler_attrs ---

#[test]
fn parse_handler_attrs_empty_returns_none() {
    let result = parse_handler_attrs_inner(quote! {}).unwrap();
    assert!(result.cache_config.is_none());
    assert!(result.init_fn.is_none());
}

#[test]
fn parse_handler_attrs_valid_cache() {
    let result = parse_handler_attrs_inner(quote! { cache = "1h" }).unwrap();
    let config = result.cache_config.unwrap();
    assert_eq!(config.cache_control, "public, max-age=0, s-maxage=3600");
    assert!(result.init_fn.is_none());
}

#[test]
fn parse_handler_attrs_cache_with_stale() {
    let result = parse_handler_attrs_inner(quote! { cache = "5m", stale = "1h" }).unwrap();
    let config = result.cache_config.unwrap();
    assert_eq!(
        config.cache_control,
        "public, max-age=0, s-maxage=300, stale-while-revalidate=3600"
    );
}

#[test]
fn parse_handler_attrs_rejects_unknown_key() {
    let err = parse_handler_attrs_inner(quote! { cache = "1h", unknown = "30s" }).unwrap_err();
    assert!(err.to_string().contains("unknown attribute"));
}

#[test]
fn parse_handler_attrs_rejects_missing_cache() {
    let err = parse_handler_attrs_inner(quote! { stale = "1h" }).unwrap_err();
    assert!(err.to_string().contains("missing required `cache`"));
}

#[test]
fn parse_handler_attrs_rejects_non_string_literal() {
    let err = parse_handler_attrs_inner(quote! { cache = 3600 }).unwrap_err();
    assert!(err.to_string().contains("expected a string literal"));
}

#[test]
fn parse_handler_attrs_rejects_duplicate_cache() {
    let err = parse_handler_attrs_inner(quote! { cache = "1h", cache = "2h" }).unwrap_err();
    assert!(err.to_string().contains("duplicate `cache`"));
}

#[test]
fn parse_handler_attrs_rejects_duplicate_stale() {
    let err =
        parse_handler_attrs_inner(quote! { cache = "1h", stale = "1h", stale = "2h" }).unwrap_err();
    assert!(err.to_string().contains("duplicate `stale`"));
}

// --- parse_handler_attrs: init ---

#[test]
fn parse_attrs_init_only() {
    let result = parse_handler_attrs_inner(quote! { init = "setup" }).unwrap();
    assert_eq!(result.init_fn.as_deref(), Some("setup"));
    assert!(result.cache_config.is_none());
}

#[test]
fn parse_attrs_init_and_cache() {
    let result = parse_handler_attrs_inner(quote! { init = "setup", cache = "1h" }).unwrap();
    assert_eq!(result.init_fn.as_deref(), Some("setup"));
    let config = result.cache_config.unwrap();
    assert_eq!(config.cache_control, "public, max-age=0, s-maxage=3600");
}

#[test]
fn parse_attrs_init_empty_rejected() {
    let err = parse_handler_attrs_inner(quote! { init = "" }).unwrap_err();
    assert!(err.to_string().contains("empty"));
}

#[test]
fn parse_attrs_init_non_string_rejected() {
    let err = parse_handler_attrs_inner(quote! { init = 42 }).unwrap_err();
    assert!(err.to_string().contains("string literal"));
}

#[test]
fn parse_attrs_duplicate_init_rejected() {
    let err = parse_handler_attrs_inner(quote! { init = "a", init = "b" }).unwrap_err();
    assert!(err.to_string().contains("duplicate"));
}

#[test]
fn parse_attrs_cache_and_init_both_parsed() {
    let result =
        parse_handler_attrs_inner(quote! { init = "setup", cache = "1h", stale = "5m" }).unwrap();
    assert!(result.cache_config.is_some());
    assert_eq!(result.init_fn.as_deref(), Some("setup"));
}

// --- parse_handler_attrs: timeout ---

#[test]
fn parse_attrs_timeout_only() {
    let result = parse_handler_attrs_inner(quote! { timeout = "30s" }).unwrap();
    assert_eq!(result.timeout_secs, Some(30));
    assert!(result.cache_config.is_none());
    assert!(result.init_fn.is_none());
}

#[test]
fn parse_attrs_timeout_minutes() {
    let result = parse_handler_attrs_inner(quote! { timeout = "5m" }).unwrap();
    assert_eq!(result.timeout_secs, Some(300));
}

#[test]
fn parse_attrs_timeout_with_cache() {
    let result = parse_handler_attrs_inner(quote! { timeout = "30s", cache = "1h" }).unwrap();
    assert_eq!(result.timeout_secs, Some(30));
    assert!(result.cache_config.is_some());
}

#[test]
fn parse_attrs_timeout_with_init() {
    let result = parse_handler_attrs_inner(quote! { timeout = "10s", init = "setup" }).unwrap();
    assert_eq!(result.timeout_secs, Some(10));
    assert_eq!(result.init_fn.as_deref(), Some("setup"));
}

#[test]
fn parse_attrs_timeout_with_all() {
    let result =
        parse_handler_attrs_inner(quote! { timeout = "1h", init = "setup", cache = "5m" }).unwrap();
    assert_eq!(result.timeout_secs, Some(3600));
    assert_eq!(result.init_fn.as_deref(), Some("setup"));
    assert!(result.cache_config.is_some());
}

#[test]
fn parse_attrs_duplicate_timeout_rejected() {
    let err = parse_handler_attrs_inner(quote! { timeout = "30s", timeout = "1m" }).unwrap_err();
    assert!(err.to_string().contains("duplicate"));
}

#[test]
fn parse_attrs_timeout_invalid_duration() {
    let err = parse_handler_attrs_inner(quote! { timeout = "abc" }).unwrap_err();
    assert!(err.to_string().contains("suffix"));
}

#[test]
fn parse_attrs_timeout_empty_rejected() {
    let err = parse_handler_attrs_inner(quote! { timeout = "" }).unwrap_err();
    assert!(err.to_string().contains("empty"));
}

// --- parse_handler_attrs: idempotent ---

#[test]
fn parse_attrs_idempotent_only() {
    let result = parse_handler_attrs_inner(quote! { idempotent }).unwrap();
    assert!(result.idempotent);
    assert!(result.cache_config.is_none());
    assert!(result.init_fn.is_none());
    assert!(result.timeout_secs.is_none());
}

#[test]
fn parse_attrs_idempotent_with_init() {
    let result = parse_handler_attrs_inner(quote! { idempotent, init = "setup" }).unwrap();
    assert!(result.idempotent);
    assert_eq!(result.init_fn.as_deref(), Some("setup"));
}

#[test]
fn parse_attrs_idempotent_with_timeout() {
    let result = parse_handler_attrs_inner(quote! { idempotent, timeout = "30s" }).unwrap();
    assert!(result.idempotent);
    assert_eq!(result.timeout_secs, Some(30));
}

#[test]
fn parse_attrs_idempotent_with_all() {
    let result = parse_handler_attrs_inner(
        quote! { idempotent, init = "setup", timeout = "1h", cache = "5m" },
    )
    .unwrap();
    assert!(result.idempotent);
    assert_eq!(result.init_fn.as_deref(), Some("setup"));
    assert_eq!(result.timeout_secs, Some(3600));
    assert!(result.cache_config.is_some());
}

#[test]
fn parse_attrs_duplicate_idempotent() {
    let err = parse_handler_attrs_inner(quote! { idempotent, idempotent }).unwrap_err();
    assert!(err.to_string().contains("duplicate"));
}

#[test]
fn parse_attrs_idempotent_rejects_value() {
    let err = parse_handler_attrs_inner(quote! { idempotent = "true" }).unwrap_err();
    assert!(err.to_string().contains("bare flag"));
}

#[test]
fn parse_attrs_idempotent_rejects_bool() {
    let err = parse_handler_attrs_inner(quote! { idempotent = true }).unwrap_err();
    assert!(err.to_string().contains("bare flag"));
}

#[test]
fn parse_attrs_empty_has_idempotent_false() {
    let result = parse_handler_attrs_inner(quote! {}).unwrap();
    assert!(!result.idempotent);
}
