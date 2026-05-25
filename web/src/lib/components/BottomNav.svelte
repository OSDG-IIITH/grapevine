<script lang="ts">
	import { page } from '$app/stores';
	import { IconBook, IconUser, IconFlask, IconPencil } from '@tabler/icons-svelte';

	const nav = [
		{ id: 'courses', label: 'Courses', href: '/courses', icon: IconBook },
		{ id: 'faculty', label: 'Faculty', href: '/faculty', icon: IconUser },
		{ id: 'labs', label: 'Labs', href: '/labs', icon: IconFlask },
		{ id: 'review', label: 'Review', href: '/review', icon: IconPencil }
	];

	function isactive(href: string) {
		return $page.url.pathname === href || $page.url.pathname.startsWith(href + '/');
	}
</script>

<nav
	class="fixed bottom-4 left-1/2 z-50 flex -translate-x-1/2 items-center gap-1 rounded-2xl border border-[var(--border)] px-2 py-2 md:hidden"
	style="background: rgba(18,24,20,0.92); backdrop-filter: blur(16px) saturate(120%); box-shadow: 0 4px 24px rgba(0,0,0,0.45);"
>
	{#each nav as n (n.id)}
		<a
			href={n.href}
			aria-label={n.label}
			class="flex h-11 w-11 items-center justify-center rounded-[10px] transition-colors duration-[120ms] {isactive(n.href) ? 'bg-[var(--bg-3)] text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
		>
			<n.icon size={18} stroke={1.7} aria-hidden />
		</a>
	{/each}
</nav>
