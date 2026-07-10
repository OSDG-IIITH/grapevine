<script lang="ts">
	import { base } from '$app/paths';
	import { getLabs } from '$lib/api';
	import type { LabLean } from '$lib/types';
	import BrowseCard from '$lib/components/BrowseCard.svelte';
	import Pager from '$lib/components/Pager.svelte';

	const PER_PAGE = 9;

	let all = $state<LabLean[]>([]);
	let q = $state('');
	let page = $state(1);

	$effect(() => {
		getLabs().then((data) => { if (Array.isArray(data)) all = data; });
	});

	const filtered = $derived(
		all.filter((l) => {
			if (!q.trim()) return true;
			const s = q.toLowerCase();
			return l.name.toLowerCase().includes(s) || l.short.toLowerCase().includes(s);
		})
	);

	const totalpages = $derived(Math.max(1, Math.ceil(filtered.length / PER_PAGE)));
	const visible = $derived(filtered.slice((page - 1) * PER_PAGE, page * PER_PAGE));
</script>

<svelte:head>
	<title>Labs · grapevine</title>
</svelte:head>

<div class="mx-auto w-full max-w-[1180px] px-4 pb-[120px] pt-10 sm:px-8" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	<!-- breadcrumbs -->
	<div class="mb-[18px] flex items-center gap-2 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">
		<a href="{base}/" class="transition-colors duration-[120ms] hover:text-[var(--fg)]">grapevine</a>
		<span class="text-[var(--fg-4)]">/</span>
		<span class="text-[var(--fg-2)]">labs</span>
	</div>

	<!-- page head -->
	<div class="flex flex-wrap items-start justify-between gap-6">
		<div>
			<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;">
				Research Labs
			</h1>
			<div class="mb-[22px] flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
				<span>{all.length} research centers</span>
				<span class="text-[var(--fg-4)]">·</span>
				<span>aggregated advisor ratings across all members</span>
			</div>
		</div>
	</div>

	<!-- toolbar -->
	<div class="mb-[22px] mt-[18px]">
		<div class="flex w-full items-center gap-2 rounded-[7px] border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px] sm:w-[260px]">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 text-[var(--fg-4)]" aria-hidden="true">
				<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
			</svg>
			<input
				class="w-full bg-transparent text-[13px] outline-none placeholder:text-[var(--fg-4)]"
				placeholder="Search labs…"
				bind:value={q}
				oninput={() => (page = 1)}
			/>
		</div>
	</div>

	<!-- grid -->
	{#if visible.length === 0}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
			No labs match this search.
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-[14px] sm:grid-cols-2 lg:grid-cols-3">
			{#each visible as l (l.id)}
				<BrowseCard kind="lab" item={l} />
			{/each}
		</div>
	{/if}

	<Pager {page} totalpages={totalpages} totalitems={filtered.length} onchange={(p) => (page = p)} />
</div>
