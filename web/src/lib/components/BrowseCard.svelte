<script lang="ts">
	import type { CourseLean, FacultyLean, LabLean } from '$lib/types';
	import SegBar from './SegBar.svelte';

	interface Props {
		kind: 'course' | 'faculty' | 'lab';
		item: CourseLean | FacultyLean | LabLean;
	}

	let { kind, item }: Props = $props();

	const href = $derived(
		kind === 'course'
			? `/courses/${(item as CourseLean).code}`
			: kind === 'faculty'
				? `/faculty/${(item as FacultyLean).slug}`
				: `/labs/${(item as LabLean).short}`
	);
</script>

<a
	{href}
	class="group relative flex min-h-[120px] flex-col gap-3 overflow-hidden rounded-[10px] border border-[var(--border)] px-5 py-[18px] transition-[background,border-color,transform] duration-[180ms] hover:-translate-y-px hover:border-[var(--border-strong)] hover:bg-[var(--bg-3)]"
	style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107, 143, 111, 0.035), transparent 42%);"
>
	<!-- hover glow -->
	<div
		class="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-200 group-hover:opacity-100"
		style="background: radial-gradient(ellipse 200px 100px at 100% 0%, rgba(107, 143, 111, 0.06), transparent 70%);"
	></div>

	{#if kind === 'course'}
		{@const c = item as CourseLean}
		<div class="flex items-center justify-between gap-[10px]">
			<span class="text-[11px] tracking-[0.04em] text-[var(--fg-3)]" style="font-family: var(--mono);">{c.code}</span>
			<span
				class="text-[11px] tracking-[0.04em] text-[var(--fg-4)] {c.type === 'core' ? 'text-[var(--accent-2)]' : ''}"
				style="font-family: var(--mono);"
			>{c.type}</span>
		</div>
		<div class="text-[15px] font-medium leading-[1.3] tracking-[-0.01em] text-[var(--fg)]">{c.name}</div>
		<div class="mt-auto flex items-center gap-2">
			<div class="flex flex-1 items-center gap-2">
				<div class="flex-1"><SegBar score={c.overall ?? 0} size="sm" /></div>
				<span class="text-[12px] text-[var(--fg)]" style="font-family: var(--mono);">{(c.overall ?? 0).toFixed(1)}</span>
			</div>
		</div>

	{:else if kind === 'faculty'}
		{@const f = item as FacultyLean}
		<div class="flex items-center justify-between gap-[10px]">
			{#if f.lab}
				<span class="text-[11px] tracking-[0.04em] text-[var(--fg-3)]" style="font-family: var(--mono);">{f.lab}</span>
			{:else}
				<span></span>
			{/if}
		</div>
		<div class="text-[15px] font-medium leading-[1.3] tracking-[-0.01em] text-[var(--fg)]">{f.name}</div>
		<div class="mt-auto flex items-center gap-2">
			<div class="flex flex-1 items-center gap-2">
				<div class="flex-1"><SegBar score={f.overall ?? 0} size="sm" /></div>
				<span class="text-[12px] text-[var(--fg)]" style="font-family: var(--mono);">{(f.overall ?? 0).toFixed(1)}</span>
			</div>
		</div>

	{:else}
		{@const l = item as LabLean}
		<div class="flex items-center justify-between gap-[10px]">
			<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{l.short}</span>
			<span class="text-[11px] tracking-[0.04em] text-[var(--fg-4)]" style="font-family: var(--mono);">{l.facultycount} faculty</span>
		</div>
		<div class="text-[15px] font-medium leading-[1.3] tracking-[-0.01em] text-[var(--fg)]">{l.name}</div>
		<div class="mt-auto flex items-center gap-2">
			<div class="flex flex-1 items-center gap-2">
				<div class="flex-1"><SegBar score={l.overall ?? 0} size="sm" /></div>
				<span class="text-[12px] text-[var(--fg)]" style="font-family: var(--mono);">{(l.overall ?? 0).toFixed(1)}</span>
			</div>
		</div>
	{/if}
</a>
