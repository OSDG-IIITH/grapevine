<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { currentUser, searchOpen } from '$lib/stores';
	import { PUBLIC_API_URL } from '$env/static/public';
	import { IconSearch } from '@tabler/icons-svelte';

	const nav = [
		{ id: 'courses', label: 'Courses', href: '/courses' },
		{ id: 'faculty', label: 'Faculty', href: '/faculty' },
		{ id: 'labs', label: 'Labs', href: '/labs' },
		{ id: 'review', label: 'Review', href: '/review' }
	];

	function isactive(href: string) {
		return $page.route.id === href || $page.route.id?.startsWith(href + '/');
	}

	const initials = $derived(
		$currentUser?.display_name
			.split(' ')
			.map((n) => n[0])
			.slice(0, 2)
			.join('')
			.toUpperCase() ?? ''
	);

	const ishome = $derived($page.route.id === '/');

	let dropopen = $state(false);
	let dropref = $state<HTMLDivElement | null>(null);

	$effect(() => {
		if (!dropopen) return;
		function handle(e: MouseEvent) {
			if (!dropref?.contains(e.target as Node)) dropopen = false;
		}
		document.addEventListener('click', handle);
		return () => document.removeEventListener('click', handle);
	});

	async function logout() {
		await fetch(`${PUBLIC_API_URL || '/api'}/auth/logout`, { method: 'POST', credentials: 'include' });
		currentUser.set(null);
		goto(base + '/');
	}
</script>

<header
	class="sticky top-0 z-50 flex items-center border-b border-[var(--border)] px-7 py-[14px]"
	style="background: rgba(18,24,20,0.78); backdrop-filter: blur(14px) saturate(120%);"
>
	<div>
		<a
			href="{base}/"
			class="inline-flex items-center gap-2 text-[22px] tracking-[-0.01em] text-[var(--fg)]"
			style="font-family: var(--serif);"
		>
			grapevine
		</a>
	</div>

	<nav class="absolute left-1/2 hidden -translate-x-1/2 gap-1 md:flex">
		{#each nav as n (n.id)}
			<a
				href={base + n.href}
				class="rounded-md px-[14px] py-[6px] text-[13px] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)] {isactive(n.href) ? 'text-[var(--fg)]' : 'text-[var(--fg-2)]'}"
			>
				{n.label}
			</a>
		{/each}
	</nav>

	<div class="ml-auto flex items-center gap-3">
		<button
			type="button"
			aria-label="Search"
			aria-hidden={ishome}
			disabled={ishome}
			onclick={() => searchOpen.set(true)}
			class="hidden w-[320px] items-center gap-[10px] whitespace-nowrap rounded-lg border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px] text-[13px] text-[var(--fg-3)] transition-[border-color,color] duration-[120ms] hover:border-[var(--border-2)] hover:text-[var(--fg-2)] md:flex {ishome ? 'invisible pointer-events-none' : ''}"
		>
			<IconSearch size={14} stroke={2} class="shrink-0" aria-hidden />
			<span class="overflow-hidden text-ellipsis whitespace-nowrap">Search courses, labs, faculty…</span>
			<span
				class="ml-auto shrink-0 whitespace-nowrap rounded border border-[var(--border-2)] bg-[var(--bg-3)] px-[6px] py-[2px] text-[11px] text-[var(--fg-3)]"
				style="font-family: var(--mono);"
			>
				Ctrl K
			</span>
		</button>

		{#if !ishome}
			<button
				type="button"
				aria-label="Search"
				onclick={() => searchOpen.set(true)}
				class="flex h-[30px] w-[30px] items-center justify-center rounded-lg border border-[var(--border)] text-[var(--fg-3)] transition-colors duration-[120ms] hover:border-[var(--border-2)] hover:text-[var(--fg)] md:hidden"
			>
				<IconSearch size={15} stroke={1.8} aria-hidden />
			</button>
		{/if}

		{#if $currentUser}
			<div class="relative" bind:this={dropref}>
				<button
					type="button"
					aria-label="User menu"
					onclick={() => (dropopen = !dropopen)}
					class="inline-flex h-[30px] w-[30px] shrink-0 cursor-pointer items-center justify-center rounded-full border border-[var(--border-2)] text-[11px] font-medium tracking-[0.04em] text-[var(--fg)]"
					style="background: linear-gradient(135deg, #2f4a33, #1a221d);"
					title={$currentUser.display_name}
				>
					{initials}
				</button>

				{#if dropopen}
					<div
						class="absolute right-0 top-[calc(100%+8px)] z-[100] min-w-[180px] overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--bg-2)] shadow-lg"
					>
						<div class="px-3 py-[9px] text-[12px] text-[var(--fg-3)]">{$currentUser.display_name}</div>
						<div class="border-t border-[var(--border)]"></div>
						<a
							href="{base}/profile"
							onclick={() => (dropopen = false)}
							class="flex w-full items-center px-3 py-[9px] text-[13px] text-[var(--fg-2)] transition-colors duration-[100ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
						>
							Profile
						</a>
						<button
							type="button"
							onclick={logout}
							class="flex w-full items-center px-3 py-[9px] text-[13px] text-[var(--fg-2)] transition-colors duration-[100ms] hover:bg-[var(--bg-3)] hover:text-[var(--danger)]"
						>
							Logout
						</button>
					</div>
				{/if}
			</div>
		{:else}
			<a
				href="{PUBLIC_API_URL || '/api'}/auth/login"
				class="inline-flex items-center rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[6px] text-[13px] text-[var(--fg-2)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
			>
				Login
			</a>
		{/if}
	</div>
</header>
