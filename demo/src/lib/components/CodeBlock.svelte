<script lang="ts">
	let {
		html,
		large = false,
		showToolbar = true,
		showCopy = true
	}: {
		html: string;
		large?: boolean;
		showToolbar?: boolean;
		showCopy?: boolean;
	} = $props();

	const lang = $derived(html.match(/data-lang="([^"]+)"/)?.[1] ?? '');

	let copied = $state(false);

	function copyCode() {
		const tmp = document.createElement('div');
		tmp.innerHTML = html;
		const text = tmp.textContent ?? '';
		navigator.clipboard.writeText(text).then(() => {
			copied = true;
			setTimeout(() => (copied = false), 1500);
		});
	}
</script>

<div class="code-block" class:code-block-lg={large}>
	{#if showToolbar}
		<div
			class="code-block-toolbar flex items-center justify-between px-3 py-1.5 bg-[#141418] rounded-t-lg"
		>
			{#if lang}
				<span
					class="code-lang-badge rounded px-1.5 py-0.5 text-[10px] font-medium leading-none select-none"
					class:lang-rust={lang === 'rust'}
					class:lang-ts={lang === 'ts'}
					class:lang-sh={lang === 'sh'}
					class:lang-toml={lang === 'toml'}>{lang}</span
				>
			{:else}
				<span></span>
			{/if}
			{#if showCopy}
				<button
					class="w-5 h-5 flex items-center justify-center rounded transition-colors cursor-pointer {copied
						? 'text-white'
						: 'text-text-faint hover:text-text-muted'}"
					onclick={copyCode}
					title="Copy"
				>
					{#if copied}
						<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"
							><path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2.5"
								d="M5 13l4 4L19 7"
							/></svg
						>
					{:else}
						<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"
							><path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
							/></svg
						>
					{/if}
				</button>
			{/if}
		</div>
	{/if}
	<div class="relative">
		{#if !showToolbar && lang}
			<span
				class="code-lang-badge absolute top-[0.75rem] right-[0.75rem] rounded px-1.5 py-0.5 text-[10px] font-medium leading-none select-none z-10"
				class:lang-rust={lang === 'rust'}
				class:lang-ts={lang === 'ts'}
				class:lang-sh={lang === 'sh'}
				class:lang-toml={lang === 'toml'}>{lang}</span
			>
		{/if}
		<!-- eslint-disable-next-line svelte/no-at-html-tags -- Shiki pre-rendered HTML -->
		{@html html}
	</div>
</div>
