<script lang="ts">
	import { getFaculty, getLabs } from '$lib/api';
	import type { FacultyLean, LabLean } from '$lib/types';
	import BrowseCard from '$lib/components/BrowseCard.svelte';
	import Pager from '$lib/components/Pager.svelte';

	const PER_PAGE = 9;

	let all = $state<FacultyLean[]>([]);
	let labs = $state<LabLean[]>([]);
	let labfilter = $state('all');
	let sort = $state<'' | 'rating_asc' | 'rating_desc'>('');
	let q = $state('');
	let page = $state(1);

	$effect(() => {
		getLabs().then((data) => { if (data) labs = data; });
	});

	$effect(() => {
		const params: { sort?: 'rating_asc' | 'rating_desc' } = {};
		if (sort) params.sort = sort;
		page = 1;
		getFaculty(params).then((data) => { if (data) all = data; });
	});

	const filtered = $derived(
		all.filter((f) => {
			if (labfilter !== 'all') {
				if (labfilter === 'independent' && f.lab) return false;
				if (labfilter !== 'independent' && f.lab !== labfilter) return false;
			}
			if (q.trim() && !f.name.toLowerCase().includes(q.toLowerCase())) return false;
			return true;
		})
	);

	const totalpages = $derived(Math.max(1, Math.ceil(filtered.length / PER_PAGE)));
	const visible = $derived(filtered.slice((page - 1) * PER_PAGE, page * PER_PAGE));

	function nextsort() {
		if (sort === '') sort = 'rating_desc';
		else if (sort === 'rating_desc') sort = 'rating_asc';
		else sort = '';
	}
</script>

<div class="mx-auto w-full max-w-[1180px] px-4 pb-[120px] pt-10 sm:px-8" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	<!-- breadcrumbs -->
	<div class="mb-[18px] flex items-center gap-2 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">
		<a href="/" class="transition-colors duration-[120ms] hover:text-[var(--fg)]">grapevine</a>
		<span class="text-[var(--fg-4)]">/</span>
		<span class="text-[var(--fg-2)]">faculty</span>
	</div>

	<!-- page head -->
	<div class="flex flex-wrap items-start justify-between gap-6">
		<div>
			<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;">
				Faculty
			</h1>
			<div class="mb-[22px] flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
				<span>{all.length} faculty</span>
				<span class="text-[var(--fg-4)]">·</span>
				<span>advisor ratings from research students</span>
			</div>
		</div>
		<a
			href="/review"
			class="inline-flex items-center gap-2 self-start whitespace-nowrap rounded-[7px] px-[14px] py-2 text-[13px] font-medium transition-[background,border-color] duration-[120ms]"
			style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
		>
			<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
				<path d="M5 12h14M12 5v14" />
			</svg>
			Write a review
		</a>
	</div>

	<!-- toolbar -->
	<div class="mb-[22px] mt-[18px] flex flex-wrap items-center justify-between gap-3">
		<div class="flex flex-wrap gap-[6px]">
			<!-- mobile: lab select -->
			<select
				bind:value={labfilter}
				onchange={() => (page = 1)}
				class="sm:hidden rounded-[5px] border bg-[var(--bg-inset)] px-[10px] py-[5px] text-[11px] tracking-[0.04em] outline-none transition-[border-color,color] duration-[120ms] {labfilter !== 'all' ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)]'}"
				style="font-family: var(--mono);"
			>
				<option value="all">all labs</option>
				{#each labs as l (l.id)}<option value={l.short}>{l.short}</option>{/each}
				<option value="independent">independent</option>
			</select>
			<!-- desktop: lab pills -->
			<button
				type="button"
				onclick={() => { labfilter = 'all'; page = 1; }}
				class="hidden sm:inline-block rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms] {labfilter === 'all' ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
				style="font-family: var(--mono);"
			>all labs</button>
			{#each labs as l (l.id)}
				<button
					type="button"
					onclick={() => { labfilter = l.short; page = 1; }}
					class="hidden sm:inline-block rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms] {labfilter === l.short ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
					style="font-family: var(--mono);"
				>{l.short}</button>
			{/each}
			<button
				type="button"
				onclick={() => { labfilter = 'independent'; page = 1; }}
				class="hidden sm:inline-block rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms] {labfilter === 'independent' ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
				style="font-family: var(--mono);"
			>independent</button>
		</div>
		<div class="flex flex-1 items-center gap-2 sm:flex-none">
			<button
				type="button"
				onclick={nextsort}
				class="shrink-0 rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms] {sort ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
				style="font-family: var(--mono);"
			>
				{sort === 'rating_desc' ? 'rating ↓' : sort === 'rating_asc' ? 'rating ↑' : 'sort'}
			</button>
			<div class="flex flex-1 items-center gap-2 rounded-[7px] border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px] sm:w-[260px] sm:flex-none">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 text-[var(--fg-4)]" aria-hidden="true">
					<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
				</svg>
				<input
					class="w-full bg-transparent text-[13px] outline-none placeholder:text-[var(--fg-4)]"
					placeholder="Search faculty…"
					bind:value={q}
					oninput={() => (page = 1)}
				/>
			</div>
		</div>
	</div>

	<!-- grid -->
	{#if visible.length === 0}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
			No faculty match these filters.
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-[14px] sm:grid-cols-2 lg:grid-cols-3">
			{#each visible as f (f.id)}
				<BrowseCard kind="faculty" item={f} />
			{/each}
		</div>
	{/if}

	<Pager {page} totalpages={totalpages} totalitems={filtered.length} onchange={(p) => (page = p)} />
</div>
