use super::common::{GENERATED_HEADER, is_void_input};
use super::typescript::{emit_jsdoc, rust_type_to_ts};
use crate::model::{Manifest, ProcedureKind};

/// Standard RPC error class with status code and structured error data.
const ERROR_CLASS: &str = r#"export class RpcError extends Error {
  readonly status: number;
  readonly data: unknown;

  constructor(status: number, message: string, data?: unknown) {
    super(message);
    this.name = "RpcError";
    this.status = status;
    this.data = data;
  }
}"#;

/// Context passed to the `onRequest` lifecycle hook.
const REQUEST_CONTEXT_INTERFACE: &str = r#"export interface RequestContext {
  procedure: string;
  method: "GET" | "POST";
  url: string;
  headers: Record<string, string>;
  input?: unknown;
}"#;

/// Context passed to the `onResponse` lifecycle hook.
const RESPONSE_CONTEXT_INTERFACE: &str = r#"export interface ResponseContext {
  procedure: string;
  method: "GET" | "POST";
  url: string;
  response: Response;
  data: unknown;
  duration: number;
}"#;

/// Context passed to the `onError` lifecycle hook.
const ERROR_CONTEXT_INTERFACE: &str = r#"export interface ErrorContext {
  procedure: string;
  method: "GET" | "POST";
  url: string;
  error: unknown;
  attempt: number;
  willRetry: boolean;
}"#;

/// Retry policy configuration.
const RETRY_POLICY_INTERFACE: &str = r#"export interface RetryPolicy {
  attempts: number;
  delay: number | ((attempt: number) => number);
  retryOn?: number[];
}"#;

/// Configuration interface for the RPC client.
const CONFIG_INTERFACE: &str = r#"export interface RpcClientConfig {
  baseUrl: string;
  fetch?: typeof globalThis.fetch;
  headers?:
    | Record<string, string>
    | (() => Record<string, string> | Promise<Record<string, string>>);
  onRequest?: (ctx: RequestContext) => void | Promise<void>;
  onResponse?: (ctx: ResponseContext) => void | Promise<void>;
  onError?: (ctx: ErrorContext) => void | Promise<void>;
  retry?: RetryPolicy;
  timeout?: number;
  serialize?: (input: unknown) => string;
  deserialize?: (text: string) => unknown;
  // AbortSignal for cancelling all requests made by this client.
  signal?: AbortSignal;
  dedupe?: boolean;
}"#;

/// Per-call options that override client-level defaults for a single request.
const CALL_OPTIONS_INTERFACE: &str = r#"export interface CallOptions {
  headers?: Record<string, string>;
  timeout?: number;
  signal?: AbortSignal;
  dedupe?: boolean;
}"#;

/// Computes a dedup map key from procedure name and serialized input.
const DEDUP_KEY_FN: &str = r#"function dedupKey(procedure: string, input: unknown, config: RpcClientConfig): string {
  const serialized = input === undefined
    ? ""
    : config.serialize
      ? config.serialize(input)
      : JSON.stringify(input);
  return procedure + ":" + serialized;
}"#;

/// Wraps a shared promise so that a per-caller AbortSignal can reject independently.
const WRAP_WITH_SIGNAL_FN: &str = r#"function wrapWithSignal<T>(promise: Promise<T>, signal?: AbortSignal): Promise<T> {
  if (!signal) return promise;
  if (signal.aborted) return Promise.reject(signal.reason);
  return new Promise<T>((resolve, reject) => {
    const onAbort = () => reject(signal.reason);
    signal.addEventListener("abort", onAbort, { once: true });
    promise.then(
      (value) => { signal.removeEventListener("abort", onAbort); resolve(value); },
      (error) => { signal.removeEventListener("abort", onAbort); reject(error); },
    );
  });
}"#;

/// Internal fetch helper shared by query and mutate methods.
const FETCH_HELPER: &str = r#"const DEFAULT_RETRY_ON = [408, 429, 500, 502, 503, 504];

