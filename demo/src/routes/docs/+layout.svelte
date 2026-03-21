<script lang="ts">
	import { base } from '$app/paths';
	import { page } from '$app/state';
	import { slide } from 'svelte/transition';

	let { children } = $props();

	const nav = [
		{ label: 'Getting Started', href: '/docs/getting-started' },
		{
			label: 'Procedures',
			children: [
				{ label: 'Queries', href: '/docs/procedures/queries' },
				{ label: 'Mutations', href: '/docs/procedures/mutations' },
				{ label: 'Streaming', href: '/docs/procedures/streaming' }
			]
		},
		{
			label: 'Macro Attributes',
			children: [
				{ label: 'Overview', href: '/docs/macros/attributes' },
				{ label: 'cache', href: '/docs/macros/cache' },
				{ label: 'stale', href: '/docs/macros/stale' },
				{ label: 'init', href: '/docs/macros/init' },
				{ label: 'timeout', href: '/docs/macros/timeout' },
				{ label: 'idempotent', href: '/docs/macros/idempotent' }
			]
		},
		{
			label: 'Codegen',
			children: [
				{ label: 'Type Mappings', href: '/docs/codegen/type-mappings' },
				{ label: 'Serde Support', href: '/docs/codegen/serde' },
				{ label: 'Generics', href: '/docs/codegen/generics' },
				{ label: 'Doc Comments', href: '/docs/codegen/doc-comments' },
				{ label: 'Field Naming', href: '/docs/codegen/field-naming' },
				{ label: 'Branded Newtypes', href: '/docs/codegen/branded-newtypes' },
				{ label: 'BigInt Mapping', href: '/docs/codegen/bigint' },
				{ label: 'Type Overrides', href: '/docs/codegen/type-overrides' }
			]
		},
		{
			label: 'Client',
			children: [
				{ label: 'Config', href: '/docs/client/config' },
				{ label: 'Headers', href: '/docs/client/headers' },
				{ label: 'Timeout & Abort', href: '/docs/client/timeout' },
				{ label: 'Lifecycle Hooks', href: '/docs/client/hooks' },
				{ label: 'Retry', href: '/docs/client/retry' },
				{ label: 'Deduplication', href: '/docs/client/deduplication' },
				{ label: 'Custom Fetch', href: '/docs/client/fetch' },
				{ label: 'Serialization', href: '/docs/client/serialization' }
			]
		},
		{
			label: 'Frameworks',
			children: [
				{ label: 'Svelte 5', href: '/docs/frameworks/svelte' },
				{ label: 'React', href: '/docs/frameworks/react' },
				{ label: 'Vue 3', href: '/docs/frameworks/vue' },
				{ label: 'SolidJS', href: '/docs/frameworks/solid' }
			]
		},
		{ label: 'Error Handling', href: '/docs/error-handling' },
		{
			label: 'CLI',
			children: [
				{ label: 'Overview', href: '/docs/cli/commands' },
				{ label: 'generate', href: '/docs/cli/generate' },
				{ label: 'scan', href: '/docs/cli/scan' },
				{ label: 'watch', href: '/docs/cli/watch' }
			]
		},
		{ label: 'Config File', href: '/docs/config-file' }
	] as const;

	type NavEntry = (typeof nav)[number];
	type NavGroup = Extract<NavEntry, { children: readonly unknown[] }>;

	function isGroup(entry: NavEntry): entry is NavGroup {
		return 'children' in entry;
	}

	function isActive(href: string): boolean {
		return page.url.pathname === href;
	}

	// Alternating color on each navigation click
	let clickCount = $state(0);
	const activeColor = $derived(clickCount % 2 === 0 ? 'rust' : 'ts');

	function hasActiveChild(group: NavGroup): boolean {
		return group.children.some((child) => isActive(child.href));
	}

	let toggleOverrides: Record<string, boolean> = $state({});

	function isGroupOpen(group: NavGroup): boolean {
		if (group.label in toggleOverrides) return toggleOverrides[group.label];
		return hasActiveChild(group);
	}

	function toggleGroup(group: NavGroup) {
		toggleOverrides[group.label] = !isGroupOpen(group);
	}

	let sidebarOpen = $state(false);

	$effect(() => {
		document.body.style.overflow = sidebarOpen ? 'hidden' : '';
		return () => {
			document.body.style.overflow = '';
		};
	});
