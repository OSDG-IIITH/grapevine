<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { Toaster } from 'svelte-sonner';
	import { IconBrandGithub } from '@tabler/icons-svelte';
	import Header from '$lib/components/Header.svelte';
	import BottomNav from '$lib/components/BottomNav.svelte';
	import SearchModal from '$lib/components/SearchModal.svelte';
	import { getMe } from '$lib/api';
	import { currentUser, searchOpen } from '$lib/stores';

	let { children } = $props();

	onMount(() => {
		getMe().then((u) => currentUser.set(u));

		function onkeydown(e: KeyboardEvent) {
			if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
				e.preventDefault();
				searchOpen.update((v) => !v);
			}
		}
		window.addEventListener('keydown', onkeydown);
		return () => window.removeEventListener('keydown', onkeydown);
	});
</script>

<Toaster richColors />
<SearchModal />
<BottomNav />
<div class="relative z-[1] flex min-h-screen flex-col">
	<Header />
	<main class="flex flex-1 min-h-0 flex-col">
		{@render children()}
	</main>
	<footer
		class="hidden md:grid grid-cols-3 items-center border-t border-[var(--border)] px-8 py-6 text-[13px] text-[var(--fg-3)]"
	>
		<div></div>
		<div class="text-center">
			made with &hearts; by
			<a
				href="https://osdg.iiit.ac.in"
				target="_blank"
				rel="noopener noreferrer"
				class="text-[var(--fg-2)] hover:text-[var(--fg)] transition-colors"
			>
				osdg
			</a>
		</div>
		<div class="flex justify-end">
			<a
				href="https://github.com/OSDG-IIITH/grapevine"
				target="_blank"
				rel="noopener noreferrer"
				aria-label="github"
				class="text-[var(--fg-2)] hover:text-[var(--fg)] transition-colors"
			>
				<IconBrandGithub size={16} stroke={1.6} aria-hidden="true" />
			</a>
		</div>
	</footer>
</div>