async function rpcFetch(
  config: RpcClientConfig,
  method: "GET" | "POST",
  procedure: string,
  input?: unknown,
  callOptions?: CallOptions,
): Promise<unknown> {
  let url = `${config.baseUrl}/${procedure}`;
  const customHeaders = typeof config.headers === "function"
    ? await config.headers()
    : config.headers;
  const baseHeaders: Record<string, string> = { ...customHeaders, ...callOptions?.headers };

  if (method === "GET" && input !== undefined) {
    const serialized = config.serialize ? config.serialize(input) : JSON.stringify(input);
    url += `?input=${encodeURIComponent(serialized)}`;
  } else if (method === "POST" && input !== undefined) {
    baseHeaders["Content-Type"] = "application/json";
  }

  const fetchFn = config.fetch ?? globalThis.fetch;
  const maxAttempts = 1 + (config.retry?.attempts ?? 0);
  const retryOn = config.retry?.retryOn ?? DEFAULT_RETRY_ON;
  const effectiveTimeout = callOptions?.timeout ?? PROCEDURE_TIMEOUTS[procedure] ?? config.timeout;
  const start = Date.now();

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const reqCtx: RequestContext = { procedure, method, url, headers: { ...baseHeaders }, input };
    await config.onRequest?.(reqCtx);

    const init: RequestInit = { method, headers: reqCtx.headers };
    if (method === "POST" && input !== undefined) {
      init.body = config.serialize ? config.serialize(input) : JSON.stringify(input);
    }

    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    const signals: AbortSignal[] = [];
    if (config.signal) signals.push(config.signal);
    if (callOptions?.signal) signals.push(callOptions.signal);
    if (effectiveTimeout) {
      const controller = new AbortController();
      timeoutId = setTimeout(() => controller.abort(), effectiveTimeout);
      signals.push(controller.signal);
    }
    if (signals.length > 0) {
      init.signal = signals.length === 1 ? signals[0] : AbortSignal.any(signals);
    }

    const isRetryable = attempt < maxAttempts && (method === "GET" || IDEMPOTENT_MUTATIONS.has(procedure));

    try {
      const res = await fetchFn(url, init);

      if (!res.ok) {
        let data: unknown;
        try {
          data = await res.json();
        } catch {
          data = await res.text().catch(() => null);
        }
        const rpcError = new RpcError(
          res.status,
          `RPC error on "${procedure}": ${res.status} ${res.statusText}`,
          data,
        );
        const canRetry = retryOn.includes(res.status) && isRetryable;
        await config.onError?.({ procedure, method, url, error: rpcError, attempt, willRetry: canRetry });
        if (!canRetry) throw rpcError;
      } else {
        const json = config.deserialize ? config.deserialize(await res.text()) : await res.json();
        const result = json?.result?.data ?? json;
        const duration = Date.now() - start;
        await config.onResponse?.({ procedure, method, url, response: res, data: result, duration });
        return result;
      }
    } catch (err) {
      if (err instanceof RpcError) throw err;
      await config.onError?.({ procedure, method, url, error: err, attempt, willRetry: isRetryable });
      if (!isRetryable) throw err;
    } finally {
      if (timeoutId !== undefined) clearTimeout(timeoutId);
    }

    if (config.retry) {
      const d = typeof config.retry.delay === "function"
        ? config.retry.delay(attempt) : config.retry.delay;
      await new Promise(r => setTimeout(r, d));
    }
  }
}"#;

