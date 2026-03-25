use super::helpers::{no_attrs, parse_fn};
use crate::attrs::{CacheConfig, HandlerAttrs};
use crate::codegen::{HandlerKind, build_handler};
use crate::codegen_stream::build_stream_handler;
use syn::ItemFn;

// --- generate_handler: query ---

#[test]
fn query_no_input() {
    let func = parse_fn("async fn version() -> String { \"1.0\".into() }");
    let tokens = build_handler(func, HandlerKind::Query, no_attrs()).unwrap();
    let code = tokens.to_string();
    assert!(code.contains("\"GET\""));
    assert!(code.contains("__rpc_handler"));
    assert!(code.contains("__rpc_ok_response"));
}

#[test]
fn query_with_input() {
    let func = parse_fn("async fn hello(name: String) -> String { name }");
    let tokens = build_handler(func, HandlerKind::Query, no_attrs()).unwrap();
    let code = tokens.to_string();
    assert!(code.contains("\"GET\""));
    assert!(code.contains("input"));
}

#[test]
fn query_returns_result() {
    let func = parse_fn("async fn fetch(id: u32) -> Result<String, String> { Ok(\"ok\".into()) }");
    let tokens = build_handler(func, HandlerKind::Query, no_attrs()).unwrap();
    let code = tokens.to_string();
    assert!(code.contains("__rpc_error_response (400"));
    assert!(code.contains("Ok (__val)"));
    assert!(code.contains("Err (__err)"));
}

#[test]
fn query_no_return_type() {
    let func = parse_fn("async fn ping() {}");
    let tokens = build_handler(func, HandlerKind::Query, no_attrs()).unwrap();
    let code = tokens.to_string();
    assert!(code.contains("__rpc_ok_response"));
}

// --- generate_handler: mutation ---

#[test]
fn mutation_with_input() {
    let func = parse_fn("async fn create(input: Data) -> Data { input }");
    let tokens = build_handler(func, HandlerKind::Mutation, no_attrs()).unwrap();
    let code = tokens.to_string();
    assert!(code.contains("\"POST\""));
    assert!(code.contains("into_body"));
}

#[test]
fn mutation_no_input() {
    let func = parse_fn("async fn reset() -> u32 { 0 }");
    let tokens = build_handler(func, HandlerKind::Mutation, no_attrs()).unwrap();
    let code = tokens.to_string();
    assert!(code.contains("\"POST\""));
}

// --- generate_handler: errors ---

#[test]
fn rejects_multiple_params() {
    let func = parse_fn("async fn bad(a: String, b: u32) -> String { a }");
    let err = build_handler(func, HandlerKind::Query, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("at most one input parameter"));
}

#[test]
fn rejects_non_async_function() {
    let func: ItemFn = syn::parse_str("fn sync_handler() -> String { \"hi\".into() }").unwrap();
    let err = build_handler(func, HandlerKind::Query, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("must be async"));
}

#[test]
fn self_receiver_ignored() {
    let func: ItemFn =
        syn::parse_str("async fn method(self, name: String) -> String { name }").unwrap();
    let tokens = build_handler(func, HandlerKind::Query, no_attrs()).unwrap();
    let code = tokens.to_string();
    // `self` is filtered out, only `name: String` remains as input
    assert!(code.contains("input"));
}

// --- generate_handler: shared structure ---

#[test]
fn generates_cors_headers() {
    let func = parse_fn("async fn ping() -> String { \"pong\".into() }");
    let code = build_handler(func, HandlerKind::Query, no_attrs())
        .unwrap()
        .to_string();
    assert!(code.contains("Access-Control-Allow-Origin"));
    assert!(code.contains("Access-Control-Allow-Methods"));
    assert!(code.contains("Access-Control-Max-Age"));
}

#[test]
fn generates_options_handler() {
    let func = parse_fn("async fn ping() -> String { \"pong\".into() }");
    let code = build_handler(func, HandlerKind::Query, no_attrs())
        .unwrap()
        .to_string();
    assert!(code.contains("\"OPTIONS\""));
    assert!(code.contains("204"));
}

