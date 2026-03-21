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

const QUERY_RESULT_INTERFACE: &str = r#"export interface QueryResult<K extends QueryKey> {
  /** The latest successfully resolved data, or placeholderData. */
  data: () => QueryOutput<K> | undefined;

  /** The error from the most recent failed fetch, cleared on next attempt. */
  error: () => RpcError | undefined;

  /** True while a fetch is in-flight (including the initial fetch). */
  isLoading: () => boolean;

  /** True after the first successful fetch. Stays true even if a later refetch fails. */
  isSuccess: () => boolean;

  /** True when the most recent fetch failed. */
  isError: () => boolean;

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
  data: () => MutationOutput<K> | undefined;

  /** The error from the most recent failed mutation, cleared on next attempt. */
  error: () => RpcError | undefined;

  /** True while a mutation is in-flight. */
  isLoading: () => boolean;

  /** True after the most recent mutation succeeded. */
  isSuccess: () => boolean;

  /** True when the most recent mutation failed. */
  isError: () => boolean;

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

  const initialOpts = resolveOptions();
  const initialEnabled = typeof initialOpts?.enabled === "function"
    ? initialOpts.enabled()
    : (initialOpts?.enabled ?? true);

  const [data, setData] = createSignal<QueryOutput<K> | undefined>(initialOpts?.placeholderData);
  const [error, setError] = createSignal<RpcError | undefined>();
  const [isLoading, setIsLoading] = createSignal(initialEnabled);
  const [hasFetched, setHasFetched] = createSignal(false);

  const resolvedEnabled = createMemo(() => resolveEnabled());
  const isSuccess = createMemo(() => hasFetched());
  const isError = createMemo(() => error() !== undefined);

  let generation = 0;
  let controller: AbortController | undefined;
  let intervalId: ReturnType<typeof setInterval> | undefined;

  async function fetchData(input: QueryInput<K> | undefined, signal: AbortSignal, gen: number) {
    const opts = resolveOptions();
    setIsLoading(true);
    setError(undefined);
    try {
      const callArgs: unknown[] = [key];
      if (input !== undefined) callArgs.push(input);
      const mergedCallOptions = { ...opts?.callOptions, signal: opts?.callOptions?.signal
          ? AbortSignal.any([signal, opts.callOptions.signal])
          : signal };
      callArgs.push(mergedCallOptions);
      const result = await (client.query as (...a: unknown[]) => Promise<unknown>)(
        ...callArgs
      ) as QueryOutput<K>;
      if (gen !== generation) return;
      setData(result as Exclude<QueryOutput<K> | undefined, Function>);
      setHasFetched(true);
      opts?.onSuccess?.(result);
    } catch (e) {
      if (gen !== generation) return;
      const err = e as RpcError;
      setError(err as Exclude<RpcError | undefined, Function>);
      opts?.onError?.(err);
    } finally {
      if (gen === generation) {
        setIsLoading(false);
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

  createEffect(() => {
    const enabled = resolvedEnabled();
    const input = inputFn?.();

    if (controller) controller.abort();
    if (enabled) {
      generation++;
      const gen = generation;
      controller = new AbortController();
      untrack(() => { void fetchData(input, controller.signal, gen); });
    } else {
      setIsLoading(false);
      controller = undefined;
    }

    onCleanup(() => {
      if (controller) { controller.abort(); controller = undefined; }
    });
  });

  createEffect(() => {
    const enabled = resolveEnabled();
    const refetchInterval = resolveOptions()?.refetchInterval;

    setupInterval(enabled, refetchInterval);

    onCleanup(() => {
      if (intervalId) { clearInterval(intervalId); intervalId = undefined; }
    });
  });

  return {
    data: data as () => QueryOutput<K> | undefined,
    error,
    isLoading,
    isSuccess,
    isError,
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
  const [data, setData] = createSignal<MutationOutput<K> | undefined>();
  const [error, setError] = createSignal<RpcError | undefined>();
  const [isLoading, setIsLoading] = createSignal(false);
  const [hasSucceeded, setHasSucceeded] = createSignal(false);

  const isSuccess = createMemo(() => hasSucceeded());
  const isError = createMemo(() => error() !== undefined);

  async function execute(...input: MutationArgs<K>): Promise<MutationOutput<K>> {
    setIsLoading(true);
    setError(undefined);
    setHasSucceeded(false);
    try {
      const callArgs: unknown[] = [key];
      if (input.length > 0) callArgs.push(input[0]);
      if (options?.callOptions) callArgs.push(options.callOptions);
      const result = await (client.mutate as (...a: unknown[]) => Promise<unknown>)(
        ...callArgs
      ) as MutationOutput<K>;
      setData(result as Exclude<MutationOutput<K> | undefined, Function>);
      setHasSucceeded(true);
      options?.onSuccess?.(result);
      return result;
    } catch (e) {
      const err = e as RpcError;
      setError(err as Exclude<RpcError | undefined, Function>);
      options?.onError?.(err);
      throw e;
    } finally {
      setIsLoading(false);
      options?.onSettled?.();
    }
  }

  return {
    mutate: async (...args: MutationArgs<K>) => { await execute(...args); },
    mutateAsync: (...args: MutationArgs<K>) => execute(...args),
    data: data as () => MutationOutput<K> | undefined,
    error,
    isLoading,
    isSuccess,
    isError,
    reset: () => batch(() => { setData(undefined); setError(undefined); setIsLoading(false); setHasSucceeded(false); }),
  };
}"#;

const STREAM_OPTIONS_INTERFACE: &str = r#"export interface StreamOptions<K extends StreamKey> {
  callOptions?: CallOptions;
  onChunk?: (chunk: StreamOutput<K>) => void;
  onDone?: () => void;
  onError?: (error: RpcError) => void;
}"#;

const STREAM_RESULT_INTERFACE: &str = r#"export interface StreamResult<K extends StreamKey> {
  readonly chunks: () => StreamOutput<K>[];
  readonly error: () => RpcError | undefined;
  readonly isStreaming: () => boolean;
  readonly isDone: () => boolean;
  start: () => void;
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

  const [chunks, setChunks] = createSignal<StreamOutput<K>[]>([]);
  const [error, setError] = createSignal<RpcError | undefined>();
  const [isStreaming, setIsStreaming] = createSignal(false);
  const [isDone, setIsDone] = createSignal(false);
  let controller: AbortController | undefined;

  function stop() {
    if (controller) {
      controller.abort();
      controller = undefined;
    }
  }

  async function run() {
    stop();
    controller = new AbortController();
    batch(() => {
      setChunks([]);
      setError(undefined);
      setIsStreaming(true);
      setIsDone(false);
    });

    try {
      const callArgs: unknown[] = [key];
      const input = inputFn?.();
      if (input !== undefined) callArgs.push(input);
      const mergedCallOptions = { ...options?.callOptions, signal: controller.signal };
      callArgs.push(mergedCallOptions);
      const gen = (client.stream as (...a: unknown[]) => AsyncGenerator<unknown>)(...callArgs);
      for await (const chunk of gen) {
        if (controller.signal.aborted) break;
        setChunks(prev => [...prev, chunk as StreamOutput<K>]);
        options?.onChunk?.(chunk as StreamOutput<K>);
      }
      if (!controller.signal.aborted) {
        setIsDone(true);
        options?.onDone?.();
      }
    } catch (e) {
      if (!controller.signal.aborted) {
        setError(e as RpcError);
        options?.onError?.(e as RpcError);
      }
    } finally {
      setIsStreaming(false);
    }
  }

  onCleanup(stop);

  return { chunks, error, isStreaming, isDone, start: () => { void run(); }, stop };
}"#;

const FRAMEWORK_IMPORT: &str = "import { createSignal, createEffect, createMemo, onCleanup, batch, untrack } from \"solid-js\";";

/// Generates the complete SolidJS reactive primitives file content from a manifest.
///
/// Returns an empty string when the manifest contains no procedures (the caller
/// should skip writing the file in that case).
pub fn generate_solid_file(
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
            framework_import: Some(FRAMEWORK_IMPORT),
            query_fn_name: "createQuery",
            input_as_getter: true,
            query_interfaces: &[QUERY_OPTIONS_INTERFACE, QUERY_RESULT_INTERFACE],
            mutation_interfaces: &[MUTATION_OPTIONS_INTERFACE, MUTATION_RESULT_INTERFACE],
            query_impl: CREATE_QUERY_IMPL,
            mutation_impl: CREATE_MUTATION_IMPL,
            stream_fn_name: "createStream",
            stream_interfaces: &[STREAM_OPTIONS_INTERFACE, STREAM_RESULT_INTERFACE],
            stream_impl: CREATE_STREAM_IMPL,
        },
    )
}
