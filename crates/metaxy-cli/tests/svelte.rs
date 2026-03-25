mod common;

use std::path::PathBuf;

use metaxy_cli::codegen::svelte::generate_svelte_file;
use metaxy_cli::model::*;

// --- imports ---

#[test]
fn svelte_imports_client_and_types() {
    let manifest = common::make_test_manifest();
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(
        output.contains(
            "import { type RpcClient, RpcError, type CallOptions } from \"./rpc-client\""
        )
    );
    assert!(output.contains("import type { Procedures"));
    assert!(output.contains("from \"./rpc-types\""));
}

#[test]
fn svelte_reexports() {
    let manifest = common::make_test_manifest();
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export { RpcError }"));
    assert!(output.contains("export type { RpcClient, CallOptions, Procedures"));
}

// --- createQuery ---

#[test]
fn svelte_contains_create_query() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export function createQuery"));
}

#[test]
fn svelte_contains_create_mutation() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export function createMutation"));
}

// --- Interfaces ---

#[test]
fn svelte_contains_query_options() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export interface QueryOptions<K extends QueryKey>"));
}

#[test]
fn svelte_contains_query_result() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export interface QueryResult<K extends QueryKey>"));
}

#[test]
fn svelte_contains_mutation_options() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export interface MutationOptions<K extends MutationKey>"));
}

#[test]
fn svelte_contains_mutation_result() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export interface MutationResult<K extends MutationKey>"));
}

// --- Runes ---

#[test]
fn svelte_uses_state_rune() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("$state"));
}

#[test]
fn svelte_uses_effect_rune() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("$effect"));
}

// --- Void/non-void ---

#[test]
fn svelte_void_query_overload() {
    let manifest = common::make_manifest(vec![common::make_query(
        "time",
        None,
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("type VoidQueryKey = \"time\""));
    assert!(output.contains(
        "createQuery<K extends \"time\">(client: RpcClient, key: K, options?: QueryOptions<K> | (() => QueryOptions<K>)): QueryResult<K>"
    ));
}

#[test]
fn svelte_non_void_query_overload() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("type NonVoidQueryKey = \"hello\""));
    assert!(output.contains(
        "createQuery<K extends \"hello\">(client: RpcClient, key: K, input: () => QueryInput<K>, options?: QueryOptions<K> | (() => QueryOptions<K>)): QueryResult<K>"
    ));
}

#[test]
fn svelte_void_query_set() {
    let manifest = common::make_manifest(vec![
        common::make_query("time", None, Some(RustType::simple("String"))),
        common::make_query(
            "hello",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
    ]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("type VoidQueryKey = \"time\""));
    assert!(output.contains("type NonVoidQueryKey = \"hello\""));
}

#[test]
fn svelte_no_void_query_type_when_all_non_void() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(!output.contains("type VoidQueryKey"));
}

// --- Conditional emission ---

#[test]
fn svelte_queries_only_no_mutation() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("createQuery"));
    assert!(!output.contains("createMutation"));
    assert!(!output.contains("MutationKey"));
}

#[test]
fn svelte_mutations_only_no_query() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("createMutation"));
    assert!(!output.contains("createQuery"));
    assert!(!output.contains("QueryKey"));
}

#[test]
fn svelte_empty_manifest_returns_empty() {
    let manifest = common::make_manifest(vec![]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.is_empty());
}

// --- Result members ---

#[test]
fn svelte_refetch_in_result() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("refetch: async ()"));
    assert!(output.contains("await fetchData(inputFn?.(), localController.signal, gen)"));
}

#[test]
fn svelte_reset_in_mutation_result() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("reset: ()"));
}

#[test]
fn svelte_mutate_async_in_result() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "create_item",
        Some(RustType::simple("CreateInput")),
        Some(RustType::simple("Item")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("mutateAsync"));
}

// --- Custom import paths ---

#[test]
fn svelte_custom_import_paths() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "$lib/rpc-client.js", "$lib/rpc-types.js", false);
    assert!(output.contains("from \"$lib/rpc-client.js\""));
    assert!(output.contains("from \"$lib/rpc-types.js\""));
}

