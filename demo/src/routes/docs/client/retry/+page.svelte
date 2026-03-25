<script lang="ts">
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import Code from '$lib/components/Code.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import SectionHeading from '$lib/components/SectionHeading.svelte';

	let { data } = $props();
</script>

<svelte:head>
	<title>Retry — metaxy</title>
</svelte:head>

<div class="max-w-3xl space-y-8">
	<PageHeader title="Retry">
		Queries are retried automatically on network errors or retryable HTTP status codes. Mutations
		are <strong>never</strong> retried unless explicitly marked as
		<a href="/docs/macros/idempotent" class="text-accent-ts hover:underline"
			><Code>idempotent</Code></a
		>.
	</PageHeader>

	<SectionHeading>RetryPolicy</SectionHeading>
	<CodeBlock html={data.highlighted['retryConfig']} />

	<p class="text-text-muted leading-relaxed text-sm">
		A request is retried when a network error occurs or the response status is in
		<Code>retryOn</Code>, up to
		<Code>attempts</Code> additional tries. On each retry the full
		<a href="/docs/client/hooks" class="text-accent-ts hover:underline"><Code>onRequest</Code></a> hook
		runs again, so dynamic headers (e.g. refreshed auth tokens) are re-evaluated.
	</p>

	<SectionHeading>Basic Usage</SectionHeading>
	<CodeBlock html={data.highlighted['retryBasic']} />

	<SectionHeading>Exponential Backoff</SectionHeading>
	<CodeBlock html={data.highlighted['retryExponential']} />

	<SectionHeading>Custom Retry Logic</SectionHeading>
	<CodeBlock html={data.highlighted['retryCustom']} />

	<SectionHeading>Streaming</SectionHeading>
	<p class="text-text-muted leading-relaxed text-sm">
		Retry is <strong>not supported</strong> for <Code>stream()</Code> calls. Streams open a long-lived
		SSE connection, and the server manages their lifetime via the
		<a href="/docs/macros/timeout" class="text-accent-ts hover:underline"><Code>timeout</Code></a>
		attribute. Restarting a stream after failure is the application's responsibility — use
		<Code>start()</Code> on the framework wrapper to begin a new stream.
	</p>

	<SectionHeading>Idempotent Mutations</SectionHeading>
	<p class="text-text-muted leading-relaxed text-sm">
		By default, mutations are never retried — even with a retry policy configured. To opt a mutation
		into retry, mark it as
		<a href="/docs/macros/idempotent" class="text-accent-ts hover:underline"
			><Code>idempotent</Code></a
		>
		in the Rust macro. This signals that repeated calls produce the same result.
	</p>
	<CodeBlock html={data.highlighted['retryIdempotent']} />
</div>
