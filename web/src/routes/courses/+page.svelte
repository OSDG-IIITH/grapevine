<script lang="ts">
	import { getCourses } from '$lib/api';
	import type { CourseLean } from '$lib/types';
	import BrowseCard from '$lib/components/BrowseCard.svelte';
	import Pager from '$lib/components/Pager.svelte';

	const TYPES = ['core', 'open', 'breadth', 'stream', 'bouquet', 'hs', 'sci', 'math'];
	const PER_PAGE = 9;

	let all = $state<CourseLean[]>([]);
	let filters = $state(new Set<string>());
	let q = $state('');
	let page = $state(1);

	$effect(() => {
		getCourses().then((data) => { if (data) all = data; });
	});

	const filtered = $derived(
		all.filter((c) => {
			if (filters.size && !filters.has(c.type)) return false;
			if (q.trim()) {
				const s = q.toLowerCase();
				if (!c.name.toLowerCase().includes(s) && !c.code.toLowerCase().includes(s)) return false;
			}
			return true;
		})
	);

	const totalpages = $derived(Math.max(1, Math.ceil(filtered.length / PER_PAGE)));
	const visible = $derived(filtered.slice((page - 1) * PER_PAGE, page * PER_PAGE));

	function toggle(type: string) {
		const n = new Set(filters);
		n.has(type) ? n.delete(type) : n.add(type);
		filters = n;
		page = 1;
	}
</script>

<div class="mx-auto w-full px-8 pb-[120px] pt-10 max-w-[1180px]" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	<!-- breadcrumbs -->
	<div class="mb-[18px] flex items-center gap-2 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">
		<a href="/" class="transition-colors duration-[120ms] hover:text-[var(--fg)]">grapevine</a>
		<span class="text-[var(--fg-4)]">/</span>
		<span class="text-[var(--fg-2)]">courses</span>
	</div>

	<!-- page head -->
	<div class="flex flex-wrap items-start justify-between gap-6">
		<div>
			<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: 56px; line-height: 1.05; letter-spacing: -0.015em;">
				Courses
			</h1>
			<div class="mb-[22px] flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
				<span>{all.length} courses</span>
				<span class="text-[var(--fg-4)]">·</span>
				<span>browse, filter, or search</span>
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
	<div class="mb-[22px] mt-[18px] flex items-center justify-between gap-4">
		<div class="flex flex-wrap gap-[6px]">
			<button
				type="button"
				onclick={() => { filters = new Set(); page = 1; }}
				class="rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms] {filters.size === 0 ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
				style="font-family: var(--mono);"
			>all</button>
			{#each TYPES as t (t)}
				<button
					type="button"
					onclick={() => toggle(t)}
					class="rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms] {filters.has(t) ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
					style="font-family: var(--mono);"
				>{t}</button>
			{/each}
		</div>
		<div class="flex w-[260px] items-center gap-2 rounded-[7px] border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px]">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 text-[var(--fg-4)]" aria-hidden="true">
				<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
			</svg>
			<input
				class="w-full bg-transparent text-[13px] outline-none placeholder:text-[var(--fg-4)]"
				placeholder="Search course or code…"
				bind:value={q}
				oninput={() => (page = 1)}
			/>
		</div>
	</div>

	<!-- grid -->
	{#if visible.length === 0}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
			No courses match these filters.
		</div>
	{:else}
		<div style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px;">
			{#each visible as c (c.id)}
				<BrowseCard kind="course" item={c} />
			{/each}
		</div>
	{/if}

	<Pager {page} totalpages={totalpages} totalitems={filtered.length} onchange={(p) => (page = p)} />
</div>