// --- Type helpers ---

#[test]
fn svelte_type_helpers_emitted() {
    let manifest = common::make_test_manifest();
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("type QueryKey = keyof Procedures[\"queries\"]"));
    assert!(output.contains("type MutationKey = keyof Procedures[\"mutations\"]"));
    assert!(output.contains("type QueryInput<K extends QueryKey>"));
    assert!(output.contains("type QueryOutput<K extends QueryKey>"));
    assert!(output.contains("type MutationInput<K extends MutationKey>"));
    assert!(output.contains("type MutationOutput<K extends MutationKey>"));
}

// --- User type imports ---

#[test]
fn svelte_imports_user_types() {
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
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("import type { Procedures, TimeResponse } from \"./rpc-types\""));
    assert!(output.contains("TimeResponse"));
}

// --- Polling cleanup ---

#[test]
fn svelte_refetch_interval_cleanup() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("setInterval"));
    assert!(output.contains("clearInterval"));
    assert!(output.contains("controller.abort()"));
}

// --- Void mutation ---

#[test]
fn svelte_void_mutation_key() {
    let manifest = common::make_manifest(vec![common::make_mutation(
        "reset",
        None,
        Some(RustType::simple("bool")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("type VoidMutationKey = \"reset\""));
}

// --- Status enum ---

#[test]
fn svelte_query_status_enum() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(
        output.contains(r#"export type QueryStatus = "idle" | "loading" | "success" | "error""#)
    );
}

#[test]
fn svelte_query_is_placeholder_data() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("isPlaceholderData"));
    assert!(output.contains("!hasFetched && data !== undefined"));
}

#[test]
fn svelte_status_derives_booleans() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    // Status is derived from atomic state, not stored
    assert!(output.contains("get status(): QueryStatus"));
    assert!(output.contains("get isLoading() { return loading; }"));
    assert!(output.contains("get isSuccess() { return hasFetched; }"));
    assert!(output.contains("get isError() { return error !== undefined; }"));
}

// --- AbortController & reactive options ---

#[test]
fn svelte_abort_controller_in_effect() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("new AbortController()"));
    assert!(output.contains("controller.abort()"));
}

#[test]
fn svelte_fetch_receives_signal() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("controller.signal"));
}

#[test]
fn svelte_abort_guard_in_catch() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    // Generation counter guards replace signal?.aborted
    assert!(output.contains("gen !== generation"));
    assert!(output.contains("gen === generation"));
}

#[test]
fn svelte_signal_merge() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("AbortSignal.any("));
}

#[test]
fn svelte_options_accepts_getter() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("QueryOptions<K> | (() => QueryOptions<K>)"));
}

#[test]
fn svelte_resolve_options_helper() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("resolveOptions"));
}

// --- VOID_QUERY_KEYS ---

#[test]
fn svelte_void_query_keys_set() {
    let manifest = common::make_manifest(vec![
        common::make_query("time", None, Some(RustType::simple("String"))),
        common::make_query(
            "hello",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
    ]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains(r#"const VOID_QUERY_KEYS: Set<QueryKey> = new Set(["time"])"#));
    assert!(output.contains("VOID_QUERY_KEYS.has(key)"));
}

#[test]
fn svelte_void_query_keys_multiple() {
    let manifest = common::make_manifest(vec![
        common::make_query("time", None, Some(RustType::simple("String"))),
        common::make_query("version", None, Some(RustType::simple("String"))),
    ]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains(r#"new Set(["time", "version"])"#));
}

// --- Generation counter ---

#[test]
fn svelte_generation_counter() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("let generation = 0"));
    assert!(output.contains("generation++"));
    assert!(output.contains("gen !== generation"));
}

// --- insta snapshot tests ---

#[test]
fn snapshot_svelte_full() {
    let manifest = common::make_test_manifest();
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_svelte_queries_only() {
    let manifest = common::make_manifest(vec![
        common::make_query(
            "get_user",
            Some(RustType::simple("String")),
            Some(RustType::simple("User")),
        ),
        common::make_query("version", None, Some(RustType::simple("String"))),
    ]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_svelte_mutations_only() {
    let manifest = common::make_manifest(vec![
        common::make_mutation(
            "create_item",
            Some(RustType::simple("CreateInput")),
            Some(RustType::simple("Item")),
        ),
        common::make_mutation("reset", None, Some(RustType::simple("bool"))),
    ]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    insta::assert_snapshot!(output);
}

// --- Stream tests ---

#[test]
fn svelte_contains_create_stream() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export function createStream"));
}

#[test]
fn svelte_stream_options_interface() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export interface StreamOptions<K extends StreamKey>"));
}

#[test]
fn svelte_stream_result_interface() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("export interface StreamResult<K extends StreamKey>"));
}

#[test]
fn svelte_stream_type_helpers() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("type StreamKey = keyof Procedures[\"streams\"]"));
    assert!(output.contains("type StreamInput<K extends StreamKey>"));
    assert!(output.contains("type StreamOutput<K extends StreamKey>"));
}

#[test]
fn svelte_stream_void_overload() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "events",
        None,
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains(
        "createStream<K extends \"events\">(client: RpcClient, key: K, options?: StreamOptions<K>): StreamResult<K>"
    ));
}

