<script lang="ts">
	import { page } from '$app/stores';
	import { currentUser, searchOpen } from '$lib/stores';
	import { env } from '$env/dynamic/public';

	const nav = [
		{ id: 'courses', label: 'Courses', href: '/courses' },
		{ id: 'faculty', label: 'Faculty', href: '/faculty' },
		{ id: 'labs', label: 'Labs', href: '/labs' },
		{ id: 'review', label: 'Review', href: '/review' }
	];

	function isactive(href: string) {
		return $page.url.pathname === href || $page.url.pathname.startsWith(href + '/');
	}

	const initials = $derived(
		$currentUser?.display_name
			.split(' ')
			.map((n) => n[0])
			.slice(0, 2)
			.join('')
			.toUpperCase() ?? ''
	);

	const ishome = $derived($page.url.pathname === '/');
</script>

<header
	class="sticky top-0 z-50 grid items-center border-b border-[var(--border)] px-7 py-[14px] backdrop-blur-[14px]"
	style="grid-template-columns: 1fr auto 1fr; background: rgba(18,24,20,0.78); backdrop-filter: blur(14px) saturate(120%);"
>
	<div>
		<a
			href="/"
			class="inline-flex items-center gap-2 text-[22px] tracking-[-0.01em] text-[var(--fg)]"
			style="font-family: var(--serif);"
		>
			grapevine
		</a>
	</div>

	<nav class="flex gap-1">
		{#each nav as n (n.id)}
			<a
				href={n.href}
				class="rounded-md px-[14px] py-[6px] text-[13px] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)] {isactive(n.href) ? 'text-[var(--fg)]' : 'text-[var(--fg-2)]'}"
			>
				{n.label}
			</a>
		{/each}
	</nav>

	<div class="flex items-center justify-end gap-3">
		<button
			type="button"
			aria-label="Search"
			aria-hidden={ishome}
			disabled={ishome}
			onclick={() => searchOpen.set(true)}
			class="flex w-[320px] items-center gap-[10px] whitespace-nowrap rounded-lg border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px] text-[13px] text-[var(--fg-3)] transition-[border-color,color] duration-[120ms] hover:border-[var(--border-2)] hover:text-[var(--fg-2)] {ishome ? 'invisible pointer-events-none' : ''}"
		>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0">
				<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
			</svg>
			<span class="overflow-hidden text-ellipsis whitespace-nowrap">Search courses, labs, faculty…</span>
			<span
				class="ml-auto shrink-0 whitespace-nowrap rounded border border-[var(--border-2)] bg-[var(--bg-3)] px-[6px] py-[2px] text-[11px] text-[var(--fg-3)]"
				style="font-family: var(--mono);"
			>
				Ctrl K
			</span>
		</button>

		{#if $currentUser}
			<div
				class="inline-flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-full border border-[var(--border-2)] text-[11px] font-medium tracking-[0.04em] text-[var(--fg)]"
				style="background: linear-gradient(135deg, #2f4a33, #1a221d);"
				title={$currentUser.display_name}
			>
				{initials}
			</div>
		{:else}
			<a
				href="{env.PUBLIC_API_URL}/auth/login"
				class="inline-flex items-center rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[6px] text-[13px] text-[var(--fg-2)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
			>
				Login
			</a>
		{/if}
	</div>
</header>
