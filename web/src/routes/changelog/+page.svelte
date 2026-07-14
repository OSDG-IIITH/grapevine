<script lang="ts">
	import { onMount } from 'svelte';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { currentUser } from '$lib/stores';
	import { getMe } from '$lib/api';
	import { PUBLIC_API_URL } from '$env/static/public';
	import { IconBrandGithub } from '@tabler/icons-svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';

	let { data } = $props();

	function md(s: string): string {
		return s
			.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/\*(.+?)\*/g, '<em>$1</em>')
			.replace(/`(.+?)`/g, '<code>$1</code>');
	}

	let dropopen = $state(false);
	let dropref = $state<HTMLDivElement | null>(null);
	let settingsopen = $state(false);

	const initials = $derived(
		$currentUser?.display_name
			.split(' ')
			.map((n) => n[0])
			.slice(0, 2)
			.join('')
			.toUpperCase() ?? ''
	);

	$effect(() => {
		if (!dropopen) return;
		function handle(e: MouseEvent) {
			if (!dropref?.contains(e.target as Node)) dropopen = false;
		}
		document.addEventListener('click', handle);
		return () => document.removeEventListener('click', handle);
	});

	async function logout() {
		try {
			const res = await fetch(`${PUBLIC_API_URL || '/grapevine/api'}/auth/logout`, {
				method: 'POST',
				credentials: 'include'
			});
			if (res.ok) {
				const d = await res.json();
				currentUser.set(null);
				window.location.href = d.redirect_url;
				return;
			}
		} catch {}
		currentUser.set(null);
	}

	onMount(() => {
		if (!$currentUser) getMe().then((u) => currentUser.set(u));
	});
</script>

<svelte:head>
	<title>Changelog — Grapevine</title>
</svelte:head>