#[test]
fn svelte_stream_non_void_overload() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains(
        "createStream<K extends \"chat\">(client: RpcClient, key: K, input: () => StreamInput<K>, options?: StreamOptions<K>): StreamResult<K>"
    ));
}

#[test]
fn svelte_stream_uses_state_rune() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("$state"));
}

#[test]
fn svelte_stream_result_has_start_stop() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("start:"));
    assert!(output.contains("stop:"));
    assert!(output.contains("chunks"));
    assert!(output.contains("isStreaming"));
    assert!(output.contains("isDone"));
}

#[test]
fn svelte_stream_abort_controller() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "chat",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("AbortController"));
    assert!(output.contains("controller.abort()"));
}

#[test]
fn svelte_stream_void_keys_set() {
    let manifest = common::make_manifest(vec![
        common::make_stream("events", None, Some(RustType::simple("String"))),
        common::make_stream(
            "chat",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
    ]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("VOID_STREAM_KEYS"));
    assert!(output.contains("new Set([\"events\"])"));
}

#[test]
fn svelte_no_stream_when_no_streams() {
    let manifest = common::make_manifest(vec![common::make_query(
        "hello",
        Some(RustType::simple("String")),
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(!output.contains("createStream"));
    assert!(!output.contains("StreamKey"));
}

#[test]
fn svelte_streams_only_no_query_no_mutation() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "events",
        None,
        Some(RustType::simple("String")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    assert!(output.contains("createStream"));
    assert!(!output.contains("createQuery"));
    assert!(!output.contains("createMutation"));
}

#[test]
fn svelte_stream_merges_user_signal_with_controller() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "events",
        None,
        Some(RustType::simple("Event")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    let stream_start = output.find("createStream").unwrap();
    let stream_body = &output[stream_start..];
    assert!(
        stream_body.contains("AbortSignal.any"),
        "createStream must merge user callOptions.signal with internal AbortController"
    );
}

#[test]
fn svelte_stream_passes_call_options_through() {
    let manifest = common::make_manifest(vec![common::make_stream(
        "events",
        None,
        Some(RustType::simple("Event")),
    )]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    let stream_start = output.find("createStream").unwrap();
    let stream_body = &output[stream_start..];
    // callOptions must be spread into mergedCallOptions so timeout, headers etc. pass through
    assert!(
        stream_body.contains("...options?.callOptions"),
        "createStream must spread callOptions so timeout and headers are forwarded to rpcStream"
    );
}

#[test]
fn snapshot_svelte_streams_only() {
    let manifest = common::make_manifest(vec![
        common::make_stream(
            "chat",
            Some(RustType::simple("String")),
            Some(RustType::simple("String")),
        ),
        common::make_stream("events", None, Some(RustType::simple("Event"))),
    ]);
    let output = generate_svelte_file(&manifest, "./rpc-client", "./rpc-types", false);
    insta::assert_snapshot!(output);
}
