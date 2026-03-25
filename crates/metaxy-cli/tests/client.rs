mod common;

use std::path::PathBuf;

use metaxy_cli::codegen::client::generate_client_file;
use metaxy_cli::model::*;

#[test]
fn contains_generated_header() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.starts_with("// This file is auto-generated"));
}

#[test]
fn imports_procedures_type() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("import type { Procedures } from \"./rpc-types\""));
}

#[test]
fn reexports_procedures_type() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export type { Procedures }"));
}

#[test]
fn contains_rpc_error_class() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export class RpcError extends Error"));
    assert!(output.contains("this.status = status"));
}

#[test]
fn contains_fetch_helper() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("async function rpcFetch("));
    assert!(output.contains("encodeURIComponent(serialized)"));
}

#[test]
fn generates_query_method() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("query(key: \"hello\", input: string): Promise<string>"));
    assert!(output.contains("rpcFetch(config, \"GET\", key, input, callOptions)"));
    assert!(output.contains("export interface RpcClient"));
}

#[test]
fn generates_void_query_overload() {
    let manifest = common::make_manifest(vec![common::make_query(
        "version",
        None,
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("query(key: \"version\"): Promise<string>"));
}

#[test]
fn generates_mutation_method() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("mutate(key: \"create_item\", input: CreateInput): Promise<Item>"));
    assert!(
        output.contains(
            "rpcFetch(config, \"POST\", key, args[0], args[1] as CallOptions | undefined)"
        )
    );
    assert!(output.contains("export interface RpcClient"));
}

#[test]
fn generates_void_mutation_overload() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "reset",
        None,
        Some(RustType::simple("bool")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("mutate(key: \"reset\"): Promise<boolean>"));
}

#[test]
fn generates_create_rpc_client_factory() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export function createRpcClient(config: RpcClientConfig)"));
}

#[test]
fn generates_type_helpers() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("type QueryKey = keyof Procedures[\"queries\"]"));
    assert!(output.contains("type QueryInput<K extends QueryKey>"));
    assert!(output.contains("type QueryOutput<K extends QueryKey>"));
}

#[test]
fn mixed_queries_and_mutations() {
    let manifest = common::make_manifest(vec![
        common::make_query(
            "get_user",
            Some(RustType::simple("String")),
            Some(RustType::simple("User")),
        ),
        common::make_mutation(
            "delete_user",
            Some(RustType::simple("String")),
            Some(RustType::simple("bool")),
        ),
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("query(key: \"get_user\", input: string): Promise<User>"));
    assert!(output.contains("mutate(key: \"delete_user\", input: string): Promise<boolean>"));
}

#[test]
fn empty_manifest_no_methods() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("createRpcClient"));
    assert!(!output.contains("query("));
    assert!(!output.contains("mutate("));
}

