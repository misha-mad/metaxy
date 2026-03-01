import { highlightBlocks } from '$lib/highlight.server';
import type { PageServerLoad } from './$types';

const codeBlocks: Record<string, { code: string; lang: 'rust' | 'typescript' }> = {
	exampleRust: {
		lang: 'rust',
		code: `#[rpc_mutation]
async fn create_order(input: OrderInput) -> Order {
    db::insert_order(input).await
}`
	},
	voidRust: {
		lang: 'rust',
		code: `// Void-input mutation — no arguments needed
#[rpc_mutation]
async fn reset() -> bool {
    cache::clear_all().await;
    true
}

// Result error handling in mutations
#[rpc_mutation]
async fn delete_user(id: u64) -> Result<(), String> {
    if !db::user_exists(id).await {
        return Err("User not found".into());
    }
    db::delete_user(id).await;
    Ok(())
}`
	},
	exampleTs: {
		lang: 'typescript',
		code: `// Direct call
const order = await rpc.mutate('create_order', orderInput);

// With options
const order = await rpc.mutate('create_order', orderInput, {
  timeout: 10000,
});

// Void-input mutation
await rpc.mutate('reset');

// Result<T, E> — errors throw RpcError
try {
  await rpc.mutate('delete_user', userId);
} catch (e) {
  if (e instanceof RpcError) {
    console.error(e.status, e.data);
  }
}`
	},
	exampleSvelte: {
		lang: 'typescript',
		code: `// Reactive wrapper — triggered explicitly via .mutate()
const order = createMutation(rpc, 'create_order', {
  onSuccess: (data) => console.log('Created:', data),
  onError: (err) => console.error(err),
});

// Fire-and-forget
order.mutate(orderInput);

// Await the result
const result = await order.mutateAsync(orderInput);

// Access state
// order.data       — Order | undefined
// order.error      — RpcError | undefined
// order.isLoading  — boolean
// order.isSuccess  — boolean
// order.isError    — boolean
// order.reset()    — clear state`
	},
	callOptionsType: {
		lang: 'typescript',
		code: `interface CallOptions {
  headers?: Record<string, string>;  // merged with client headers
  timeout?: number;                   // override client timeout (ms)
  signal?: AbortSignal;              // combined with client signal
  dedupe?: boolean;                  // per-call dedup override
}`
	},
	echoRust: {
		lang: 'rust',
		code: `#[derive(Deserialize, Serialize)]
pub struct EchoInput {
    pub message: String,
    pub uppercase: bool,
}

#[derive(Serialize)]
pub struct EchoOutput {
    pub original: String,
    pub transformed: String,
    pub length: u32,
}

#[rpc_mutation]
async fn echo(input: EchoInput) -> EchoOutput {
    let transformed = if input.uppercase {
        input.message.to_uppercase()
    } else {
        input.message.clone()
    };
    EchoOutput {
        length: transformed.len() as u32,
        original: input.message,
        transformed,
    }
}`
	},
	echoTs: {
		lang: 'typescript',
		code: `const echo = createMutation(rpc, "echo");

// Trigger explicitly:
echo.mutate({ message: "Hello", uppercase: true });

echo.data       // EchoOutput | undefined
echo.isLoading  // boolean (loading)
echo.isError    // boolean
echo.error      // RpcError | undefined`
	},
	callOptionsMutation: {
		lang: 'typescript',
		code: `// Mutation with input + options
await rpc.mutate('create_order', orderInput, {
  signal: abortController.signal,
  dedupe: false,
});`
	}
};

export const load: PageServerLoad = () => highlightBlocks(codeBlocks);