/// Internal SSE stream helper for the `stream()` method.
const STREAM_HELPER: &str = r#"async function* rpcStream<T>(
  config: RpcClientConfig,
  procedure: string,
  input?: unknown,
  callOptions?: CallOptions,
): AsyncGenerator<T> {
  let url = `${config.baseUrl}/${procedure}`;
  const customHeaders = typeof config.headers === "function"
    ? await config.headers()
    : config.headers;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...customHeaders,
    ...callOptions?.headers,
  };

  const fetchFn = config.fetch ?? globalThis.fetch;
  const init: RequestInit = { method: "POST", headers };
  if (input !== undefined) {
    init.body = config.serialize ? config.serialize(input) : JSON.stringify(input);
  }

  const signals: AbortSignal[] = [];
  if (config.signal) signals.push(config.signal);
  if (callOptions?.signal) signals.push(callOptions.signal);
  if (signals.length > 0) {
    init.signal = signals.length === 1 ? signals[0] : AbortSignal.any(signals);
  }

  const res = await fetchFn(url, init);
  if (!res.ok) {
    let data: unknown;
    try { data = await res.json(); } catch { data = null; }
    throw new RpcError(res.status, `RPC stream error on "${procedure}": ${res.status} ${res.statusText}`, data);
  }

  const reader = res.body!.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      const parts = buffer.split("\n\n");
      buffer = parts.pop()!;
      for (const part of parts) {
        for (const line of part.split("\n")) {
          if (line.startsWith("data: ")) {
            const payload = line.slice(6);
            yield (config.deserialize ? config.deserialize(payload) : JSON.parse(payload)) as T;
          }
        }
      }
    }
  } finally {
    reader.releaseLock();
  }
}"#;

/// Generates the complete `rpc-client.ts` file content from a manifest.
///
/// The output includes:
/// 1. Auto-generation header
/// 2. Re-export of `Procedures` type from the types file
/// 3. `RpcError` class for structured error handling
/// 4. Internal `rpcFetch` helper
/// 5. `createRpcClient` factory function with fully typed `query` / `mutate` methods
pub fn generate_client_file(
    manifest: &Manifest,
    types_import_path: &str,
    preserve_docs: bool,
) -> String {
    let mut out = String::with_capacity(2048);

    // Header
    out.push_str(GENERATED_HEADER);
    out.push('\n');

    // Collect all user-defined type names (structs + enums) for import
    let type_names: Vec<&str> = manifest
        .structs
        .iter()
        .map(|s| s.name.as_str())
        .chain(manifest.enums.iter().map(|e| e.name.as_str()))
        .collect();

    // Import Procedures type (and any referenced types) from the types file
    if type_names.is_empty() {
        emit!(
            out,
            "import type {{ Procedures }} from \"{types_import_path}\";\n"
        );
        emit!(out, "export type {{ Procedures }};\n");
    } else {
        let types_csv = type_names.join(", ");
        emit!(
            out,
            "import type {{ Procedures, {types_csv} }} from \"{types_import_path}\";\n"
        );
        emit!(out, "export type {{ Procedures, {types_csv} }};\n");
    }

    // Error class
    emit!(out, "{ERROR_CLASS}\n");

    // Lifecycle hook context interfaces
    emit!(out, "{REQUEST_CONTEXT_INTERFACE}\n");
    emit!(out, "{RESPONSE_CONTEXT_INTERFACE}\n");
    emit!(out, "{ERROR_CONTEXT_INTERFACE}\n");

    // Retry policy interface
    emit!(out, "{RETRY_POLICY_INTERFACE}\n");

    // Client config interface
    emit!(out, "{CONFIG_INTERFACE}\n");

    // Per-call options interface
    emit!(out, "{CALL_OPTIONS_INTERFACE}\n");

    // Per-procedure timeout defaults (ms)
    generate_procedure_timeouts(manifest, &mut out);

    // Idempotent mutations set (for retry gating)
    generate_idempotent_mutations(manifest, &mut out);

    // Internal fetch helper
    emit!(out, "{FETCH_HELPER}\n");

    // Dedup helpers (only when the manifest has queries)
    let has_queries = manifest
        .procedures
        .iter()
        .any(|p| p.kind == ProcedureKind::Query);
    if has_queries {
        emit!(out, "{DEDUP_KEY_FN}\n");
        emit!(out, "{WRAP_WITH_SIGNAL_FN}\n");
    }

    // Stream helper (only when the manifest has streams)
    let has_streams = manifest
        .procedures
        .iter()
        .any(|p| p.kind == ProcedureKind::Stream);
    if has_streams {
        emit!(out, "{STREAM_HELPER}\n");
    }

    // Type helpers for ergonomic API
    generate_type_helpers(&mut out);
    out.push('\n');

    // Client factory
    generate_client_factory(manifest, preserve_docs, &mut out);

    out
}

