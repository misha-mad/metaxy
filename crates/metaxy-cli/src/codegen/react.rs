use crate::model::Manifest;

use super::common::{self, FrameworkConfig};

const QUERY_OPTIONS_INTERFACE: &str = r#"export interface QueryOptions<K extends QueryKey> {
  /**
   * Whether to execute the query. @default true
   *
   * Pass a getter `() => bool` for reactive updates — a plain `boolean` is
   * read once when `useQuery` is called and will not trigger re-fetches.
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
  readonly data: QueryOutput<K> | undefined;

  /** The error from the most recent failed fetch, cleared on next attempt. */
  readonly error: RpcError | undefined;

  /** True while a fetch is in-flight (including the initial fetch). */
  readonly isLoading: boolean;

  /** True after the first successful fetch. Stays true even if a later refetch fails. */
  readonly isSuccess: boolean;

  /** True when the most recent fetch failed. */
  readonly isError: boolean;

  /** Manually trigger a refetch. Works even when `enabled` is false. */
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

  /** True when error is set. */
  readonly isError: boolean;

  /** Reset state back to idle (clear data, error, status). */
  reset: () => void;
}"#;

const USE_QUERY_IMPL: &str = r#"export function useQuery<K extends QueryKey>(
  client: RpcClient,
  ...args: unknown[]
): QueryResult<K> {
  const key = args[0] as K;

  let input: QueryInput<K> | undefined;
  let optionsArg: QueryOptions<K> | (() => QueryOptions<K>) | undefined;

  if (VOID_QUERY_KEYS.has(key)) {
    optionsArg = args[1] as QueryOptions<K> | (() => QueryOptions<K>) | undefined;
  } else {
    input = args[1] as QueryInput<K>;
    optionsArg = args[2] as QueryOptions<K> | (() => QueryOptions<K>) | undefined;
  }

  const optionsArgRef = useRef(optionsArg);
  optionsArgRef.current = optionsArg;

  const resolveOptions = useCallback((): QueryOptions<K> | undefined =>
    typeof optionsArgRef.current === "function"
      ? optionsArgRef.current()
      : optionsArgRef.current,
  []);

  function resolveEnabled(): boolean {
    const opts = resolveOptions();
    return typeof opts?.enabled === "function"
      ? opts.enabled()
      : (opts?.enabled ?? true);
  }

  const [data, setData] = useState<QueryOutput<K> | undefined>(() => resolveOptions()?.placeholderData);
  const [error, setError] = useState<RpcError | undefined>();
  const [isLoading, setIsLoading] = useState(resolveEnabled);
  const [hasFetched, setHasFetched] = useState(false);

  const generationRef = useRef(0);
  const controllerRef = useRef<AbortController | undefined>();
  const inputRef = useRef(input);
  inputRef.current = input;
  const serializedInput = JSON.stringify(input);

  const fetchData = useCallback(async (
    inputVal: QueryInput<K> | undefined,
    signal: AbortSignal,
    gen: number,
  ) => {
    const opts = resolveOptions();
    setIsLoading(true);
    setError(undefined);
    try {
      const callArgs: unknown[] = [key];
      if (inputVal !== undefined) callArgs.push(inputVal);
      const mergedCallOptions = { ...opts?.callOptions, signal: opts?.callOptions?.signal
          ? AbortSignal.any([signal, opts.callOptions.signal])
          : signal };
      callArgs.push(mergedCallOptions);
      const result = await (client.query as (...a: unknown[]) => Promise<unknown>)(
        ...callArgs
      ) as QueryOutput<K>;
      if (gen !== generationRef.current) return;
      setData(result);
      setHasFetched(true);
      opts?.onSuccess?.(result);
    } catch (e) {
      if (gen !== generationRef.current) return;
      const err = e as RpcError;
      setError(err);
      opts?.onError?.(err);
    } finally {
      if (gen === generationRef.current) {
        setIsLoading(false);
        opts?.onSettled?.();
      }
    }
  }, [client, key, resolveOptions]);

  const enabled = resolveEnabled();
  const refetchInterval = resolveOptions()?.refetchInterval;

  useEffect(() => {
    if (!enabled) {
      setIsLoading(false);
      return;
    }

    generationRef.current++;
    const gen = generationRef.current;
    const controller = new AbortController();
    controllerRef.current = controller;
    void fetchData(inputRef.current, controller.signal, gen);

    let interval: ReturnType<typeof setInterval> | undefined;
    if (refetchInterval) {
      interval = setInterval(() => {
        const ctrl = controllerRef.current;
        if (ctrl && !ctrl.signal.aborted) {
          void fetchData(inputRef.current, ctrl.signal, generationRef.current);
        }
      }, refetchInterval);
    }

    return () => {
      controller.abort();
      if (interval) clearInterval(interval);
    };
  }, [fetchData, enabled, serializedInput, refetchInterval]);

  return {
    data, error, isLoading,
    isSuccess: hasFetched,
    isError: error !== undefined,
    refetch: async () => {
      generationRef.current++;
      const gen = generationRef.current;
      const localController = new AbortController();
      if (controllerRef.current) controllerRef.current.abort();
      controllerRef.current = localController;
      await fetchData(inputRef.current, localController.signal, gen);
    },
  };
}"#;

