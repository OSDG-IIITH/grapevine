<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { Toaster } from 'svelte-sonner';
	import { IconBrandGithub } from '@tabler/icons-svelte';
	import { page } from '$app/state';
	import { goto, afterNavigate } from '$app/navigation';
	import { base } from '$app/paths';
	import Header from '$lib/components/Header.svelte';
	import BottomNav from '$lib/components/BottomNav.svelte';
	import SearchModal from '$lib/components/SearchModal.svelte';
	import { getMe } from '$lib/api';
	import { currentUser, searchOpen } from '$lib/stores';

	let { children } = $props();

	// Routes that render regardless of auth state (no redirect loop).
	const PUBLIC_ROUTES = ['/login', '/verify', '/changelog'];

	let authChecked = $state(false);

	const routeId = $derived(page.route.id);
	const isPublicRoute = $derived(!!routeId && PUBLIC_ROUTES.includes(routeId));
	// Render the app once the auth check is done, except gated routes only
	// render when there's a verified user. Public routes always render.
	const canRender = $derived(
		isPublicRoute || (authChecked && !!$currentUser && $currentUser.verified)
	);

	// Single source of truth for redirects: logged-out -> /login, logged-in but
	// unverified -> /verify, and verified users bounced off the front door. Runs
	// after the initial /me check AND after every client-side navigation (not
	// just on mount), so deep links and back/forward into a gated route are
	// guarded too. Loop-free: each branch only redirects when the current route
	// isn't already the allowed one.
	function enforce() {
		if (!authChecked) return;
		const id = page.route.id;
		const u = $currentUser;
		if (!u) {
			if (id !== '/login') goto(base + '/login');
		} else if (!u.verified) {
			if (id !== '/verify') goto(base + '/verify');
		} else if (id === '/login' || id === '/verify') {
			goto(base + '/');
		}
	}

	afterNavigate(() => enforce());

	onMount(() => {
		getMe().then((u) => {
			currentUser.set(u);
			authChecked = true;
			enforce();
		});

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

<Toaster theme="dark" position="bottom-right" />

{#if isPublicRoute}
	<!-- Public front-door routes (/login, /verify): no app chrome. -->
	<main class="flex min-h-screen flex-col">
		{@render children()}
	</main>
{:else if canRender}
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
{/if}
