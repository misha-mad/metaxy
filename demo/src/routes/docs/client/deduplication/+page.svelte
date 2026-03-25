<script lang="ts">
	import { rpc } from '$lib/client';
	import { createRpcClient } from '$lib/rpc-client';
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import CollapsibleCode from '$lib/components/CollapsibleCode.svelte';
	import Code from '$lib/components/Code.svelte';
	import Button from '$lib/components/Button.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import SectionHeading from '$lib/components/SectionHeading.svelte';
	import DemoCard from '$lib/components/DemoCard.svelte';
	import OutputBox from '$lib/components/OutputBox.svelte';

	let { data } = $props();

	type DedupResult = { mode: string; calls: number; serverRequests: string; results: string[] };
	let dedupResult: DedupResult | undefined = $state();
	let loading = $state(false);

	async function fireCalls(dedupe: boolean) {
		loading = true;
		dedupResult = undefined;
		const client = dedupe ? rpc : createRpcClient({ baseUrl: '/api', dedupe: false });

		const N = 5;
		const promises = Array.from({ length: N }, () => client.query('dedup_demo'));
		const results = await Promise.all(promises);

		const requestNumbers = results.map((r) => r.request_number);
		const unique = new Set(requestNumbers);

		dedupResult = {
			mode: dedupe ? 'dedupe: true (default)' : 'dedupe: false',
			calls: N,
			serverRequests: `${unique.size} unique server request(s)`,
			results: results.map((r, i) => `call ${i + 1} → request #${r.request_number}`)
		};
		loading = false;
	}
</script>

<svelte:head>
	<title>Request Deduplication — metaxy</title>
</svelte:head>

<div class="max-w-3xl space-y-8">
	<PageHeader title="Request Deduplication">
		Identical in-flight queries are automatically deduplicated — only one HTTP request is made and
		all callers share the same promise.
	</PageHeader>

	<SectionHeading>How It Works</SectionHeading>
	<p class="text-text-muted leading-relaxed text-sm">
		When multiple callers issue the same query with the same input concurrently, only one HTTP
		request is made. Requests are matched by procedure name + serialized input. Subsequent callers
		receive the same in-flight promise.
	</p>
	<CodeBlock html={data.highlighted['dedupExample']} />

	<SectionHeading>Disabling Deduplication</SectionHeading>
	<p class="text-text-muted leading-relaxed text-sm">
		Dedup is controlled at two levels — client config and per-call. Per-call takes precedence.
	</p>
	<CodeBlock html={data.highlighted['dedupDisableGlobal']} />
	<CodeBlock html={data.highlighted['dedupDisablePerCall']} />

	<p class="text-text-muted leading-relaxed text-sm">
		Mutations and streams are never deduplicated. Each stream call opens its own SSE connection.
		Each per-caller <Code>AbortSignal</Code> is wrapped independently — aborting one caller does not affect
		others sharing the same in-flight promise.
	</p>

	<!-- Try it -->
	<SectionHeading level="large">Try it</SectionHeading>
	<p class="text-text-muted text-sm">
		Fire 5 identical queries concurrently. With dedup enabled, only 1 HTTP request is made. The
		server sleeps 500ms and increments a counter on each real request.
	</p>

	<DemoCard>
		<div class="flex items-center gap-2 mb-4 flex-wrap">
			<Button onclick={() => fireCalls(true)} disabled={loading}>5 calls (dedupe on)</Button>
			<button
				onclick={() => fireCalls(false)}
				disabled={loading}
				class="rounded-md bg-red-600 px-3 py-1.5 text-sm font-medium text-white transition-opacity hover:opacity-85 disabled:opacity-50"
				>5 calls (dedupe off)</button
			>
			{#if loading}
				<span class="text-sm text-text-muted">Loading...</span>
			{/if}
		</div>

		{#if dedupResult}
			<OutputBox mono>
				<div class="text-text-faint">{dedupResult.mode}</div>
				<div class="text-text-muted">{dedupResult.calls} calls → {dedupResult.serverRequests}</div>
				{#each dedupResult.results as line, i (i)}
					<div class="text-accent-ts">{line}</div>
				{/each}
			</OutputBox>
		{/if}

		<CollapsibleCode html={data.highlighted['dedupDemoRust']} />
	</DemoCard>
</div>