#[test]
fn generates_current_thread_runtime() {
    let func = parse_fn("async fn ping() -> String { \"pong\".into() }");
    let code = build_handler(func, HandlerKind::Query, no_attrs())
        .unwrap()
        .to_string();
    assert!(code.contains("new_current_thread"));
    assert!(!code.contains("tokio :: main"));
}

#[test]
fn generates_method_not_allowed() {
    let func = parse_fn("async fn ping() -> String { \"pong\".into() }");
    let code = build_handler(func, HandlerKind::Query, no_attrs())
        .unwrap()
        .to_string();
    assert!(code.contains("405"));
    assert!(code.contains("not allowed"));
}

// --- generate_handler: caching ---

#[test]
fn query_with_cache_header() {
    let func = parse_fn("async fn get_settings() -> String { String::new() }");
    let attrs = HandlerAttrs {
        cache_config: Some(CacheConfig {
            cache_control: "public, max-age=0, s-maxage=3600".into(),
        }),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("Cache-Control"));
    assert!(code.contains("s-maxage=3600"));
}

#[test]
fn query_with_stale_while_revalidate() {
    let func = parse_fn("async fn get_feed() -> String { String::new() }");
    let attrs = HandlerAttrs {
        cache_config: Some(CacheConfig {
            cache_control: "public, max-age=0, s-maxage=300, stale-while-revalidate=3600".into(),
        }),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("stale-while-revalidate=3600"));
}

#[test]
fn query_with_private_cache() {
    let func = parse_fn("async fn get_profile() -> String { String::new() }");
    let attrs = HandlerAttrs {
        cache_config: Some(CacheConfig {
            cache_control: "private, max-age=600".into(),
        }),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("private, max-age=600"));
    assert!(!code.contains("s-maxage"));
}

#[test]
fn query_without_cache_no_header() {
    let func = parse_fn("async fn plain() -> String { String::new() }");
    let code = build_handler(func, HandlerKind::Query, no_attrs())
        .unwrap()
        .to_string();
    assert!(!code.contains("Cache-Control"));
}

#[test]
fn mutation_never_has_cache_header() {
    let func = parse_fn("async fn create(input: String) -> String { input }");
    let code = build_handler(func, HandlerKind::Mutation, no_attrs())
        .unwrap()
        .to_string();
    assert!(!code.contains("Cache-Control"));
}

#[test]
fn error_response_never_has_cache_header() {
    let func = parse_fn("async fn risky(id: u32) -> Result<String, String> { Ok(\"ok\".into()) }");
    let attrs = HandlerAttrs {
        cache_config: Some(CacheConfig {
            cache_control: "public, max-age=0, s-maxage=3600".into(),
        }),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    // Cache-Control appears in __rpc_ok_response but not __rpc_error_response
    let ok_section = code.split("__rpc_error_response").next().unwrap();
    let err_section = code.split("__rpc_ok_response").last().unwrap();
    assert!(ok_section.contains("Cache-Control"));
    assert!(!err_section.contains("Cache-Control"));
}

// --- generate_handler: init ---

#[test]
fn mutation_with_init_no_cache() {
    let func = parse_fn("async fn create(input: String) -> String { input }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Mutation, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("\"POST\""));
    assert!(!code.contains("Cache-Control"));
}