#[test]
fn complex_types_in_overloads() {
    let manifest = common::make_manifest(vec![common::make_query(
        "search",
        Some(RustType::simple("SearchQuery")),
        Some(RustType::with_generics(
            "Vec",
            vec![RustType::simple("SearchResult")],
        )),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("query(key: \"search\", input: SearchQuery): Promise<SearchResult[]>"));
}

#[test]
fn uses_get_for_queries_post_for_mutations() {
    let manifest = common::make_manifest(vec![
        common::make_query(
            "q",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
        common::make_mutation(
            "m",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("rpcFetch(config, \"GET\", key, input, callOptions)"));
    assert!(
        output.contains(
            "rpcFetch(config, \"POST\", key, args[0], args[1] as CallOptions | undefined)"
        )
    );
}

#[test]
fn custom_types_import_path() {
    let output = generate_client_file(&common::make_manifest(vec![]), "$lib/rpc-types", false);
    assert!(output.contains("from \"$lib/rpc-types\""));
}

#[test]
fn import_path_with_extension() {
    let output = generate_client_file(&common::make_manifest(vec![]), "./rpc-types.js", false);
    assert!(output.contains("from \"./rpc-types.js\""));
}

#[test]
fn interface_based_overloads() {
    let manifest = common::make_manifest(vec![common::make_query(
        "test",
        None,
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export interface RpcClient"));
    assert!(output.contains("query(key: \"test\"): Promise<string>"));
    assert!(output.contains("createRpcClient(config: RpcClientConfig): RpcClient"));
    assert!(output.contains("as RpcClient"));
}

#[test]
fn response_unwrapping() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("json?.result?.data ?? json"));
}

#[test]
fn imports_referenced_structs() {
    let manifest = Manifest {
        procedures: vec![common::make_query(
            "get_time",
            None,
            Some(RustType::simple("TimeResponse")),
        )],
        structs: vec![StructDef {
            name: "TimeResponse".to_string(),
            generics: vec![],
            fields: vec![common::field("timestamp", RustType::simple("u64"))],
            tuple_fields: vec![],
            source_file: PathBuf::from("api/time.rs"),
            docs: None,
            rename_all: None,
        }],
        enums: vec![],
    };
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("import type { Procedures, TimeResponse } from \"./rpc-types\""));
    assert!(output.contains("export type { Procedures, TimeResponse }"));
}

#[test]
fn error_handling_in_fetch() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("if (!res.ok)"));
    assert!(output.contains("new RpcError("));
    assert!(output.contains("throw rpcError"));
}

#[test]
fn test_jsdoc_on_overload() {
    let manifest = common::make_manifest(vec![
        Procedure {
            name: "hello".to_string(),
            kind: ProcedureKind::Query,
            input: Some(RustType::simple("String")),
            output: Some(RustType::simple("String")),
            source_file: PathBuf::from("api/hello.rs"),
            docs: Some("Say hello.".to_string()),
            timeout_ms: None,
            idempotent: false,
        },
        Procedure {
            name: "reset".to_string(),
            kind: ProcedureKind::Mutation,
            input: None,
            output: Some(RustType::simple("bool")),
            source_file: PathBuf::from("api/reset.rs"),
            docs: Some("Reset state.".to_string()),
            timeout_ms: None,
            idempotent: false,
        },
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", true);
    assert!(
        output.contains(
            "  /** Say hello. */\n  query(key: \"hello\", input: string): Promise<string>;"
        )
    );
    assert!(output.contains("  /** Reset state. */\n  mutate(key: \"reset\"): Promise<boolean>;"));
}

#[test]
fn test_jsdoc_on_void_query_overload() {
    let manifest = common::make_manifest(vec![Procedure {
        name: "version".to_string(),
        kind: ProcedureKind::Query,
        input: None,
        output: Some(RustType::simple("String")),
        source_file: PathBuf::from("api/version.rs"),
        docs: Some("Get version.".to_string()),
        timeout_ms: None,
        idempotent: false,
    }]);
    let output = generate_client_file(&manifest, "./rpc-types", true);
    assert!(output.contains("  /** Get version. */\n  query(key: \"version\"): Promise<string>;"));
}

#[test]
fn test_jsdoc_on_non_void_mutation_overload() {
    let manifest = common::make_manifest(vec![Procedure {
        name: "update".to_string(),
        kind: ProcedureKind::Mutation,
        input: Some(RustType::simple("String")),
        output: Some(RustType::simple("bool")),
        source_file: PathBuf::from("api/update.rs"),
        docs: Some("Update item.".to_string()),
        timeout_ms: None,
        idempotent: false,
    }]);
    let output = generate_client_file(&manifest, "./rpc-types", true);
    assert!(output.contains(
        "  /** Update item. */\n  mutate(key: \"update\", input: string): Promise<boolean>;"
    ));
}

#[test]
fn test_no_jsdoc_on_overload_when_disabled() {
    let manifest = common::make_manifest(vec![Procedure {
        name: "hello".to_string(),
        kind: ProcedureKind::Query,
        input: Some(RustType::simple("String")),
        output: Some(RustType::simple("String")),
        source_file: PathBuf::from("api/hello.rs"),
        docs: Some("Say hello.".to_string()),
        timeout_ms: None,
        idempotent: false,
    }]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(!output.contains("/**"));
}

#[test]
fn contains_config_interface() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export interface RpcClientConfig"));
    assert!(output.contains("baseUrl: string"));
}

#[test]
fn config_has_fetch_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("fetch?: typeof globalThis.fetch"));
}

#[test]
fn config_has_headers_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("headers?:"));
    assert!(output.contains("Record<string, string>"));
}

#[test]
fn config_has_on_request_hook() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("onRequest?: (ctx: RequestContext) => void | Promise<void>"));
}

#[test]
fn config_has_on_response_hook() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("onResponse?: (ctx: ResponseContext) => void | Promise<void>"));
}

