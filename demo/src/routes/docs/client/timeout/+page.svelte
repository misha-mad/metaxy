<script lang="ts">
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

	type LogEntry = { label: string; status: 'ok' | 'abort' | 'error'; detail: string };
	let callLog: LogEntry[] = $state([]);
	let loading = $state(false);
	let abortController: AbortController | undefined = $state();

	async function runClientTimeout(serverDelayMs: number, clientTimeoutMs: number) {
		loading = true;
		const client = createRpcClient({ baseUrl: '/api' });
		try {
			const res = await client.query(
				'timeout_demo',
				{ sleep_ms: serverDelayMs },
				{
					timeout: clientTimeoutMs
				}
			);
			callLog = [
				...callLog.slice(-4),
				{
					label: `delay=${serverDelayMs}ms, timeout=${clientTimeoutMs}ms`,
					status: 'ok',
					detail: `200 OK — actual ${res.actual_ms}ms`
				}
			];
		} catch (e: unknown) {
			const isAbort = e instanceof DOMException && e.name === 'AbortError';
			callLog = [
				...callLog.slice(-4),
				{
					label: `delay=${serverDelayMs}ms, timeout=${clientTimeoutMs}ms`,
					status: isAbort ? 'abort' : 'error',
					detail: isAbort ? 'AbortError: client timeout exceeded' : String(e)
				}
			];
		} finally {
			loading = false;
		}
	}

	async function runManualAbort() {
		loading = true;
		abortController = new AbortController();
		const client = createRpcClient({ baseUrl: '/api' });
		try {
			const res = await client.query(
				'timeout_demo',
				{ sleep_ms: 10000 },
				{
					signal: abortController.signal
				}
			);
			callLog = [
				...callLog.slice(-4),
				{
					label: 'manual abort (10s delay)',
					status: 'ok',
					detail: `200 OK — actual ${res.actual_ms}ms`
				}
			];
		} catch (e: unknown) {
			const isAbort = e instanceof DOMException && e.name === 'AbortError';
			callLog = [
				...callLog.slice(-4),
				{
					label: 'manual abort (10s delay)',
					status: isAbort ? 'abort' : 'error',
					detail: isAbort ? 'AbortError: manually cancelled' : String(e)
				}
			];
		} finally {
			loading = false;
			abortController = undefined;
		}
	}
</script>

<svelte:head>
	<title>Timeout &amp; Abort — metaxy</title>
</svelte:head>

<div class="max-w-3xl space-y-8">
	<PageHeader title="Timeout & Abort">
		Set a global timeout for all requests, pass an <Code>AbortSignal</Code>
		for manual cancellation, or combine both. Timeouts and aborts throw a
		<Code>DOMException</Code> with
		<Code>name: 'AbortError'</Code>, not
		<Code>RpcError</Code>.
	</PageHeader>

	<SectionHeading>Client Timeout</SectionHeading>
	<CodeBlock html={data.highlighted['clientTimeout']} />

	<SectionHeading>Per-Call Timeout</SectionHeading>
	<CodeBlock html={data.highlighted['perCallTimeout']} />

	<SectionHeading>AbortSignal</SectionHeading>
	<p class="text-text-muted text-sm mb-2">
		A client-level signal aborts all in-flight requests when fired.
	</p>
	<CodeBlock html={data.highlighted['abortSignal']} />

	<SectionHeading>Per-Call Signal</SectionHeading>
	<CodeBlock html={data.highlighted['perCallSignal']} />

	<SectionHeading>Combined Signals</SectionHeading>
	<p class="text-text-muted text-sm mb-2">
		When both client and per-call signals are provided, they are combined — the request aborts when
		either signal fires.
	</p>
	<CodeBlock html={data.highlighted['combinedSignals']} />

	<SectionHeading>Streaming</SectionHeading>
	<p class="text-text-muted text-sm mb-2">
		For <Code>stream()</Code> calls, <Code>callOptions.timeout</Code> and
		<Code>callOptions.signal</Code> work normally — they abort the SSE connection. However, the global
		<Code>config.timeout</Code> and per-procedure timeouts from
		<Code>PROCEDURE_TIMEOUTS</Code> are <strong>not applied</strong> to streams. The server manages
		stream duration via the
		<a href="/docs/macros/timeout" class="text-accent-ts hover:underline"
			><Code>#[rpc_stream(timeout)]</Code></a
		> attribute instead.
	</p>

	<SectionHeading>Error Handling</SectionHeading>
	<p class="text-text-muted text-sm mb-2">
		Timeouts and manual aborts throw a <Code>DOMException</Code>, not an <Code>RpcError</Code>.
	</p>
	<CodeBlock html={data.highlighted['errorHandling']} />

	<!-- Try it -->
	<SectionHeading level="large">Try it</SectionHeading>
	<p class="text-text-muted text-sm">
		The server sleeps for the requested duration. Client-side timeout fires an
		<Code>AbortError</Code> before the server responds.
	</p>

	<DemoCard title="Client Timeout">
		<p class="text-text-muted text-xs mb-3">
			Server delay 1s, client timeout 2s (OK) vs server delay 1s, client timeout 300ms (abort).
		</p>
		<div class="flex items-center gap-2 mb-4 flex-wrap">
			<Button onclick={() => runClientTimeout(1000, 2000)} disabled={loading}
				>1s delay, 2s timeout</Button
			>
			<button
				onclick={() => runClientTimeout(1000, 300)}
				disabled={loading}
				class="rounded-md bg-red-600 px-3 py-1.5 text-sm font-medium text-white transition-opacity hover:opacity-85 disabled:opacity-50"
				>1s delay, 300ms timeout</button
			>
		</div>

		<h3 class="text-lg font-semibold mb-1">Manual Abort</h3>
		<p class="text-text-muted text-xs mb-3">
			Start a 10s request, then cancel it with AbortController.
		</p>
		<div class="flex items-center gap-2 mb-4">
			{#if abortController}
				<button
					onclick={() => abortController?.abort()}
					class="rounded-md bg-red-600 px-3 py-1.5 text-sm font-medium text-white transition-opacity hover:opacity-85"
					>Abort now</button
				>
				<span class="text-sm text-text-muted">Request in flight...</span>
			{:else}
				<Button onclick={runManualAbort} disabled={loading}>Start 10s request</Button>
			{/if}
		</div>

		{#if callLog.length > 0}
			<OutputBox mono>
				{#each callLog as entry, i (i)}
					<div class="flex gap-4">
						<span class="text-text-faint">#{i + 1}</span>
						<span class="text-text-muted">{entry.label}</span>
						{#if entry.status === 'ok'}
							<span class="text-green-400">{entry.detail}</span>
						{:else if entry.status === 'abort'}
							<span class="text-red-400">{entry.detail}</span>
						{:else}
							<span class="text-yellow-400">{entry.detail}</span>
						{/if}
					</div>
				{/each}
			</OutputBox>
		{/if}

		<CollapsibleCode html={data.highlighted['timeoutDemoRust']} />
	</DemoCard>
</div>