<div class="shell">
	<!-- header -->
	<header>
		<div class="header-inner">
			<a href="{base}/" class="brand">grapevine</a>

			<div style="margin-left: auto;">
				{#if $currentUser}
					<div class="relative" bind:this={dropref}>
						<button
							type="button"
							aria-label="User menu"
							onclick={() => (dropopen = !dropopen)}
							class="avatar"
							title={$currentUser.display_name}
						>
							{initials}
						</button>

						{#if dropopen}
							<div class="drop">
								<div class="drop-name">{$currentUser.display_name}</div>
								<div class="drop-sep"></div>
								{#if $currentUser.is_admin}
									<a href="{base}/admin" onclick={() => (dropopen = false)} class="drop-item">Admin</a>
								{/if}
								<a href="{base}/profile" onclick={() => (dropopen = false)} class="drop-item">Profile</a>
								<button type="button" onclick={() => { dropopen = false; settingsopen = true; }} class="drop-item">Settings</button>
								<button type="button" onclick={logout} class="drop-item danger">Logout</button>
							</div>
						{/if}
					</div>
				{:else}
					<a href="{base}/login" class="loginbtn">Login</a>
				{/if}
			</div>
		</div>
	</header>

	<!-- content -->
	<main>
		<div class="page">
			<div class="heading">
				<h1>Changelog</h1>
				<p>Updates, improvements, and fixes to Grapevine.</p>
			</div>

			<div class="feed">
				{#each data.entries as entry}
					<div class="entry">
						<div class="date">{entry.label}</div>
						<div class="line-col">
							<span class="dot" aria-hidden="true"></span>
						</div>
						<div class="content">
							{#each entry.features as feature, i}
								<div class="feature" class:notfirst={i > 0}>
									<p class="etitle">{@html md(feature.title)}</p>
									{#if feature.desc}
										<p class="edesc">{@html md(feature.desc)}</p>
									{/if}
									{#if feature.items.length}
										<ul>
											{#each feature.items as item}
												<li>{@html md(item)}</li>
											{/each}
										</ul>
									{/if}
								</div>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>
	</main>

	<!-- footer -->
	<footer>
		<div></div>
		<div class="footer-center">
			made with &hearts; by
			<a href="https://osdg.iiit.ac.in" target="_blank" rel="noopener noreferrer">osdg</a>
		</div>
		<div class="footer-right">
			<a href="https://github.com/OSDG-IIITH/grapevine" target="_blank" rel="noopener noreferrer" aria-label="github">
				<IconBrandGithub size={16} stroke={1.6} aria-hidden="true" />
			</a>
		</div>
	</footer>
</div>

{#if settingsopen && $currentUser}
	<SettingsModal user={$currentUser} onclose={() => settingsopen = false} />
{/if}

<style>
	.shell {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	/* ── header ── */
	header {
		position: sticky;
		top: 0;
		z-index: 50;
		border-bottom: 1px solid var(--border);
		background: rgba(18, 24, 20, 0.78);
		backdrop-filter: blur(14px) saturate(120%);
	}

	.header-inner {
		width: 100%;
		padding: 14px 28px;
		display: flex;
		align-items: center;
	}

	.brand {
		font-family: var(--serif, 'Instrument Serif', Georgia, serif);
		font-size: 22px;
		font-style: normal;
		color: var(--fg);
		text-decoration: none;
		letter-spacing: -0.01em;
	}

	.loginbtn {
		display: inline-flex;
		align-items: center;
		border-radius: 7px;
		border: 1px solid var(--border-2);
		background: var(--bg-2);
		padding: 5px 14px;
		font-size: 0.8125rem;
		color: var(--fg-2);
		text-decoration: none;
		transition: color 120ms, background 120ms;
	}

	.loginbtn:hover {
		background: var(--bg-3);
		color: var(--fg);
	}

	.avatar {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: 50%;
		border: 1px solid var(--border-2);
		background: linear-gradient(135deg, #2f4a33, #1a221d);
		font-size: 11px;
		font-weight: 500;
		letter-spacing: 0.04em;
		color: var(--fg);
		cursor: pointer;
	}

	.drop {
		position: absolute;
		left: 0;
		top: calc(100% + 8px);
		z-index: 100;
		min-width: 180px;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--bg-2);
		box-shadow: 0 8px 24px rgba(0,0,0,0.4);
		overflow: hidden;
	}

	.drop-name {
		padding: 9px 12px;
		font-size: 0.75rem;
		color: var(--fg-3);
	}

	.drop-sep {
		border-top: 1px solid var(--border);
	}

	:global(.drop-item) {
		display: flex;
		width: 100%;
		padding: 9px 12px;
		font-size: 0.8125rem;
		color: var(--fg-2);
		background: none;
		border: none;
		cursor: pointer;
		text-decoration: none;
		transition: background 100ms, color 100ms;
		text-align: left;
	}

	:global(.drop-item:hover) {
		background: var(--bg-3);
		color: var(--fg);
	}

	:global(.drop-item.danger:hover) {
		color: var(--danger, #e05252);
	}

	/* ── main ── */
	main {
		flex: 1;
	}

	.page {
		max-width: 900px;
		margin: 0 auto;
		padding: 64px 40px 96px;
	}

	.heading {
		margin-bottom: 56px;
	}

	h1 {
		font-family: 'Instrument Serif', Georgia, serif;
		font-size: 2.6rem;
		font-weight: 400;
		font-style: italic;
		color: var(--fg);
		margin: 0 0 8px;
	}

	.heading p {
		color: var(--fg-3);
		font-size: 0.875rem;
		margin: 0;
	}

	/* ── feed ── */
	.feed {
		display: flex;
		flex-direction: column;
	}

	.entry {
		display: grid;
		grid-template-columns: 130px 24px 1fr;
		align-items: start;
		padding-bottom: 52px;
		position: relative;
	}

	.entry:last-child {
		padding-bottom: 0;
	}

	.entry:not(:last-child) .line-col::after {
		content: '';
		position: absolute;
		top: 8px;
		left: 130px;
		width: 1px;
		height: calc(100% - 8px);
		background: var(--border);
		transform: translateX(11px);
	}

	.date {
		font-size: 0.8rem;
		color: var(--accent-2);
		font-variant-numeric: tabular-nums;
		text-align: right;
		padding-right: 20px;
		padding-top: 1px;
		white-space: nowrap;
	}

	.line-col {
		display: flex;
		justify-content: center;
		padding-top: 2px;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--accent);
		box-shadow: 0 0 0 2px var(--accent-dim);
		flex-shrink: 0;
		display: block;
	}

	.content {
		padding-left: 20px;
	}

	.feature.notfirst {
		margin-top: 24px;
	}

	.etitle {
		font-size: 1rem;
		font-weight: 600;
		color: var(--fg);
		margin: 0 0 6px;
	}

	.edesc {
		font-size: 0.8125rem;
		color: var(--fg-3);
		margin: 0 0 12px;
		line-height: 1.55;
	}

	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	li {
		font-size: 0.875rem;
		color: var(--fg-2);
		line-height: 1.6;
		display: flex;
		gap: 10px;
	}

	li::before {
		content: '·';
		color: var(--fg-4);
		flex-shrink: 0;
	}

	.content :global(strong) {
		font-family: var(--sans);
		font-weight: 600;
		color: var(--fg);
	}

	.content :global(em) {
		font-style: italic;
	}

	.content :global(code) {
		font-family: var(--mono);
		font-size: 0.8em;
		color: var(--fg-2);
		background: var(--bg-3);
		padding: 1px 5px;
		border-radius: 4px;
	}

	/* ── footer ── */
	footer {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		border-top: 1px solid var(--border);
		padding: 24px 40px;
		font-size: 0.8125rem;
		color: var(--fg-3);
	}

	.footer-center a {
		color: var(--fg-2);
		text-decoration: none;
		transition: color 120ms;
	}

	.footer-center a:hover {
		color: var(--fg);
	}

	.footer-right {
		display: flex;
		justify-content: flex-end;
		color: var(--fg-3);
	}

	.footer-right a {
		color: var(--fg-3);
		text-decoration: none;
		transition: color 120ms;
	}

	.footer-right a:hover {
		color: var(--fg);
	}

	@media (max-width: 560px) {
		.header-inner { padding: 14px 20px; }

		.page { padding: 48px 20px 64px; }

		.entry {
			grid-template-columns: 1fr;
			padding-bottom: 36px;
		}

		.date { text-align: left; padding-right: 0; margin-bottom: 8px; }
		.line-col { display: none; }
		.content { padding-left: 0; }

		footer { padding: 20px; grid-template-columns: 1fr; gap: 8px; }
		.footer-right { justify-content: flex-start; }
	}
</style>