/// Emits the `PROCEDURE_TIMEOUTS` record mapping procedure names to their default timeout in ms.
fn generate_procedure_timeouts(manifest: &Manifest, out: &mut String) {
    let entries: Vec<_> = manifest
        .procedures
        .iter()
        .filter_map(|p| p.timeout_ms.map(|ms| format!("  \"{}\": {}", p.name, ms)))
        .collect();

    if entries.is_empty() {
        emit!(
            out,
            "const PROCEDURE_TIMEOUTS: Record<string, number> = {{}};\n"
        );
    } else {
        emit!(out, "const PROCEDURE_TIMEOUTS: Record<string, number> = {{");
        for entry in &entries {
            emit!(out, "{entry},");
        }
        emit!(out, "}};\n");
    }
}

/// Emits the `IDEMPOTENT_MUTATIONS` set listing mutations marked as safe to retry.
fn generate_idempotent_mutations(manifest: &Manifest, out: &mut String) {
    let names: Vec<_> = manifest
        .procedures
        .iter()
        .filter(|p| p.idempotent)
        .map(|p| format!("\"{}\"", p.name))
        .collect();

    if names.is_empty() {
        emit!(
            out,
            "const IDEMPOTENT_MUTATIONS: Set<string> = new Set();\n"
        );
    } else {
        emit!(
            out,
            "const IDEMPOTENT_MUTATIONS: Set<string> = new Set([{}]);\n",
            names.join(", ")
        );
    }
}

/// Emits utility types that power the typed client API.
fn generate_type_helpers(out: &mut String) {
    emit!(out, "type QueryKey = keyof Procedures[\"queries\"];");
    emit!(out, "type MutationKey = keyof Procedures[\"mutations\"];");
    emit!(out, "type StreamKey = keyof Procedures[\"streams\"];");
    emit!(
        out,
        "type QueryInput<K extends QueryKey> = Procedures[\"queries\"][K][\"input\"];"
    );
    emit!(
        out,
        "type QueryOutput<K extends QueryKey> = Procedures[\"queries\"][K][\"output\"];"
    );
    emit!(
        out,
        "type MutationInput<K extends MutationKey> = Procedures[\"mutations\"][K][\"input\"];"
    );
    emit!(
        out,
        "type MutationOutput<K extends MutationKey> = Procedures[\"mutations\"][K][\"output\"];"
    );
    emit!(
        out,
        "type StreamInput<K extends StreamKey> = Procedures[\"streams\"][K][\"input\"];"
    );
    emit!(
        out,
        "type StreamOutput<K extends StreamKey> = Procedures[\"streams\"][K][\"output\"];"
    );
}