#[test]
fn init_side_effects_only() {
    let func = parse_fn("async fn get_data() -> String { String::new() }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("setup () . await"));
    assert!(!code.contains("OnceLock"));
    assert!(!code.contains("__RPC_STATE"));
}

#[test]
fn init_with_state() {
    let func = parse_fn("async fn get_data(state: &AppState) -> String { String::new() }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("OnceLock"));
    assert!(code.contains("__RPC_STATE"));
    assert!(code.contains("__state"));
    assert!(code.contains("setup () . await"));
}

#[test]
fn init_with_state_and_input() {
    let func = parse_fn("async fn get_user(id: u32, state: &AppState) -> String { String::new() }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("__input"));
    assert!(code.contains("__state"));
    assert!(code.contains("OnceLock"));
}

#[test]
fn init_with_state_and_headers() {
    let func = parse_fn(
        "async fn get_data(state: &AppState, headers: Headers) -> String { String::new() }",
    );
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("__state"));
    assert!(code.contains("__headers"));
    assert!(code.contains("OnceLock"));
}

#[test]
fn init_with_all_three_params() {
    let func = parse_fn(
        "async fn get_user(id: u32, state: &AppState, headers: Headers) -> String { String::new() }",
    );
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("__input"));
    assert!(code.contains("__state"));
    assert!(code.contains("__headers"));
}

#[test]
fn init_compatible_with_cache() {
    let func = parse_fn("async fn get_data(state: &AppState) -> String { String::new() }");
    let attrs = HandlerAttrs {
        cache_config: Some(CacheConfig {
            cache_control: "public, max-age=0, s-maxage=3600".into(),
        }),
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("OnceLock"));
    assert!(code.contains("Cache-Control"));
}

#[test]
fn init_compatible_with_mutation() {
    let func = parse_fn("async fn create(input: String, state: &AppState) -> String { input }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Mutation, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("\"POST\""));
    assert!(code.contains("OnceLock"));
    assert!(code.contains("__state"));
}