#[test]
fn config_has_on_error_hook() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("onError?: (ctx: ErrorContext) => void | Promise<void>"));
}

#[test]
fn contains_request_context_interface() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export interface RequestContext"));
}

#[test]
fn contains_response_context_interface() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export interface ResponseContext"));
}

#[test]
fn contains_error_context_interface() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export interface ErrorContext"));
}

#[test]
fn fetch_helper_calls_on_request() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("config.onRequest"));
}

#[test]
fn fetch_helper_calls_on_response() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("config.onResponse"));
}

#[test]
fn fetch_helper_calls_on_error() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("config.onError"));
}

#[test]
fn fetch_helper_tracks_duration() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("Date.now()"));
}

#[test]
fn contains_retry_policy_interface() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export interface RetryPolicy"));
    assert!(output.contains("attempts: number"));
    assert!(output.contains("delay: number | ((attempt: number) => number)"));
    assert!(output.contains("retryOn?: number[]"));
}

#[test]
fn config_has_retry_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("retry?: RetryPolicy"));
}

#[test]
fn config_has_timeout_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("timeout?: number"));
}

#[test]
fn error_context_has_attempt() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("attempt: number"));
}

#[test]
fn error_context_has_will_retry() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("willRetry: boolean"));
}

#[test]
fn fetch_helper_handles_timeout() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("AbortController"));
    assert!(output.contains("effectiveTimeout"));
    assert!(output.contains("clearTimeout"));
}

#[test]
fn fetch_helper_handles_retry() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("maxAttempts"));
    assert!(output.contains("config.retry"));
    assert!(output.contains("for (let attempt = 1"));
}

#[test]
fn fetch_helper_uses_retry_on() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("DEFAULT_RETRY_ON"));
    assert!(output.contains("[408, 429, 500, 502, 503, 504]"));
    assert!(output.contains("retryOn.includes"));
}

#[test]
fn config_has_serialize_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("serialize?: (input: unknown) => string"));
}

#[test]
fn config_has_deserialize_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("deserialize?: (text: string) => unknown"));
}

#[test]
fn fetch_helper_uses_custom_serialize() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("config.serialize"));
}

#[test]
fn fetch_helper_uses_custom_deserialize() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("config.deserialize"));
}

#[test]
fn config_has_signal_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("signal?: AbortSignal"));
}

#[test]
fn fetch_helper_uses_client_signal() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("config.signal"));
    assert!(output.contains("AbortSignal.any"));
}

// --- insta snapshot tests ---

#[test]
fn snapshot_full_client() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_client_with_methods() {
    let manifest = common::make_manifest(vec![
        common::make_query(
            "get_user",
            Some(RustType::simple("String")),
            Some(RustType::simple("User")),
        ),
        common::make_query("version", None, Some(RustType::simple("String"))),
        common::make_mutation(
            "create_item",
            Some(RustType::simple("CreateInput")),
            Some(RustType::simple("Item")),
        ),
        common::make_mutation("reset", None, Some(RustType::simple("bool"))),
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_client_with_jsdoc() {
    let manifest = common::make_manifest(vec![
        Procedure {
            name: "hello".to_string(),
            kind: ProcedureKind::Query,
            input: Some(RustType::simple("String")),
            output: Some(RustType::simple("String")),
            source_file: PathBuf::from("api/hello.rs"),
            docs: Some("Say hello to someone.".to_string()),
            timeout_ms: None,
            idempotent: false,
        },
        Procedure {
            name: "reset".to_string(),
            kind: ProcedureKind::Mutation,
            input: None,
            output: Some(RustType::simple("bool")),
            source_file: PathBuf::from("api/reset.rs"),
            docs: Some("Reset all state.".to_string()),
            timeout_ms: None,
            idempotent: false,
        },
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", true);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_client_imports_structs() {
    let manifest = Manifest {
        procedures: vec![
            common::make_query("get_time", None, Some(RustType::simple("TimeResponse"))),
            common::make_mutation(
                "create_item",
                Some(RustType::simple("CreateInput")),
                Some(RustType::simple("Item")),
            ),
        ],
        structs: vec![
            StructDef {
                name: "TimeResponse".to_string(),
                generics: vec![],
                fields: vec![common::field("timestamp", RustType::simple("u64"))],
                tuple_fields: vec![],
                source_file: PathBuf::from("api/time.rs"),
                docs: None,
                rename_all: None,
            },
            StructDef {
                name: "CreateInput".to_string(),
                generics: vec![],
                fields: vec![common::field("title", RustType::simple("String"))],
                tuple_fields: vec![],
                source_file: PathBuf::from("api/create.rs"),
                docs: None,
                rename_all: None,
            },
            StructDef {
                name: "Item".to_string(),
                generics: vec![],
                fields: vec![common::field("id", RustType::simple("u64"))],
                tuple_fields: vec![],
                source_file: PathBuf::from("api/create.rs"),
                docs: None,
                rename_all: None,
            },
        ],
        enums: vec![],
    };
    let output = generate_client_file(&manifest, "./rpc-types", false);
    insta::assert_snapshot!(output);
}

// --- CallOptions tests ---

#[test]
fn contains_call_options_interface() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("export interface CallOptions"));
    assert!(output.contains("headers?: Record<string, string>"));
    assert!(output.contains("timeout?: number"));
    assert!(output.contains("signal?: AbortSignal"));
}

#[test]
fn query_overload_with_options() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(
        output.contains(
            "query(key: \"hello\", input: string, options: CallOptions): Promise<string>"
        )
    );
}

#[test]
fn query_void_overload_with_options() {
    let manifest = common::make_manifest(vec![common::make_query(
        "version",
        None,
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("query(key: \"version\", options: CallOptions): Promise<string>"));
}

#[test]
fn mutation_overload_with_options() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains(
        "mutate(key: \"create_item\", input: CreateInput, options: CallOptions): Promise<Item>"
    ));
}

#[test]
fn mutation_void_overload_with_options() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "reset",
        None,
        Some(RustType::simple("bool")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("mutate(key: \"reset\", options: CallOptions): Promise<boolean>"));
}

#[test]
fn fetch_helper_accepts_call_options() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("callOptions?: CallOptions"));
}

