use crate::model::Manifest;

use super::common::{self, FrameworkConfig};

const QUERY_OPTIONS_INTERFACE: &str = r#"export interface QueryOptions<K extends QueryKey> {
  /**
   * Whether to execute the query. @default true
   *
   * Pass a getter `() => bool` for reactive updates — a plain `boolean` is
   * read once when `createQuery` is called and will not trigger re-fetches.
   */
  enabled?: boolean | (() => boolean);

  /** Auto-refetch interval in milliseconds. Set to 0 or omit to disable. */
  refetchInterval?: number;

  /** Initial data shown before the first fetch completes. */
  placeholderData?: QueryOutput<K>;

  /** Per-call options forwarded to client.query(). */
  callOptions?: CallOptions;

  /** Called when the query succeeds. */
  onSuccess?: (data: QueryOutput<K>) => void;

  /** Called when the query fails. */
  onError?: (error: RpcError) => void;

  /** Called when the query settles (success or failure). */
  onSettled?: () => void;
}"#;

const QUERY_RESULT_INTERFACE: &str = r#"export type QueryStatus = "idle" | "loading" | "success" | "error";

export interface QueryResult<K extends QueryKey> {
  /** The latest successfully resolved data, or placeholderData. */
  readonly data: QueryOutput<K> | undefined;

  /** The error from the most recent failed fetch, cleared on next attempt. */
  readonly error: RpcError | undefined;

  /** Current status of the query. Derived: loading > error > success > idle. */
  readonly status: QueryStatus;

  /** True while a fetch is in-flight (including the initial fetch). */
  readonly isLoading: boolean;

  /** True after the first successful fetch. Stays true even if a later refetch fails. */
  readonly isSuccess: boolean;

  /** True when the most recent fetch failed. */
  readonly isError: boolean;

  /** True when placeholderData is being shown and no real fetch has completed yet. */
  readonly isPlaceholderData: boolean;

  /** Manually trigger a refetch. Works even when `enabled` is false. Resets the polling interval. */
  refetch: () => Promise<void>;
}"#;

const MUTATION_OPTIONS_INTERFACE: &str = r#"export interface MutationOptions<K extends MutationKey> {
  /** Per-call options forwarded to client.mutate(). */
  callOptions?: CallOptions;

  /** Called when the mutation succeeds. */
  onSuccess?: (data: MutationOutput<K>) => void;

  /** Called when the mutation fails. */
  onError?: (error: RpcError) => void;

  /** Called when the mutation settles (success or failure). */
  onSettled?: () => void;
}"#;

const MUTATION_RESULT_INTERFACE: &str = r#"export interface MutationResult<K extends MutationKey> {
  /** Execute the mutation. Rejects on error. */
  mutate: (...args: MutationArgs<K>) => Promise<void>;

  /** Execute the mutation and return the result. Rejects on error. */
  mutateAsync: (...args: MutationArgs<K>) => Promise<MutationOutput<K>>;

  /** The latest successfully resolved data. */
  readonly data: MutationOutput<K> | undefined;

  /** The error from the most recent failed mutation, cleared on next attempt. */
  readonly error: RpcError | undefined;

  /** True while a mutation is in-flight. */
  readonly isLoading: boolean;

  /** True after the most recent mutation succeeded. */
  readonly isSuccess: boolean;

  /** True when the most recent mutation failed. */
  readonly isError: boolean;

  /** Reset state back to idle (clear data, error, status). */
  reset: () => void;
}"#;

