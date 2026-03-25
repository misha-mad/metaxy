<script lang="ts">
	import { rpc } from '$lib/client';
	import { createStream } from '$lib/rpc.svelte';
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import Code from '$lib/components/Code.svelte';
	import SectionHeading from '$lib/components/SectionHeading.svelte';
	import DemoCard from '$lib/components/DemoCard.svelte';
	import OutputBox from '$lib/components/OutputBox.svelte';

	let { data } = $props();

	// --- Countdown demo ---
	let countdownFrom = $state(5);
	let countdownDelay = $state(500);
	const countdown = createStream(rpc, 'countdown', () => ({
		from: countdownFrom,
		delay_ms: countdownDelay
	}));

	// --- Token stream demo ---
	let prompt = $state('The quick brown fox jumps over the lazy dog');
	const tokenStream = createStream(rpc, 'token_stream', () => ({
		prompt
	}));

	const assembledText = $derived(tokenStream.chunks.map((t: { text: string }) => t.text).join(''));
</script>

<svelte:head>
	<title>Streaming — metaxy</title>
</svelte:head>

<div class="max-w-3xl space-y-8">
	<PageHeader title="Streaming">
		<Code>#[rpc_stream]</Code> is a procedure type alongside queries and mutations that enables HTTP streaming
		responses via Server-Sent Events. It's built on Axum's streaming primitives and Vercel's streaming
		support.
	</PageHeader>

	<!-- How it works -->
	<div class="rounded-lg border border-border bg-bg-code px-4 py-3 text-sm text-text-muted">
		<strong class="text-text-primary">How it works:</strong> The handler receives typed input plus a
		<Code>StreamSender</Code> for emitting chunks. Each chunk is serialized as a
		<Code>data: &#123;json&#125;\n\n</Code> SSE event. The generated TypeScript client gets a
		<Code>stream()</Code> method returning an <Code>AsyncGenerator</Code>, and framework wrappers
		provide reactive state management via <Code>createStream</Code> / <Code>useStream</Code>.
	</div>

	<!-- Code examples -->
	<SectionHeading level="large">Examples</SectionHeading>

	<SectionHeading>Countdown — Structured Streaming</SectionHeading>
	<p class="text-text-muted text-sm">
		A stream handler with typed input and structured output chunks. Demonstrates
		<Code>timeout</Code> attribute and graceful client disconnection handling.
	</p>
	<CodeBlock html={data.highlighted['countdownRust']} />

	<SectionHeading>Token Stream — LLM-style Streaming</SectionHeading>
	<p class="text-text-muted text-sm">
		Simulates LLM token streaming by splitting input into words and streaming them back with
		realistic latency. Demonstrates the typical AI integration pattern.
	</p>
	<CodeBlock html={data.highlighted['tokenStreamRust']} />

	<!-- Client usage -->
	<SectionHeading>Client Usage</SectionHeading>
	<CodeBlock html={data.highlighted['clientUsage']} />
	<CodeBlock html={data.highlighted['svelteUsage']} />

	<!-- Supported attributes -->
	<SectionHeading>Supported Attributes</SectionHeading>
	<div class="overflow-x-auto">
		<table class="w-full text-sm text-left">
			<thead class="text-text-muted border-b border-border">
				<tr>
					<th class="py-2 pr-4">Attribute</th>
					<th class="py-2 pr-4">Stream</th>
					<th class="py-2">Notes</th>
				</tr>
			</thead>
			<tbody class="text-text-primary">
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>timeout</Code></td>
					<td class="py-2 pr-4 text-green-400">Yes</td>
					<td class="py-2 text-text-muted">Sends SSE error event on expiry</td>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>init</Code></td>
					<td class="py-2 pr-4 text-green-400">Yes</td>
					<td class="py-2 text-text-muted">Cold-start initialization, state injection</td>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>cache</Code></td>
					<td class="py-2 pr-4 text-red-400">No</td>
					<td class="py-2 text-text-muted">Streaming responses cannot be cached</td>
				</tr>
				<tr>
					<td class="py-2 pr-4 font-mono text-xs"><Code>idempotent</Code></td>
					<td class="py-2 pr-4 text-red-400">No</td>
					<td class="py-2 text-text-muted">Streams are inherently non-idempotent</td>
				</tr>
			</tbody>
		</table>
	</div>

	<SectionHeading>Client-Side Options</SectionHeading>
	<p class="text-text-muted text-sm mb-3">
		How <Code>RpcClientConfig</Code> and <Code>CallOptions</Code> behave for streams:
	</p>
	<div class="overflow-x-auto">
		<table class="w-full text-sm text-left">
			<thead class="text-text-muted border-b border-border">
				<tr>
					<th class="py-2 pr-4">Option</th>
					<th class="py-2 pr-4">Stream</th>
					<th class="py-2">Notes</th>
				</tr>
			</thead>
			<tbody class="text-text-primary">
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>callOptions.signal</Code></td>
					<td class="py-2 pr-4 text-green-400">Yes</td>
					<td class="py-2 text-text-muted"
						>Merged with internal controller via <Code>AbortSignal.any()</Code></td
					>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>callOptions.timeout</Code></td>
					<td class="py-2 pr-4 text-green-400">Yes</td>
					<td class="py-2 text-text-muted">Aborts the SSE connection after the given duration</td>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>onRequest</Code></td>
					<td class="py-2 pr-4 text-green-400">Yes</td>
					<td class="py-2 text-text-muted">Fires before the fetch</td>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>onError</Code></td>
					<td class="py-2 pr-4 text-green-400">Yes</td>
					<td class="py-2 text-text-muted">Fires on non-ok response or network error</td>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>onResponse</Code></td>
					<td class="py-2 pr-4 text-red-400">No</td>
					<td class="py-2 text-text-muted"
						>No single response body — use <Code>onChunk</Code> / <Code>onDone</Code> in framework wrappers</td
					>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>retry</Code></td>
					<td class="py-2 pr-4 text-red-400">No</td>
					<td class="py-2 text-text-muted">Stream restart is the application's responsibility</td>
				</tr>
				<tr class="border-b border-border/50">
					<td class="py-2 pr-4 font-mono text-xs"><Code>config.timeout</Code></td>
					<td class="py-2 pr-4 text-red-400">No</td>
					<td class="py-2 text-text-muted"
						>Server manages stream duration via <Code>#[rpc_stream(timeout)]</Code></td
					>
				</tr>
				<tr>
					<td class="py-2 pr-4 font-mono text-xs"><Code>dedupe</Code></td>
					<td class="py-2 pr-4 text-red-400">No</td>
					<td class="py-2 text-text-muted">Each stream call opens its own SSE connection</td>
				</tr>
			</tbody>
		</table>
	</div>

	<!-- Try it -->
	<SectionHeading level="large">Try it</SectionHeading>

	<!-- Countdown demo -->
	<DemoCard title="Countdown — Structured Streaming">
		<p class="text-text-muted text-sm mb-4">
			Streams countdown ticks with configurable start value and delay.
		</p>
		<div class="flex flex-wrap items-center gap-3 mb-3">
			<label class="text-sm text-text-muted">
				From:
				<input
					type="number"
					min="1"
					max="10"
					bind:value={countdownFrom}
					class="w-16 ml-1 rounded-md border border-border bg-bg-code px-2 py-1.5 text-sm text-text-primary outline-none focus:border-accent-ts"
				/>
			</label>
			<label class="text-sm text-text-muted">
				Delay:
				<select
					bind:value={countdownDelay}
					class="ml-1 rounded-md border border-border bg-bg-code px-2 py-1.5 text-sm text-text-primary outline-none focus:border-accent-ts"
				>
					<option value={200}>200ms</option>
					<option value={500}>500ms</option>
					<option value={1000}>1s</option>
				</select>
			</label>
			<button
				onclick={() => countdown.start()}
				disabled={countdown.isStreaming}
				class="rounded-md bg-accent-rust/20 text-accent-rust px-4 py-1.5 text-sm font-medium hover:bg-accent-rust/30 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
			>
				{countdown.isStreaming ? 'Streaming...' : 'Start'}
			</button>
			{#if countdown.isStreaming}
				<button
					onclick={() => countdown.stop()}
					class="rounded-md bg-red-500/20 text-red-400 px-4 py-1.5 text-sm font-medium hover:bg-red-500/30 transition-colors"
				>
					Stop
				</button>
			{/if}
		</div>
		{#if countdown.chunks.length > 0}
			<OutputBox mono>
				{#each countdown.chunks as tick, i (i)}
					<div>
						<span class="text-text-muted">[{tick.remaining}]</span>
						<span class={tick.remaining === 0 ? 'text-green-400' : 'text-text-primary'}
							>{tick.message}</span
						>
					</div>
				{/each}
			</OutputBox>
		{/if}
		{#if countdown.isDone && countdown.chunks.length > 0}
			<p class="text-green-400 text-xs mt-2">
				Stream completed ({countdown.chunks.length} chunks received)
			</p>
		{/if}
		{#if countdown.error}
			<OutputBox status="error">{countdown.error.message}</OutputBox>
		{/if}
	</DemoCard>

	<!-- Token stream demo -->
	<DemoCard title="Token Stream — LLM-style Streaming">
		<p class="text-text-muted text-sm mb-4">
			Simulates LLM token-by-token generation. Watch the text assemble in real time.
		</p>
		<div class="space-y-3 mb-3">
			<textarea
				bind:value={prompt}
				rows="2"
				class="w-full rounded-md border border-border bg-bg-code px-3 py-2 text-sm text-text-primary outline-none focus:border-accent-ts resize-none"
				placeholder="Enter a prompt..."
			></textarea>
			<div class="flex gap-3">
				<button
					onclick={() => tokenStream.start()}
					disabled={tokenStream.isStreaming}
					class="rounded-md bg-accent-ts/20 text-accent-ts px-4 py-1.5 text-sm font-medium hover:bg-accent-ts/30 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				>
					{tokenStream.isStreaming ? 'Generating...' : 'Generate'}
				</button>
				{#if tokenStream.isStreaming}
					<button
						onclick={() => tokenStream.stop()}
						class="rounded-md bg-red-500/20 text-red-400 px-4 py-1.5 text-sm font-medium hover:bg-red-500/30 transition-colors"
					>
						Stop
					</button>
				{/if}
			</div>
		</div>
		{#if tokenStream.chunks.length > 0}
			<OutputBox>
				<div class="text-text-primary">
					{assembledText}{#if tokenStream.isStreaming}<span
							class="inline-block w-2 h-4 bg-accent-ts/60 animate-pulse ml-0.5 align-text-bottom"
						></span>{/if}
				</div>
				<div class="text-text-muted text-xs mt-2">
					{tokenStream.chunks.length} tokens{#if tokenStream.isDone}
						&mdash; complete{/if}
				</div>
			</OutputBox>
		{/if}
		{#if tokenStream.error}
			<OutputBox status="error">{tokenStream.error.message}</OutputBox>
		{/if}
	</DemoCard>
</div>
