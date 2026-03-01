<script lang="ts">
	import { resolve } from '$app/paths';
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import FeatureRow from '$lib/components/FeatureRow.svelte';
	import Code from '$lib/components/Code.svelte';

	let { data } = $props();
</script>

<svelte:head>
	<title>metaxy</title>
	<meta
		name="description"
		content="End-to-end typesafe RPC between Rust lambdas on Vercel and any frontend"
	/>
</svelte:head>

<section
	class="relative flex flex-col items-center justify-center px-4 py-20 text-center overflow-hidden"
>
	<img
		src="/hero-left.png"
		alt=""
		class="pointer-events-none absolute left-0 top-1/2 -translate-y-1/2 w-1/3 object-contain invert opacity-70 hidden sm:block"
	/>
	<img
		src="/hero-right.png"
		alt=""
		class="pointer-events-none absolute right-0 top-1/2 -translate-y-1/2 w-1/3 object-contain invert opacity-70 hidden sm:block"
	/>
	<h1 class="relative text-4xl sm:text-5xl font-bold mb-2">metaxy</h1>
	<p class="text-sm text-text-faint mb-6">/mɛˈtæk.si/ · Greek: μεταξύ · <em>the in-between</em></p>

	<p class="text-lg sm:text-xl text-text-muted max-w-2xl mb-10 leading-relaxed">
		End-to-end typesafe RPC between<br />
		<strong class="text-accent-rust">Rust lambdas</strong>
		&amp;
		<strong class="text-accent-ts">TS frontend</strong>
	</p>

	<div class="flex gap-4 mb-16">
		<a
			href={resolve('/docs')}
			class="rounded-lg bg-accent-rust px-6 py-2.5 font-semibold text-white text-sm transition-opacity hover:opacity-85"
		>
			Get Started →
		</a>
		<a
			href="https://github.com/misha-mad/metaxy"
			class="rounded-lg border border-accent-ts px-6 py-2.5 font-semibold text-accent-ts text-sm transition-opacity hover:opacity-85"
		>
			GitHub
		</a>
	</div>
</section>

<section
	class="border-t border-b border-border grid grid-cols-1 sm:grid-cols-3 gap-px bg-border mb-16"
>
	<!-- 1: End-to-end type safety -->
	<FeatureRow title="End-to-end type safety">
		{#snippet code()}
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-3 flex-1">
				<CodeBlock html={data.highlighted['typeSafetyRust']} showToolbar={false} />
				<CodeBlock html={data.highlighted['typeSafetyTs']} showToolbar={false} />
			</div>
		{/snippet}
		Define a Rust struct once — the CLI generates matching TypeScript interfaces. Rename a field, and
		your frontend won't compile until it's updated. No more <Code>any</Code>, no runtime surprises.
	</FeatureRow>

	<!-- 2: Auto-generated client -->
	<FeatureRow title="Auto-generated client" reverse>
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['autoClient']} />{/snippet}
		A fully typed <Code>rpc.query()</Code> /
		<Code>rpc.mutate()</Code> client with autocomplete for every procedure. Input and output types are
		inferred from your Rust code.
	</FeatureRow>

	<!-- 3: Watch mode -->
	<FeatureRow title="Watch mode">
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['watchMode']} />{/snippet}
		Save a <Code>.rs</Code> file and types regenerate instantly. Runs alongside your dev server so the
		frontend always has the latest types without manual steps.
	</FeatureRow>

	<!-- 4: Macro-driven -->
	<FeatureRow title="Macro-driven" reverse>
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['macroDriven']} />{/snippet}
		Annotate with <Code>#[rpc_query]</Code> or
		<Code>#[rpc_mutation]</Code> and get CORS, input parsing, JSON serialization, error handling, and
		HTTP method validation — all generated at compile time.
	</FeatureRow>

	<!-- 5: Init & state injection -->
	<FeatureRow title="Init & state injection">
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['initState']} />{/snippet}
		Set up database pools, HTTP clients, or loggers once at cold start. The macro injects shared state
		as <Code>&T</Code> into your handler automatically.
	</FeatureRow>

	<!-- 6: Serde support -->
	<FeatureRow title="Serde support" reverse>
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['serde']} />{/snippet}
		<Code>rename_all</Code>,
		<Code>skip</Code>,
		<Code>flatten</Code>, and all four enum tagging strategies — the codegen reads your serde
		attributes and generates TypeScript that matches the actual JSON output.
	</FeatureRow>

	<!-- 7: Edge caching -->
	<FeatureRow title="Edge caching">
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['edgeCache']} />{/snippet}
		Add <Code>cache = "1h"</Code> and the macro generates <Code>Cache-Control</Code> headers. On Vercel,
		this enables CDN caching with zero infrastructure changes.
	</FeatureRow>

	<!-- 8: Vercel-native -->
	<FeatureRow title="Vercel-native" reverse>
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['vercelNative']} />{/snippet}
		Each <Code>.rs</Code> file in
		<Code>api/</Code>
		becomes a serverless function. No routing config, no server setup — just deploy with
		<Code>vercel</Code>.
	</FeatureRow>

	<!-- 9: 4 framework wrappers -->
	<FeatureRow title="4 framework wrappers">
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['frameworks']} />{/snippet}
		Opt-in reactive wrappers for Svelte 5, React, Vue 3, and SolidJS. Auto-refetching queries, mutation
		lifecycle callbacks — generated alongside the base client.
	</FeatureRow>

	<!-- 10: Rich client -->
	<FeatureRow title="Rich client" reverse>
		{#snippet code()}<CodeBlock showToolbar={false} html={data.highlighted['richClient']} />{/snippet}
		Retry with backoff, per-request timeout, AbortSignal support, request deduplication, lifecycle hooks,
		async headers — all configurable globally or per-call.
	</FeatureRow>
</section>