const USE_MUTATION_IMPL: &str = r#"export function useMutation<K extends MutationKey>(
  client: RpcClient,
  key: K,
  options?: MutationOptions<K>,
): MutationResult<K> {
  const [data, setData] = useState<MutationOutput<K> | undefined>();
  const [error, setError] = useState<RpcError | undefined>();
  const [isLoading, setIsLoading] = useState(false);
  const [hasSucceeded, setHasSucceeded] = useState(false);

  const optionsRef = useRef(options);
  optionsRef.current = options;

  const execute = useCallback(async (...input: MutationArgs<K>): Promise<MutationOutput<K>> => {
    setIsLoading(true);
    setError(undefined);
    setHasSucceeded(false);
    try {
      const callArgs: unknown[] = [key];
      if (input.length > 0) callArgs.push(input[0]);
      if (optionsRef.current?.callOptions) callArgs.push(optionsRef.current.callOptions);
      const result = await (client.mutate as (...a: unknown[]) => Promise<unknown>)(...callArgs) as MutationOutput<K>;
      setData(result);
      setHasSucceeded(true);
      optionsRef.current?.onSuccess?.(result);
      return result;
    } catch (e) {
      const err = e as RpcError;
      setError(err);
      optionsRef.current?.onError?.(err);
      throw e;
    } finally {
      setIsLoading(false);
      optionsRef.current?.onSettled?.();
    }
  }, [client, key]);

  const reset = useCallback(() => {
    setData(undefined);
    setError(undefined);
    setIsLoading(false);
    setHasSucceeded(false);
  }, []);

  return {
    mutate: async (...args: MutationArgs<K>) => { await execute(...args); },
    mutateAsync: (...args: MutationArgs<K>) => execute(...args),
    data, error, isLoading,
    isSuccess: hasSucceeded,
    isError: error !== undefined,
    reset,
  };
}"#;

const STREAM_OPTIONS_INTERFACE: &str = r#"export interface StreamOptions<K extends StreamKey> {
  callOptions?: CallOptions;
  onChunk?: (chunk: StreamOutput<K>) => void;
  onDone?: () => void;
  onError?: (error: RpcError) => void;
}"#;

const STREAM_RESULT_INTERFACE: &str = r#"export interface StreamResult<K extends StreamKey> {
  readonly chunks: StreamOutput<K>[];
  readonly error: RpcError | undefined;
  readonly isStreaming: boolean;
  readonly isDone: boolean;
  start: () => void;
  stop: () => void;
}"#;

const USE_STREAM_IMPL: &str = r#"export function useStream<K extends StreamKey>(
  client: RpcClient,
  ...args: unknown[]
): StreamResult<K> {
  const key = args[0] as K;

  let inputArg: StreamInput<K> | undefined;
  let options: StreamOptions<K> | undefined;

  if (VOID_STREAM_KEYS.has(key)) {
    options = args[1] as StreamOptions<K> | undefined;
  } else {
    inputArg = args[1] as StreamInput<K>;
    options = args[2] as StreamOptions<K> | undefined;
  }

  const [chunks, setChunks] = useState<StreamOutput<K>[]>([]);
  const [error, setError] = useState<RpcError | undefined>();
  const [isStreaming, setIsStreaming] = useState(false);
  const [isDone, setIsDone] = useState(false);
  const controllerRef = useRef<AbortController | undefined>();
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const stop = useCallback(() => {
    if (controllerRef.current) {
      controllerRef.current.abort();
      controllerRef.current = undefined;
    }
  }, []);

  const start = useCallback(() => {
    stop();
    const controller = new AbortController();
    controllerRef.current = controller;
    setChunks([]);
    setError(undefined);
    setIsStreaming(true);
    setIsDone(false);

    (async () => {
      try {
        const callArgs: unknown[] = [key];
        if (inputArg !== undefined) callArgs.push(inputArg);
        const mergedCallOptions = { ...optionsRef.current?.callOptions, signal: controller.signal };
        callArgs.push(mergedCallOptions);
        const gen = (client.stream as (...a: unknown[]) => AsyncGenerator<unknown>)(...callArgs);
        for await (const chunk of gen) {
          if (controller.signal.aborted) break;
          setChunks(prev => [...prev, chunk as StreamOutput<K>]);
          optionsRef.current?.onChunk?.(chunk as StreamOutput<K>);
        }
        if (!controller.signal.aborted) {
          setIsDone(true);
          optionsRef.current?.onDone?.();
        }
      } catch (e) {
        if (!controller.signal.aborted) {
          setError(e as RpcError);
          optionsRef.current?.onError?.(e as RpcError);
        }
      } finally {
        setIsStreaming(false);
      }
    })();
  }, [client, key, inputArg, stop]);

  useEffect(() => {
    return () => stop();
  }, [stop]);

  return { chunks, error, isStreaming, isDone, start, stop };
}"#;

const FRAMEWORK_IMPORT: &str =
    "import { useState, useEffect, useRef, useCallback } from \"react\";";

/// Generates the complete React reactive wrapper file content from a manifest.
///
/// Returns an empty string when the manifest contains no procedures (the caller
/// should skip writing the file in that case).
pub fn generate_react_file(
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
            query_fn_name: "useQuery",
            stream_fn_name: "useStream",
            input_as_getter: false,
            query_interfaces: &[QUERY_OPTIONS_INTERFACE, QUERY_RESULT_INTERFACE],
            mutation_interfaces: &[MUTATION_OPTIONS_INTERFACE, MUTATION_RESULT_INTERFACE],
            stream_interfaces: &[STREAM_OPTIONS_INTERFACE, STREAM_RESULT_INTERFACE],
            query_impl: USE_QUERY_IMPL,
            mutation_impl: USE_MUTATION_IMPL,
            stream_impl: USE_STREAM_IMPL,
        },
    )
}