/// Generates the `createRpcClient` factory using an interface for typed overloads.
fn generate_client_factory(manifest: &Manifest, preserve_docs: bool, out: &mut String) {
    let queries: Vec<_> = manifest
        .procedures
        .iter()
        .filter(|p| p.kind == ProcedureKind::Query)
        .collect();
    let mutations: Vec<_> = manifest
        .procedures
        .iter()
        .filter(|p| p.kind == ProcedureKind::Mutation)
        .collect();
    let streams: Vec<_> = manifest
        .procedures
        .iter()
        .filter(|p| p.kind == ProcedureKind::Stream)
        .collect();
    let has_queries = !queries.is_empty();
    let has_mutations = !mutations.is_empty();
    let has_streams = !streams.is_empty();

    // Partition queries and mutations by void/non-void input
    let void_queries: Vec<_> = queries.iter().filter(|p| is_void_input(p)).collect();
    let non_void_queries: Vec<_> = queries.iter().filter(|p| !is_void_input(p)).collect();
    let void_mutations: Vec<_> = mutations.iter().filter(|p| is_void_input(p)).collect();
    let non_void_mutations: Vec<_> = mutations.iter().filter(|p| !is_void_input(p)).collect();

    let void_streams: Vec<_> = streams.iter().filter(|p| is_void_input(p)).collect();
    let non_void_streams: Vec<_> = streams.iter().filter(|p| !is_void_input(p)).collect();

    let query_mixed = !void_queries.is_empty() && !non_void_queries.is_empty();
    let mutation_mixed = !void_mutations.is_empty() && !non_void_mutations.is_empty();
    let stream_mixed = !void_streams.is_empty() && !non_void_streams.is_empty();

    // Emit VOID_QUERIES/VOID_MUTATIONS sets when mixed void/non-void exists
    if query_mixed {
        let names: Vec<_> = void_queries
            .iter()
            .map(|p| format!("\"{}\"", p.name))
            .collect();
        emit!(
            out,
            "const VOID_QUERIES: Set<string> = new Set([{}]);",
            names.join(", ")
        );
        out.push('\n');
    }
    if mutation_mixed {
        let names: Vec<_> = void_mutations
            .iter()
            .map(|p| format!("\"{}\"", p.name))
            .collect();
        emit!(
            out,
            "const VOID_MUTATIONS: Set<string> = new Set([{}]);",
            names.join(", ")
        );
        out.push('\n');
    }
    if stream_mixed {
        let names: Vec<_> = void_streams
            .iter()
            .map(|p| format!("\"{}\"", p.name))
            .collect();
        emit!(
            out,
            "const VOID_STREAMS: Set<string> = new Set([{}]);",
            names.join(", ")
        );
        out.push('\n');
    }

    // Emit the RpcClient interface with overloaded method signatures
    emit!(out, "export interface RpcClient {{");

    if has_queries {
        generate_query_overloads(manifest, preserve_docs, out);
    }

    if has_mutations {
        if has_queries {
            out.push('\n');
        }
        generate_mutation_overloads(manifest, preserve_docs, out);
    }

    if has_streams {
        if has_queries || has_mutations {
            out.push('\n');
        }
        generate_stream_overloads(manifest, preserve_docs, out);
    }

    emit!(out, "}}");
    out.push('\n');

    // Emit the factory function
    emit!(
        out,
        "export function createRpcClient(config: RpcClientConfig): RpcClient {{"
    );

    if has_queries {
        emit!(
            out,
            "  const inflight = new Map<string, Promise<unknown>>();\n"
        );
    }

    emit!(out, "  return {{");

    if has_queries {
        emit!(
            out,
            "    query(key: QueryKey, ...args: unknown[]): Promise<unknown> {{"
        );

        // Extract input and callOptions into locals based on void/non-void branching
        if query_mixed {
            emit!(out, "      let input: unknown;");
            emit!(out, "      let callOptions: CallOptions | undefined;");
            emit!(out, "      if (VOID_QUERIES.has(key)) {{");
            emit!(out, "        input = undefined;");
            emit!(
                out,
                "        callOptions = args[0] as CallOptions | undefined;"
            );
            emit!(out, "      }} else {{");
            emit!(out, "        input = args[0];");
            emit!(
                out,
                "        callOptions = args[1] as CallOptions | undefined;"
            );
            emit!(out, "      }}");
        } else if !void_queries.is_empty() {
            emit!(out, "      const input = undefined;");
            emit!(
                out,
                "      const callOptions = args[0] as CallOptions | undefined;"
            );
        } else {
            emit!(out, "      const input = args[0];");
            emit!(
                out,
                "      const callOptions = args[1] as CallOptions | undefined;"
            );
        }

        // Dedup logic
        emit!(
            out,
            "      const shouldDedupe = callOptions?.dedupe ?? config.dedupe ?? true;"
        );
        emit!(out, "      if (shouldDedupe) {{");
        emit!(out, "        const k = dedupKey(key, input, config);");
        emit!(out, "        const existing = inflight.get(k);");
        emit!(
            out,
            "        if (existing) return wrapWithSignal(existing, callOptions?.signal);"
        );
        emit!(
            out,
            "        const promise = rpcFetch(config, \"GET\", key, input, callOptions)"
        );
        emit!(out, "          .finally(() => inflight.delete(k));");
        emit!(out, "        inflight.set(k, promise);");
        emit!(
            out,
            "        return wrapWithSignal(promise, callOptions?.signal);"
        );
        emit!(out, "      }}");
        emit!(
            out,
            "      return rpcFetch(config, \"GET\", key, input, callOptions);"
        );
        emit!(out, "    }},");
    }

    if has_mutations {
        emit!(
            out,
            "    mutate(key: MutationKey, ...args: unknown[]): Promise<unknown> {{"
        );
        if mutation_mixed {
            // Mixed: use VOID_MUTATIONS set to branch at runtime
            emit!(out, "      if (VOID_MUTATIONS.has(key)) {{");
            emit!(
                out,
                "        return rpcFetch(config, \"POST\", key, undefined, args[0] as CallOptions | undefined);"
            );
            emit!(out, "      }}");
            emit!(
                out,
                "      return rpcFetch(config, \"POST\", key, args[0], args[1] as CallOptions | undefined);"
            );
        } else if !void_mutations.is_empty() {
            // All void: args[0] is always CallOptions
            emit!(
                out,
                "      return rpcFetch(config, \"POST\", key, undefined, args[0] as CallOptions | undefined);"
            );
        } else {
            // All non-void: args[0] is input, args[1] is CallOptions
            emit!(
                out,
                "      return rpcFetch(config, \"POST\", key, args[0], args[1] as CallOptions | undefined);"
            );
        }
        emit!(out, "    }},");
    }

    if has_streams {
        emit!(
            out,
            "    stream(key: StreamKey, ...args: unknown[]): AsyncGenerator<unknown> {{"
        );
        if stream_mixed {
            emit!(out, "      if (VOID_STREAMS.has(key)) {{");
            emit!(
                out,
                "        return rpcStream(config, key, undefined, args[0] as CallOptions | undefined);"
            );
            emit!(out, "      }}");
            emit!(
                out,
                "      return rpcStream(config, key, args[0], args[1] as CallOptions | undefined);"
            );
        } else if !void_streams.is_empty() {
            emit!(
                out,
                "      return rpcStream(config, key, undefined, args[0] as CallOptions | undefined);"
            );
        } else {
            emit!(
                out,
                "      return rpcStream(config, key, args[0], args[1] as CallOptions | undefined);"
            );
        }
        emit!(out, "    }},");
    }

    emit!(out, "  }} as RpcClient;");
    emit!(out, "}}");
}

