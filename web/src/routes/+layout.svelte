<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { Toaster } from 'svelte-sonner';
	import Header from '$lib/components/Header.svelte';
	import { getMe } from '$lib/api';
	import { currentUser } from '$lib/stores';

	let { children } = $props();

	onMount(async () => {
		currentUser.set(await getMe());
	});
</script>

<Toaster richColors />
<div class="relative z-[1] flex min-h-screen flex-col">
	<Header />
	<main class="flex-1">
		{@render children()}
	</main>
</div>
