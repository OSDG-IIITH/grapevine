<script lang="ts">
	import { page } from '$app/stores';
	import { getLab } from '$lib/api';
	import type { LabDetail } from '$lib/types';
	import { ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS } from '$lib/types';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import RatingsBlock from '$lib/components/RatingsBlock.svelte';
	import SegBar from '$lib/components/SegBar.svelte';

	const shortname = $derived($page.params.shortname);

	let lab = $state<LabDetail | null>(null);
	let error = $state('');

	$effect(() => {
		const s = shortname;
		if (!s) return;
		lab = null;
		error = '';
		getLab(s)
			.then((data) => (lab = data))
			.catch(() => { error = 'Lab not found.'; });
	});
</script>

<div class="mx-auto w-full max-w-[1180px] px-8 pb-[120px] pt-10" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	{#if error}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">{error}</div>

	{:else if lab}
		<Crumbs items={[
			{ label: 'grapevine', href: '/' },
			{ label: 'labs', href: '/labs' },
			{ label: lab.short.toLowerCase() }
		]} />

		<!-- page head -->
		<div class="flex flex-wrap items-start justify-between gap-6">
			<div>
				<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: 56px; line-height: 1.05; letter-spacing: -0.015em;">
					{lab.name}
				</h1>
				<div class="mb-[22px] flex flex-wrap items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
					<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{lab.short}</span>
					<span class="text-[var(--fg-4)]">·</span>
					<span>{lab.facultycount} faculty</span>
				</div>
			</div>
		</div>

		<p class="mb-7 mt-[14px] max-w-[720px] leading-[1.65] text-[var(--fg-2)]" style="font-size: 15px; text-wrap: pretty;">
			{lab.description}
		</p>

		<RatingsBlock
			overall={lab.overall}
			axes={lab.axes}
			axisorder={[...ADVISOR_AXIS_ORDER]}
			axislabels={ADVISOR_AXIS_LABELS}
			bar="continuous"
		/>

		<!-- faculty list -->
		<div class="mb-3 mt-6 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
			Faculty members ({lab.faculty.length})
		</div>

		{#if lab.faculty.length === 0}
			<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
				No faculty profiles available yet.
			</div>
		{:else}
			<div
				class="overflow-hidden rounded-[10px] border border-[var(--border)]"
				style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107, 143, 111, 0.035), transparent 42%);"
			>
				{#each lab.faculty as m (m.id)}
					<a
						href="/faculty/{m.slug}"
						class="grid items-center gap-5 border-b border-[var(--border)] px-5 py-4 transition-colors duration-[120ms] last:border-b-0 hover:bg-[var(--bg-3)]"
						style="grid-template-columns: 1fr 200px 60px;"
					>
						<div>
							<div class="text-[14px] text-[var(--fg)]">{m.name}</div>
							<div class="mt-[2px] text-[12px] text-[var(--fg-3)]">{m.title}</div>
						</div>
						<SegBar score={m.overall} size="sm" />
						<div class="text-right text-[12px] text-[var(--fg)]" style="font-family: var(--mono);">{m.overall.toFixed(1)}</div>
					</a>
				{/each}
			</div>
		{/if}

	{:else}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
	{/if}
</div>
