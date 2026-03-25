<script lang="ts">
	import { createRpcClient } from '$lib/rpc-client';
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import Code from '$lib/components/Code.svelte';
	import SectionHeading from '$lib/components/SectionHeading.svelte';
	import DemoCard from '$lib/components/DemoCard.svelte';
	import Button from '$lib/components/Button.svelte';
	import OutputBox from '$lib/components/OutputBox.svelte';

	let { data } = $props();

	type HookEntry = { hook: 'onRequest' | 'onResponse' | 'onError'; detail: string; ts: number };
	let hookLog: HookEntry[] = $state([]);
	let loading = $state(false);

	async function runHooksDemo() {
		loading = true;
		hookLog = [];
		const startTs = Date.now();
		const log: HookEntry[] = [];

		const client = createRpcClient({
			baseUrl: '/api',
			onRequest: (ctx) => {
				log.push({
					hook: 'onRequest',
					detail: `procedure="${ctx.procedure}" url="${ctx.url}"`,
					ts: Date.now() - startTs
				});
				hookLog = [...log];
			},
			onResponse: (ctx) => {
				log.push({
					hook: 'onResponse',
					detail: `status=200 duration=${ctx.duration}ms data=${JSON.stringify(ctx.data).slice(0, 80)}`,
					ts: Date.now() - startTs
				});
				hookLog = [...log];
			},
			onError: (ctx) => {
				log.push({
					hook: 'onError',
					detail: `error="${ctx.error}"`,
					ts: Date.now() - startTs
				});
				hookLog = [...log];
			}
		});

		try {
			await client.query('time');
		} catch {
			// error logged via onError
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Lifecycle Hooks — metaxy</title>
</svelte:head>

<div class="max-w-3xl space-y-8">
	<PageHeader title="Lifecycle Hooks">
		Three hooks let you intercept the request lifecycle: before the fetch, after a successful
		response, and on failure.
	</PageHeader>

	<SectionHeading>Overview</SectionHeading>
	<CodeBlock html={data.highlighted['allHooks']} />

	<SectionHeading level="large">onRequest</SectionHeading>
	<p class="text-text-muted text-sm mb-2">
		Fires before the fetch. You can mutate <Code>ctx.headers</Code>
		to add or override headers dynamically. Runs again on every retry attempt.
	</p>
	<CodeBlock html={data.highlighted['onRequest']} />
	<CodeBlock html={data.highlighted['requestCtx']} />

	<SectionHeading level="large">onResponse</SectionHeading>
	<p class="text-text-muted text-sm mb-2">
		Fires after a successful response. Use it for logging, metrics, or cache warming.
	</p>
	<CodeBlock html={data.highlighted['onResponse']} />
	<CodeBlock html={data.highlighted['responseCtx']} />

	<SectionHeading level="large">onError</SectionHeading>
	<p class="text-text-muted text-sm mb-2">
		Fires on failure. Check <Code>ctx.willRetry</Code>
		to know if the client will retry, or use
		<Code>ctx.attempt</Code>
		to track retry progress.
	</p>
	<CodeBlock html={data.highlighted['onError']} />
	<CodeBlock html={data.highlighted['errorCtx']} />

	<!-- Streaming note -->
	<SectionHeading level="large">Streaming</SectionHeading>
	<p class="text-text-muted text-sm">
		For <Code>stream()</Code> calls, <Code>onRequest</Code> and <Code>onError</Code> fire as usual. <Code
			>onResponse</Code
		> is <strong>not called</strong> because a stream has no single response body — use <Code
			>onChunk</Code
		> / <Code>onDone</Code> in framework wrappers instead.
	</p>

	<!-- Try it -->
	<SectionHeading level="large">Try it</SectionHeading>
	<p class="text-text-muted text-sm">
		Calls an endpoint and logs every lifecycle hook as it fires.
	</p>

	<DemoCard>
		<div class="flex items-center gap-2 mb-4">
			<Button onclick={runHooksDemo} disabled={loading}>Run lifecycle demo</Button>
			{#if loading}
				<span class="text-sm text-text-muted">Loading...</span>
			{/if}
		</div>

		{#if hookLog.length > 0}
			<OutputBox mono={true}>
				{#each hookLog as entry, i (i)}
					<div class="flex gap-4 text-xs">
						<span class="text-text-faint w-12 text-right">{entry.ts}ms</span>
						{#if entry.hook === 'onRequest'}
							<span class="text-blue-400">onRequest</span>
						{:else if entry.hook === 'onResponse'}
							<span class="text-green-400">onResponse</span>
						{:else}
							<span class="text-red-400">onError</span>
						{/if}
						<span class="text-text-muted">{entry.detail}</span>
					</div>
				{/each}
			</OutputBox>
		{/if}
	</DemoCard>
</div>