const CREATE_QUERY_IMPL: &str = r#"export function createQuery<K extends QueryKey>(
  client: RpcClient,
  ...args: unknown[]
): QueryResult<K> {
  const key = args[0] as K;

  let inputFn: (() => QueryInput<K>) | undefined;
  let optionsArg: QueryOptions<K> | (() => QueryOptions<K>) | undefined;

  if (typeof args[1] === "function" && args[2] !== undefined) {
    inputFn = args[1] as () => QueryInput<K>;
    optionsArg = args[2] as QueryOptions<K> | (() => QueryOptions<K>) | undefined;
  } else if (typeof args[1] === "function") {
    if (VOID_QUERY_KEYS.has(key)) {
      optionsArg = args[1] as () => QueryOptions<K>;
    } else {
      inputFn = args[1] as () => QueryInput<K>;
    }
  } else if (typeof args[1] === "object") {
    optionsArg = args[1] as QueryOptions<K>;
  }

  function resolveOptions(): QueryOptions<K> | undefined {
    return typeof optionsArg === "function" ? optionsArg() : optionsArg;
  }

  function resolveEnabled(): boolean {
    const opts = resolveOptions();
    return typeof opts?.enabled === "function"
      ? opts.enabled()
      : (opts?.enabled ?? true);
  }

  let data = $state<QueryOutput<K> | undefined>(resolveOptions()?.placeholderData);
  let error = $state<RpcError | undefined>();
  let hasFetched = $state(false);
  let loading = $state(false);

  let generation = 0;
  let controller: AbortController | undefined;
  let intervalId: ReturnType<typeof setInterval> | undefined;
  async function fetchData(input: QueryInput<K> | undefined, signal: AbortSignal, gen: number) {
    const opts = resolveOptions();
    loading = true;
    error = undefined;
    try {
      const callArgs: unknown[] = [key];
      if (input !== undefined) callArgs.push(input);
      const mergedCallOptions = { ...opts?.callOptions, signal: opts?.callOptions?.signal
          ? AbortSignal.any([signal, opts.callOptions.signal])
          : signal };
      callArgs.push(mergedCallOptions);
      const result = await (client.query as (...a: unknown[]) => Promise<unknown>)(...callArgs) as QueryOutput<K>;
      if (gen !== generation) return;
      data = result;
      hasFetched = true;
      opts?.onSuccess?.(result);
    } catch (e) {
      if (gen !== generation) return;
      error = e as RpcError;
      opts?.onError?.(error);
    } finally {
      if (gen === generation) {
        loading = false;
        opts?.onSettled?.();
      }
    }
  }

  function setupInterval(enabled: boolean, refetchInterval: number | undefined) {
    if (intervalId) { clearInterval(intervalId); intervalId = undefined; }
    if (enabled && refetchInterval) {
      intervalId = setInterval(() => {
        if (controller && !controller.signal.aborted) {
          void fetchData(inputFn?.(), controller.signal, generation);
        }
      }, refetchInterval);
    }
  }

  $effect(() => {
    const enabled = resolveEnabled();
    const input = inputFn?.();
    const refetchInterval = resolveOptions()?.refetchInterval;

    if (controller) controller.abort();
    if (enabled) {
      generation++;
      const gen = generation;
      controller = new AbortController();
      void fetchData(input, controller.signal, gen);
    } else {
      loading = false;
      controller = undefined;
    }

    setupInterval(enabled, refetchInterval);

    return () => {
      if (intervalId) { clearInterval(intervalId); intervalId = undefined; }
    };
  });

  $effect(() => {
    return () => {
      generation++;
      if (controller) { controller.abort(); controller = undefined; }
    };
  });

  return {
    get data() { return data; },
    get error() { return error; },
    get status(): QueryStatus {
      if (loading) return "loading";
      if (error !== undefined) return "error";
      if (hasFetched) return "success";
      return "idle";
    },
    get isLoading() { return loading; },
    get isSuccess() { return hasFetched; },
    get isError() { return error !== undefined; },
    get isPlaceholderData() { return !hasFetched && data !== undefined; },
    refetch: async () => {
      generation++;
      const gen = generation;
      const localController = new AbortController();
      if (controller) controller.abort();
      controller = localController;
      const enabled = resolveEnabled();
      setupInterval(enabled, resolveOptions()?.refetchInterval);
      await fetchData(inputFn?.(), localController.signal, gen);
    },
  };
}"#;

const CREATE_MUTATION_IMPL: &str = r#"export function createMutation<K extends MutationKey>(
  client: RpcClient,
  key: K,
  options?: MutationOptions<K>,
): MutationResult<K> {
  let data = $state<MutationOutput<K> | undefined>();
  let error = $state<RpcError | undefined>();
  let loading = $state(false);
  let hasSucceeded = $state(false);

  async function execute(...input: MutationArgs<K>): Promise<MutationOutput<K>> {
    loading = true;
    error = undefined;
    hasSucceeded = false;
    try {
      const callArgs: unknown[] = [key];
      if (input.length > 0) callArgs.push(input[0]);
      if (options?.callOptions) callArgs.push(options.callOptions);
      const result = await (client.mutate as (...a: unknown[]) => Promise<unknown>)(...callArgs) as MutationOutput<K>;
      data = result;
      hasSucceeded = true;
      options?.onSuccess?.(result);
      return result;
    } catch (e) {
      error = e as RpcError;
      options?.onError?.(error);
      throw e;
    } finally {
      loading = false;
      options?.onSettled?.();
    }
  }

  return {
    mutate: async (...args: MutationArgs<K>) => { await execute(...args); },
    mutateAsync: (...args: MutationArgs<K>) => execute(...args),
    get data() { return data; },
    get error() { return error; },
    get isLoading() { return loading; },
    get isSuccess() { return hasSucceeded; },
    get isError() { return error !== undefined; },
    reset: () => { data = undefined; error = undefined; loading = false; hasSucceeded = false; },
  } as MutationResult<K>;
}"#;