</script>

<div class="mx-auto flex max-w-7xl">
	<!-- Mobile toggle -->
	<button
		class="fixed bottom-4 left-4 z-60 flex h-10 w-10 items-center justify-center rounded-lg bg-accent-rust text-white shadow-lg lg:hidden"
		onclick={() => (sidebarOpen = !sidebarOpen)}
	>
		{sidebarOpen ? '✕' : '☰'}
	</button>

	<!-- Sidebar -->
	<aside
		class="fixed top-0 bottom-0 left-0 z-40 w-60 overflow-y-auto border-r border-border bg-bg-sidebar pt-18 pb-16 px-4 transition-transform lg:translate-x-0 {sidebarOpen
			? 'translate-x-0'
			: '-translate-x-full'}"
	>
		<nav class="flex flex-col gap-0.5">
			{#each nav as entry (entry.label)}
				{#if isGroup(entry)}
					<button
						class="flex w-full items-center justify-between rounded-md px-3 py-1.5 text-sm font-medium text-text-muted transition-colors hover:bg-bg-soft hover:text-text-primary"
						onclick={() => toggleGroup(entry)}
					>
						{entry.label}
						<span
							class="text-[10px] text-text-faint transition-transform {isGroupOpen(entry)
								? 'rotate-90'
								: ''}">▶</span
						>
					</button>
					{#if isGroupOpen(entry)}
						<div
							transition:slide={{ duration: 150 }}
							class="ml-2 flex flex-col gap-0.5 border-l border-border pl-2"
						>
							{#each entry.children as child (child.href)}
								<a
									href={base + child.href}
									class="flex items-center gap-2 rounded-md px-3 py-1 text-sm transition-colors {isActive(
										child.href
									)
										? `bg-bg-soft text-text-primary border-l-2 -ml-0.5 pl-2.5 ${activeColor === 'rust' ? 'border-accent-rust' : 'border-accent-ts'}`
										: 'text-text-muted hover:bg-bg-soft hover:text-text-primary'}"
									onclick={() => {
										clickCount++;
										sidebarOpen = false;
									}}
								>
									{child.label}
									{#if 'badge' in child}
										<span class="rounded-full bg-accent-rust/20 text-accent-rust text-[10px] px-1.5"
											>{child.badge}</span
										>
									{/if}
								</a>
							{/each}
						</div>
					{/if}
				{:else}
					<a
						href={base + entry.href}
						class="rounded-md px-3 py-1.5 text-sm transition-colors {isActive(entry.href)
							? `bg-bg-soft text-text-primary border-l-2 -ml-0.5 pl-2.5 font-medium ${activeColor === 'rust' ? 'border-accent-rust' : 'border-accent-ts'}`
							: 'text-text-muted hover:bg-bg-soft hover:text-text-primary'}"
						onclick={() => {
							clickCount++;
							sidebarOpen = false;
						}}
					>
						{entry.label}
					</a>
				{/if}
			{/each}
		</nav>
	</aside>

	<!-- Backdrop for mobile -->
	{#if sidebarOpen}
		<button
			class="fixed inset-0 z-30 bg-black/50 lg:hidden"
			onclick={() => (sidebarOpen = false)}
			aria-label="Close sidebar"
		></button>
	{/if}

	<!-- Content -->
	<div class="min-w-0 flex-1 px-4 py-8 sm:px-8 lg:ml-60 lg:pl-8">
		{@render children()}
	</div>
</div>