/// Generates query overload signatures for the RpcClient interface.
fn generate_query_overloads(manifest: &Manifest, preserve_docs: bool, out: &mut String) {
    let (void_queries, non_void_queries): (Vec<_>, Vec<_>) = manifest
        .procedures
        .iter()
        .filter(|p| p.kind == ProcedureKind::Query)
        .partition(|p| is_void_input(p));

    // Overload signatures for void-input queries (no input argument required)
    for proc in &void_queries {
        if preserve_docs && let Some(doc) = &proc.docs {
            emit_jsdoc(doc, "  ", out);
        }
        let output_ts = proc
            .output
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        emit!(
            out,
            "  query(key: \"{}\"): Promise<{}>;",
            proc.name,
            output_ts,
        );
        emit!(
            out,
            "  query(key: \"{}\", options: CallOptions): Promise<{}>;",
            proc.name,
            output_ts,
        );
    }

    // Overload signatures for non-void-input queries
    for proc in &non_void_queries {
        if preserve_docs && let Some(doc) = &proc.docs {
            emit_jsdoc(doc, "  ", out);
        }
        let input_ts = proc
            .input
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        let output_ts = proc
            .output
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        emit!(
            out,
            "  query(key: \"{}\", input: {}): Promise<{}>;",
            proc.name,
            input_ts,
            output_ts,
        );
        emit!(
            out,
            "  query(key: \"{}\", input: {}, options: CallOptions): Promise<{}>;",
            proc.name,
            input_ts,
            output_ts,
        );
    }
}

