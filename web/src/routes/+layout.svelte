<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { Toaster } from 'svelte-sonner';
	import Header from '$lib/components/Header.svelte';
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
<div class="relative z-[1] flex min-h-screen flex-col">
	<Header />
	<main class="flex-1">
		{@render children()}
	</main>
</div>