const STREAM_OPTIONS_INTERFACE: &str = r#"export interface StreamOptions<K extends StreamKey> {
  /** Per-call options forwarded to client.stream(). */
  callOptions?: CallOptions;

  /** Called for each chunk received from the stream. */
  onChunk?: (chunk: StreamOutput<K>) => void;

  /** Called when the stream completes successfully. */
  onDone?: () => void;

  /** Called when the stream encounters an error. */
  onError?: (error: RpcError) => void;
}"#;

const STREAM_RESULT_INTERFACE: &str = r#"export interface StreamResult<K extends StreamKey> {
  /** All chunks received so far. */
  readonly chunks: StreamOutput<K>[];

  /** The error from the stream, if any. */
  readonly error: RpcError | undefined;

  /** True while the stream is active. */
  readonly isStreaming: boolean;

  /** True when the stream has completed without error. */
  readonly isDone: boolean;

  /** Start (or restart) the stream. */
  start: () => void;

  /** Abort the active stream. */
  stop: () => void;
}"#;

const CREATE_STREAM_IMPL: &str = r#"export function createStream<K extends StreamKey>(
  client: RpcClient,
  ...args: unknown[]
): StreamResult<K> {
  const key = args[0] as K;

  let inputFn: (() => StreamInput<K>) | undefined;
  let options: StreamOptions<K> | undefined;

  if (typeof args[1] === "function") {
    inputFn = args[1] as () => StreamInput<K>;
    options = args[2] as StreamOptions<K> | undefined;
  } else if (typeof args[1] === "object" && args[1] !== null && !VOID_STREAM_KEYS.has(key)) {
    inputFn = () => args[1] as StreamInput<K>;
    options = args[2] as StreamOptions<K> | undefined;
  } else {
    options = args[1] as StreamOptions<K> | undefined;
  }

  let chunks = $state<StreamOutput<K>[]>([]);
  let error = $state<RpcError | undefined>();
  let streaming = $state(false);
  let done = $state(false);
  let controller: AbortController | undefined;

  async function run() {
    if (controller) controller.abort();
    controller = new AbortController();
    chunks = [];
    error = undefined;
    streaming = true;
    done = false;

    try {
      const callArgs: unknown[] = [key];
      const input = inputFn?.();
      if (input !== undefined) callArgs.push(input);
      const userSignal = options?.callOptions?.signal;
      const mergedSignal = userSignal ? AbortSignal.any([controller.signal, userSignal]) : controller.signal;
      const mergedCallOptions = { ...options?.callOptions, signal: mergedSignal };
      callArgs.push(mergedCallOptions);
      const gen = (client.stream as (...a: unknown[]) => AsyncGenerator<unknown>)(...callArgs);
      for await (const chunk of gen) {
        if (controller.signal.aborted) break;
        chunks = [...chunks, chunk as StreamOutput<K>];
        options?.onChunk?.(chunk as StreamOutput<K>);
      }
      if (!controller.signal.aborted) {
        done = true;
        options?.onDone?.();
      }
    } catch (e) {
      if (!controller.signal.aborted) {
        error = e as RpcError;
        options?.onError?.(error);
      }
    } finally {
      streaming = false;
    }
  }

  return {
    get chunks() { return chunks; },
    get error() { return error; },
    get isStreaming() { return streaming; },
    get isDone() { return done; },
    start: () => { void run(); },
    stop: () => { if (controller) { controller.abort(); controller = undefined; } },
  } as StreamResult<K>;
}"#;

/// Generates the complete Svelte 5 reactive wrapper file content from a manifest.
///
/// Returns an empty string when the manifest contains no procedures (the caller
/// should skip writing the file in that case).
pub fn generate_svelte_file(
    manifest: &Manifest,
    client_import_path: &str,
    types_import_path: &str,
    preserve_docs: bool,
) -> String {
    common::generate_framework_file(
        manifest,
        client_import_path,
        types_import_path,
        preserve_docs,
        &FrameworkConfig {
            framework_import: None,
            query_fn_name: "createQuery",
            stream_fn_name: "createStream",
            input_as_getter: true,
            query_interfaces: &[QUERY_OPTIONS_INTERFACE, QUERY_RESULT_INTERFACE],
            mutation_interfaces: &[MUTATION_OPTIONS_INTERFACE, MUTATION_RESULT_INTERFACE],
            stream_interfaces: &[STREAM_OPTIONS_INTERFACE, STREAM_RESULT_INTERFACE],
            query_impl: CREATE_QUERY_IMPL,
            mutation_impl: CREATE_MUTATION_IMPL,
            stream_impl: CREATE_STREAM_IMPL,
        },
    )
}