#[test]
fn fetch_helper_merges_call_headers() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("callOptions?.headers"));
}

#[test]
fn fetch_helper_uses_call_timeout() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("callOptions?.timeout"));
}

#[test]
fn fetch_helper_uses_call_signal() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("callOptions?.signal"));
}

// --- Dedup tests ---

#[test]
fn config_has_dedupe_option() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("dedupe?: boolean"));
}

#[test]
fn call_options_has_dedupe_field() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let config_idx = output.find("export interface RpcClientConfig").unwrap();
    let call_opts_idx = output.find("export interface CallOptions").unwrap();
    // Both interfaces should contain dedupe
    let config_section = &output[config_idx..call_opts_idx];
    let call_opts_end = output[call_opts_idx..].find('}').unwrap();
    let call_opts_section = &output[call_opts_idx..call_opts_idx + call_opts_end];
    assert!(config_section.contains("dedupe?: boolean"));
    assert!(call_opts_section.contains("dedupe?: boolean"));
}

#[test]
fn contains_dedup_key_function() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("function dedupKey("));
}

#[test]
fn contains_wrap_with_signal_function() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("function wrapWithSignal<T>("));
}

#[test]
fn factory_contains_inflight_map() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("new Map<string, Promise<unknown>>()"));
}

#[test]
fn query_body_contains_dedup_logic() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("shouldDedupe"));
    assert!(output.contains("inflight.get"));
}

#[test]
fn mutate_body_has_no_dedup() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let mutate_idx = output.find("mutate(key: MutationKey").unwrap();
    let mutate_section = &output[mutate_idx..];
    assert!(!mutate_section.contains("inflight"));
    assert!(!mutate_section.contains("dedupKey"));
}

#[test]
fn dedup_omitted_when_no_queries() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(!output.contains("dedupKey"));
    assert!(!output.contains("wrapWithSignal"));
    assert!(!output.contains("inflight"));
}