#[test]
fn init_call_inside_block_on() {
    let func = parse_fn("async fn get_data() -> String { String::new() }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    // The init call must be inside block_on (after .block_on(async {)
    let block_on_section = code.split("block_on").last().unwrap();
    assert!(block_on_section.contains("setup () . await"));
}

// --- generate_handler: init error cases ---

#[test]
fn invalid_init_path_rejected() {
    let func = parse_fn("async fn get_data() -> String { String::new() }");
    // Unclosed delimiter fails proc_macro2::TokenStream parsing.
    let attrs = HandlerAttrs {
        init_fn: Some("setup(".into()),
        ..HandlerAttrs::default()
    };
    let err = build_handler(func, HandlerKind::Query, attrs).unwrap_err();
    assert!(err.to_string().contains("invalid init function path"));
}

#[test]
fn state_without_init_rejected() {
    let func = parse_fn("async fn get_data(state: &AppState) -> String { String::new() }");
    let err = build_handler(func, HandlerKind::Query, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("init"));
}

#[test]
fn mut_state_rejected() {
    let func = parse_fn("async fn get_data(state: &mut AppState) -> String { String::new() }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let err = build_handler(func, HandlerKind::Query, attrs).unwrap_err();
    assert!(err.to_string().contains("shared reference"));
}

#[test]
fn multiple_state_params_rejected() {
    let func =
        parse_fn("async fn get_data(a: &AppState, b: &OtherState) -> String { String::new() }");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let err = build_handler(func, HandlerKind::Query, attrs).unwrap_err();
    assert!(err.to_string().contains("at most one state parameter"));
}

// --- generate_handler: timeout ---

#[test]
fn query_with_timeout_wraps_call() {
    let func = parse_fn("async fn slow() -> String { String::new() }");
    let attrs = HandlerAttrs {
        timeout_secs: Some(30),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("timeout"));
    assert!(code.contains("Duration :: from_secs (30u64)"));
    assert!(code.contains("504"));
    assert!(code.contains("Handler timed out"));
}

#[test]
fn mutation_with_timeout_wraps_call() {
    let func = parse_fn("async fn slow(input: String) -> String { input }");
    let attrs = HandlerAttrs {
        timeout_secs: Some(60),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Mutation, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("timeout"));
    assert!(code.contains("Duration :: from_secs (60u64)"));
    assert!(code.contains("504"));
}

#[test]
fn no_timeout_no_wrapper() {
    let func = parse_fn("async fn fast() -> String { String::new() }");
    let code = build_handler(func, HandlerKind::Query, no_attrs())
        .unwrap()
        .to_string();
    assert!(!code.contains("Duration :: from_secs"));
    assert!(!code.contains("Handler timed out"));
}

#[test]
fn timeout_with_cache_and_init() {
    let func = parse_fn("async fn heavy(state: &AppState) -> String { String::new() }");
    let attrs = HandlerAttrs {
        cache_config: Some(CacheConfig {
            cache_control: "public, max-age=0, s-maxage=3600".into(),
        }),
        init_fn: Some("setup".into()),
        timeout_secs: Some(120),
        ..HandlerAttrs::default()
    };
    let code = build_handler(func, HandlerKind::Query, attrs)
        .unwrap()
        .to_string();
    assert!(code.contains("timeout"));
    assert!(code.contains("Duration :: from_secs (120u64)"));
    assert!(code.contains("Cache-Control"));
    assert!(code.contains("OnceLock"));
}

// --- generate_handler: idempotent ---

#[test]
fn mutation_idempotent_no_codegen_diff() {
    let func = parse_fn("async fn upsert(input: String) -> String { input }");
    let plain_attrs = no_attrs();
    let idempotent_attrs = HandlerAttrs {
        idempotent: true,
        ..HandlerAttrs::default()
    };
    let plain_code = build_handler(func.clone(), HandlerKind::Mutation, plain_attrs)
        .unwrap()
        .to_string();
    let idempotent_code = build_handler(func, HandlerKind::Mutation, idempotent_attrs)
        .unwrap()
        .to_string();
    assert_eq!(plain_code, idempotent_code);
}

// --- build_stream_handler: valid streams ---

#[test]
fn stream_basic_with_sender() {
    let func = parse_fn("async fn events(tx: StreamSender) {}");
    let code = build_stream_handler(func, no_attrs()).unwrap().to_string();
    assert!(code.contains("stream_response"));
    assert!(code.contains("VercelLayer"));
    assert!(code.contains("tokio :: main"));
    assert!(code.contains("Router"));
    assert!(code.contains("post"));
    assert!(code.contains("StreamSender :: new"));
}

#[test]
fn stream_with_input_and_sender() {
    let func = parse_fn("async fn chat(input: ChatInput, tx: StreamSender) {}");
    let code = build_stream_handler(func, no_attrs()).unwrap().to_string();
    assert!(code.contains("Json"));
    assert!(code.contains("__input"));
}

#[test]
fn stream_with_headers_and_sender() {
    let func = parse_fn("async fn events(headers: Headers, tx: StreamSender) {}");
    let code = build_stream_handler(func, no_attrs()).unwrap().to_string();
    assert!(code.contains("HeaderMap"));
    assert!(code.contains("__headers"));
}

#[test]
fn stream_with_all_params() {
    let func = parse_fn(
        "async fn events(input: Msg, state: &AppState, headers: Headers, tx: StreamSender) {}",
    );
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_stream_handler(func, attrs).unwrap().to_string();
    assert!(code.contains("__input"));
    assert!(code.contains("__state"));
    assert!(code.contains("__headers"));
    assert!(code.contains("OnceLock"));
    assert!(code.contains("setup"));
}

#[test]
fn stream_with_timeout() {
    let func = parse_fn("async fn slow(tx: StreamSender) {}");
    let attrs = HandlerAttrs {
        timeout_secs: Some(30),
        ..HandlerAttrs::default()
    };
    let code = build_stream_handler(func, attrs).unwrap().to_string();
    assert!(code.contains("timeout_at"));
    assert!(code.contains("Duration :: from_secs (30u64)"));
    assert!(code.contains("event: error"));
}

#[test]
fn stream_with_init_side_effects() {
    let func = parse_fn("async fn events(tx: StreamSender) {}");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_stream_handler(func, attrs).unwrap().to_string();
    assert!(code.contains("setup () . await"));
    assert!(!code.contains("OnceLock"));
}

#[test]
fn stream_with_init_and_state() {
    let func = parse_fn("async fn events(state: &AppState, tx: StreamSender) {}");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let code = build_stream_handler(func, attrs).unwrap().to_string();
    assert!(code.contains("OnceLock"));
    assert!(code.contains("__RPC_STATE"));
}

#[test]
fn stream_with_timeout_and_input() {
    let func = parse_fn("async fn chat(input: ChatInput, tx: StreamSender) {}");
    let attrs = HandlerAttrs {
        timeout_secs: Some(60),
        ..HandlerAttrs::default()
    };
    let code = build_stream_handler(func, attrs).unwrap().to_string();
    assert!(code.contains("__input"));
    assert!(code.contains("timeout_at"));
    assert!(code.contains("Duration :: from_secs (60u64)"));
    assert!(code.contains("event: error"));
}

#[test]
fn stream_with_init_and_timeout() {
    let func = parse_fn("async fn events(tx: StreamSender) {}");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        timeout_secs: Some(45),
        ..HandlerAttrs::default()
    };
    let code = build_stream_handler(func, attrs).unwrap().to_string();
    assert!(code.contains("setup () . await"));
    assert!(code.contains("timeout_at"));
    assert!(code.contains("Duration :: from_secs (45u64)"));
    assert!(code.contains("event: error"));
}

#[test]
fn stream_with_all_attrs_and_params() {
    let func = parse_fn(
        "async fn chat(input: ChatInput, state: &AppState, headers: Headers, tx: StreamSender) {}",
    );
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        timeout_secs: Some(120),
        ..HandlerAttrs::default()
    };
    let code = build_stream_handler(func, attrs).unwrap().to_string();
    assert!(code.contains("__input"), "must parse input");
    assert!(code.contains("__state"), "must inject state");
    assert!(code.contains("__headers"), "must pass headers");
    assert!(
        code.contains("OnceLock"),
        "must use OnceLock for state init"
    );
    assert!(code.contains("setup"), "must call init function");
    assert!(code.contains("timeout_at"), "must apply timeout");
    assert!(
        code.contains("Duration :: from_secs (120u64)"),
        "must use 120s"
    );
    assert!(
        code.contains("event: error"),
        "timeout must send SSE error event"
    );
}

