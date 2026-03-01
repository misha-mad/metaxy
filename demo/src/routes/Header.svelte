<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import github from '$lib/images/github.svg';
</script>

<header class="sticky top-0 z-50 border-b border-border bg-bg-header/80 backdrop-blur-md">
	<div class="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 sm:px-6">
		<a
			href={resolve('/')}
			class="flex items-center gap-2 text-lg font-bold text-text-primary hover:text-accent-rust transition-colors"
		>
			<span>metaxy</span>
		</a>

		<nav class="flex items-center gap-6">
			<a
				href={resolve('/')}
				class="text-sm font-medium transition-colors {page.url.pathname === '/'
					? 'text-accent-rust'
					: 'text-text-muted hover:text-text-primary'}"
			>
				Home
			</a>
			<a
				href={resolve('/docs')}
				class="text-sm font-medium transition-colors {page.url.pathname.startsWith('/docs')
					? 'text-accent-ts'
					: 'text-text-muted hover:text-text-primary'}"
			>
				Docs
			</a>
			<a
				href="https://github.com/misha-mad/metaxy"
				class="flex items-center text-text-muted hover:text-text-primary transition-colors"
				target="_blank"
				rel="noopener"
			>
				<img src={github} alt="GitHub" class="h-5 w-5 invert opacity-70 hover:opacity-100" />
			</a>
		</nav>
	</div>
</header>