/// Generates mutation overload signatures for the RpcClient interface.
fn generate_mutation_overloads(manifest: &Manifest, preserve_docs: bool, out: &mut String) {
    let (void_mutations, non_void_mutations): (Vec<_>, Vec<_>) = manifest
        .procedures
        .iter()
        .filter(|p| p.kind == ProcedureKind::Mutation)
        .partition(|p| is_void_input(p));

    // Overload signatures for void-input mutations
    for proc in &void_mutations {
        if preserve_docs && let Some(doc) = &proc.docs {
            emit_jsdoc(doc, "  ", out);
        }
        let output_ts = proc
            .output
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        emit!(
            out,
            "  mutate(key: \"{}\"): Promise<{}>;",
            proc.name,
            output_ts,
        );
        emit!(
            out,
            "  mutate(key: \"{}\", options: CallOptions): Promise<{}>;",
            proc.name,
            output_ts,
        );
    }

    // Overload signatures for non-void-input mutations
    for proc in &non_void_mutations {
        if preserve_docs && let Some(doc) = &proc.docs {
            emit_jsdoc(doc, "  ", out);
        }
        let input_ts = proc
            .input
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        let output_ts = proc
            .output
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        emit!(
            out,
            "  mutate(key: \"{}\", input: {}): Promise<{}>;",
            proc.name,
            input_ts,
            output_ts,
        );
        emit!(
            out,
            "  mutate(key: \"{}\", input: {}, options: CallOptions): Promise<{}>;",
            proc.name,
            input_ts,
            output_ts,
        );
    }
}

/// Generates stream overload signatures for the RpcClient interface.
fn generate_stream_overloads(manifest: &Manifest, preserve_docs: bool, out: &mut String) {
    let (void_streams, non_void_streams): (Vec<_>, Vec<_>) = manifest
        .procedures
        .iter()
        .filter(|p| p.kind == ProcedureKind::Stream)
        .partition(|p| is_void_input(p));

    // Overload signatures for void-input streams
    for proc in &void_streams {
        if preserve_docs && let Some(doc) = &proc.docs {
            emit_jsdoc(doc, "  ", out);
        }
        let output_ts = proc
            .output
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        emit!(
            out,
            "  stream(key: \"{}\"): AsyncGenerator<{}>;",
            proc.name,
            output_ts,
        );
        emit!(
            out,
            "  stream(key: \"{}\", options: CallOptions): AsyncGenerator<{}>;",
            proc.name,
            output_ts,
        );
    }

    // Overload signatures for non-void-input streams
    for proc in &non_void_streams {
        if preserve_docs && let Some(doc) = &proc.docs {
            emit_jsdoc(doc, "  ", out);
        }
        let input_ts = proc
            .input
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        let output_ts = proc
            .output
            .as_ref()
            .map(rust_type_to_ts)
            .unwrap_or_else(|| "void".to_string());
        emit!(
            out,
            "  stream(key: \"{}\", input: {}): AsyncGenerator<{}>;",
            proc.name,
            input_ts,
            output_ts,
        );
        emit!(
            out,
            "  stream(key: \"{}\", input: {}, options: CallOptions): AsyncGenerator<{}>;",
            proc.name,
            input_ts,
            output_ts,
        );
    }
}