#[test]
fn snapshot_client_with_dedup() {
    let manifest = common::make_manifest(vec![
        common::make_query(
            "get_user",
            Some(RustType::simple("String")),
            Some(RustType::simple("User")),
        ),
        common::make_query("version", None, Some(RustType::simple("String"))),
        common::make_mutation(
            "create_item",
            Some(RustType::simple("CreateInput")),
            Some(RustType::simple("Item")),
        ),
        common::make_mutation("reset", None, Some(RustType::simple("bool"))),
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    insta::assert_snapshot!(output);
}

#[test]
fn void_query_set_generated() {
    let manifest = common::make_manifest(vec![
        common::make_query("time", None, Some(RustType::simple("String"))),
        common::make_query(
            "hello",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("VOID_QUERIES"));
    assert!(output.contains("new Set([\"time\"])"));
}

#[test]
fn void_query_set_omitted_when_all_non_void() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(!output.contains("VOID_QUERIES"));
}

#[test]
fn snapshot_client_with_call_options() {
    let manifest = common::make_manifest(vec![
        common::make_query(
            "hello",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
        common::make_query("time", None, Some(RustType::simple("String"))),
        common::make_mutation(
            "create_item",
            Some(RustType::simple("CreateInput")),
            Some(RustType::simple("Item")),
        ),
        common::make_mutation("reset", None, Some(RustType::simple("bool"))),
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    insta::assert_snapshot!(output);
}

// --- Procedure timeout tests ---

#[test]
fn procedure_timeouts_emitted() {
    let mut proc = common::make_query(
        "slow",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    );
    proc.timeout_ms = Some(30_000);
    let manifest = common::make_manifest(vec![proc]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("PROCEDURE_TIMEOUTS"));
    assert!(output.contains("\"slow\": 30000"));
}

#[test]
fn procedure_timeouts_empty_when_no_timeouts() {
    let manifest = common::make_manifest(vec![common::make_query(
        "fast",
        None,
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("PROCEDURE_TIMEOUTS: Record<string, number> = {}"));
}

#[test]
fn effective_timeout_uses_procedure_timeout() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("PROCEDURE_TIMEOUTS[procedure]"));
    assert!(
        output.contains("callOptions?.timeout ?? PROCEDURE_TIMEOUTS[procedure] ?? config.timeout")
    );
}

// --- Idempotent mutations tests ---

#[test]
fn idempotent_mutations_set_emitted() {
    let mut proc = common::make_mutation(
        "upsert",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    );
    proc.idempotent = true;
    let manifest = common::make_manifest(vec![proc]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("IDEMPOTENT_MUTATIONS"));
    assert!(output.contains("new Set([\"upsert\"])"));
}

#[test]
fn idempotent_mutations_empty_when_none() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("IDEMPOTENT_MUTATIONS: Set<string> = new Set()"));
}

#[test]
fn retry_guard_checks_idempotent() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("IDEMPOTENT_MUTATIONS.has(procedure)"));
}

// --- Stream timeout / PROCEDURE_TIMEOUTS tests ---

#[test]
fn stream_procedure_timeout_in_procedure_timeouts() {
    let mut proc = common::make_stream("timed_stream", None, Some(RustType::simple("String")));
    proc.timeout_ms = Some(60_000);
    let manifest = common::make_manifest(vec![proc]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("\"timed_stream\": 60000"));
}

#[test]
fn stream_helper_supports_call_options_timeout() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        stream_body.contains("callOptions?.timeout"),
        "rpcStream must apply callOptions.timeout as a client-side abort"
    );
}

#[test]
fn stream_helper_does_not_apply_procedure_timeouts() {
    let mut proc = common::make_stream("slow_stream", None, Some(RustType::simple("String")));
    proc.timeout_ms = Some(30_000);
    let manifest = common::make_manifest(vec![proc]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    // PROCEDURE_TIMEOUTS is defined globally (includes stream timeout)
    assert!(output.contains("PROCEDURE_TIMEOUTS"));
    // But rpcStream must NOT read from it — server handles stream duration
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        !stream_body.contains("PROCEDURE_TIMEOUTS"),
        "rpcStream should not apply PROCEDURE_TIMEOUTS; server manages stream duration"
    );
}

// --- Stream lifecycle hooks tests ---

#[test]
fn stream_helper_calls_on_request() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        stream_body.contains("config.onRequest"),
        "rpcStream must fire onRequest hook before the fetch"
    );
}

#[test]
fn stream_helper_calls_on_error() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        stream_body.contains("config.onError"),
        "rpcStream must fire onError hook when stream fails"
    );
}

#[test]
fn stream_jsdoc_on_overload() {
    let mut proc = common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    );
    proc.docs = Some("Chat stream.".to_string());
    let manifest = common::make_manifest(vec![proc]);
    let output = generate_client_file(&manifest, "./rpc-types", true);
    assert!(
        output.contains("/** Chat stream. */\n  stream(key: \"chat\", input: string)"),
        "stream overload should have JsDoc comment"
    );
}

// --- Stream method tests ---

