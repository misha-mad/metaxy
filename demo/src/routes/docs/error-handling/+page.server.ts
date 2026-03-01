import { highlightBlocks } from '$lib/highlight.server';
import type { PageServerLoad } from './$types';

const codeBlocks: Record<string, { code: string; lang: 'rust' | 'typescript' }> = {
	secretRust: {
		lang: 'rust',
		code: `#[rpc_query]
async fn secret(headers: Headers) -> Result<String, String> {
    let auth = headers.get("authorization")
        .and_then(|v| v.to_str().ok());

    if auth != Some("Bearer secret-token-123") {
        return Err("Unauthorized".into());
    }
    Ok("Access granted!".to_string())
}`
	},
	secretTs: {
		lang: 'typescript',
		code: `import { RpcError } from './rpc-client';

try {
  const result = await rpc.query('secret');
} catch (e) {
  if (e instanceof RpcError) {
    e.status   // HTTP status code
    e.message  // error message
    e.data     // parsed error body
  }
}`
	},
	onErrorCallback: {
		lang: 'typescript',
		code: `// Global error handler via RpcClientConfig
const rpc = createRpcClient({
  baseUrl: '/api',
  onError: (ctx) => {
    console.error(\`\${ctx.procedure} failed (attempt \${ctx.attempt})\`);
    if (ctx.willRetry) {
      console.log('Retrying...');
    }
  },
});`
	},
	frameworkError: {
		lang: 'typescript',
		code: `// Reactive error handling with createQuery / createMutation
const users = createQuery(rpc, 'list_users', () => page, {
  onError: (err) => {
    // err is RpcError
    if (err.status === 401) {
      redirectToLogin();
    }
  },
});

// Check error state in templates
// users.isError   — boolean
// users.error     — RpcError | undefined`
	},
	mutationError: {
		lang: 'typescript',
		code: `// Mutation error handling with mutateAsync
const create = createMutation(rpc, 'create_order');

try {
  const order = await create.mutateAsync(orderInput);
} catch (e) {
  if (e instanceof RpcError) {
    switch (e.status) {
      case 400: showValidationErrors(e.data); break;
      case 409: showConflictMessage(); break;
      default:  showGenericError(e.message);
    }
  }
}

// Or fire-and-forget with callback
const create2 = createMutation(rpc, 'create_order', {
  onError: (err) => toast.error(err.message),
});
create2.mutate(orderInput);`
	},
	timeoutError: {
		lang: 'typescript',
		code: `// Timeout and abort errors
const controller = new AbortController();

try {
  await rpc.query('slow_report', input, {
    timeout: 5000,
    signal: controller.signal,
  });
} catch (e) {
  if (e instanceof RpcError) {
    // Server returned an error status
  } else if (e instanceof DOMException && e.name === 'AbortError') {
    // Request was aborted (timeout or manual cancel)
  }
}`
	}
};

export const load: PageServerLoad = () => highlightBlocks(codeBlocks);