// --- build_stream_handler: error cases ---

#[test]
fn stream_rejects_non_async() {
    let func: ItemFn = syn::parse_str("fn events(tx: StreamSender) {}").unwrap();
    let err = build_stream_handler(func, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("must be async"));
}

#[test]
fn stream_rejects_return_type() {
    let func = parse_fn("async fn events(tx: StreamSender) -> String { String::new() }");
    let err = build_stream_handler(func, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("must not have a return type"));
}

#[test]
fn stream_rejects_missing_sender() {
    let func = parse_fn("async fn events(input: String) {}");
    let err = build_stream_handler(func, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("must accept a StreamSender"));
}

#[test]
fn stream_rejects_multiple_senders() {
    let func = parse_fn("async fn events(tx1: StreamSender, tx2: StreamSender) {}");
    let err = build_stream_handler(func, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("at most one StreamSender"));
}

#[test]
fn stream_rejects_multiple_inputs() {
    let func = parse_fn("async fn events(a: String, b: u32, tx: StreamSender) {}");
    let err = build_stream_handler(func, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("at most one input"));
}

#[test]
fn stream_rejects_state_without_init() {
    let func = parse_fn("async fn events(state: &AppState, tx: StreamSender) {}");
    let err = build_stream_handler(func, no_attrs()).unwrap_err();
    assert!(err.to_string().contains("init"));
}

#[test]
fn stream_rejects_mut_state() {
    let func = parse_fn("async fn events(state: &mut AppState, tx: StreamSender) {}");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let err = build_stream_handler(func, attrs).unwrap_err();
    assert!(err.to_string().contains("shared reference"));
}

#[test]
fn stream_rejects_multiple_state() {
    let func = parse_fn("async fn events(a: &AppState, b: &Other, tx: StreamSender) {}");
    let attrs = HandlerAttrs {
        init_fn: Some("setup".into()),
        ..HandlerAttrs::default()
    };
    let err = build_stream_handler(func, attrs).unwrap_err();
    assert!(err.to_string().contains("at most one state"));
}

// --- build_stream_handler: incompatible attrs ---

#[test]
fn stream_no_cors_headers() {
    let func = parse_fn("async fn events(tx: StreamSender) {}");
    let code = build_stream_handler(func, no_attrs()).unwrap().to_string();
    assert!(!code.contains("Access-Control-Allow-Origin"));
}

#[test]
fn stream_no_options_handler() {
    let func = parse_fn("async fn events(tx: StreamSender) {}");
    let code = build_stream_handler(func, no_attrs()).unwrap().to_string();
    assert!(!code.contains("\"OPTIONS\""));
    assert!(!code.contains("405"));
}