#[test]
fn generates_stream_method() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("stream("));
    assert!(output.contains("rpcStream"));
}

#[test]
fn generates_void_stream_overload() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "events",
        None,
        Some(RustType::simple("Event")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("stream(key: \"events\""));
}

#[test]
fn generates_stream_type_helpers() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("type StreamKey = keyof Procedures[\"streams\"]"));
    assert!(output.contains("type StreamInput<K extends StreamKey>"));
    assert!(output.contains("type StreamOutput<K extends StreamKey>"));
}

#[test]
fn stream_helper_contains_sse_parsing() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("async function* rpcStream"));
    assert!(output.contains("\"data: \""));
}

#[test]
fn stream_helper_omitted_when_no_streams() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(!output.contains("rpcStream"));
    assert!(!output.contains("stream(key: StreamKey"));
}

#[test]
fn void_streams_set_generated() {
    let manifest = common::make_manifest(vec![
        common::make_stream("events", None, Some(RustType::simple("String"))),
        common::make_stream(
            "chat",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
    ]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("VOID_STREAMS"));
    assert!(output.contains("new Set([\"events\"])"));
}

#[test]
fn stream_factory_method_present() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    assert!(output.contains("stream(key: StreamKey"));
}

#[test]
fn stream_helper_handles_error_event() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    // SSE parser must track event type per message block
    assert!(
        stream_body.contains("eventType"),
        "must track SSE event type"
    );
    // Must detect event: error lines
    assert!(
        stream_body.contains("event: "),
        "must parse event: field from SSE"
    );
    // Must throw RpcError on error events rather than yielding the payload
    assert!(
        stream_body.contains("throw new RpcError"),
        "must throw RpcError on event: error"
    );
}

// --- Stream: combined option tests ---

#[test]
fn stream_helper_merges_config_signal_callopt_signal_and_timeout() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    // config.signal
    assert!(
        stream_body.contains("config.signal"),
        "rpcStream must push config.signal into signals array"
    );
    // callOptions.signal
    assert!(
        stream_body.contains("callOptions?.signal"),
        "rpcStream must push callOptions.signal into signals array"
    );
    // callOptions.timeout creates its own AbortController
    assert!(
        stream_body.contains("callOptions?.timeout"),
        "rpcStream must handle callOptions.timeout"
    );
    // All signals merged with AbortSignal.any()
    assert!(
        stream_body.contains("AbortSignal.any(signals)"),
        "rpcStream must merge signals with AbortSignal.any"
    );
}

#[test]
fn stream_helper_does_not_call_on_response() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        !stream_body.contains("onResponse"),
        "rpcStream must NOT call onResponse — streams have no single response body"
    );
}

#[test]
fn stream_helper_does_not_use_retry() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        !stream_body.contains("retry"),
        "rpcStream must NOT implement retry logic"
    );
    assert!(
        !stream_body.contains("maxAttempts"),
        "rpcStream must NOT have maxAttempts loop"
    );
}

#[test]
fn stream_helper_uses_custom_serialize_for_body() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        stream_body.contains("config.serialize"),
        "rpcStream must use config.serialize for request body when available"
    );
}

#[test]
fn stream_helper_uses_custom_deserialize_for_chunks() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        stream_body.contains("config.deserialize"),
        "rpcStream must use config.deserialize for SSE chunk payloads when available"
    );
}

#[test]
fn stream_helper_merges_config_and_callopt_headers() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    // Must spread both config.headers and callOptions.headers
    assert!(
        stream_body.contains("customHeaders") && stream_body.contains("callOptions?.headers"),
        "rpcStream must merge config.headers (via customHeaders) and callOptions.headers"
    );
}

#[test]
fn stream_helper_does_not_use_config_timeout() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    // rpcFetch uses: callOptions?.timeout ?? PROCEDURE_TIMEOUTS[procedure] ?? config.timeout
    // rpcStream must NOT fall back to config.timeout or PROCEDURE_TIMEOUTS
    assert!(
        !stream_body.contains("config.timeout"),
        "rpcStream must NOT use config.timeout — server manages stream duration"
    );
}

#[test]
fn stream_helper_releases_reader_in_finally() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_client_file(&manifest, "./rpc-types", false);
    let stream_start = output.find("async function* rpcStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        stream_body.contains("reader.releaseLock()"),
        "rpcStream must release the reader in a finally block"
    );
}
