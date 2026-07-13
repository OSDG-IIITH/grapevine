<script lang="ts">
	import { onMount } from 'svelte';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Drawer from '$lib/components/ui/drawer/index.js';
	import IconSortAscending from '@tabler/icons-svelte/icons/sort-ascending';
	import IconSortDescending from '@tabler/icons-svelte/icons/sort-descending';

	let {
		kind,
		value = $bindable('rating_desc'),
		onchange
	}: {
		kind: 'course' | 'faculty';
		value?: string;
		onchange?: (v: string) => void;
	} = $props();

	let open = $state(false);
	let isDesktop = $state(true);

	onMount(() => {
		const mq = window.matchMedia('(min-width: 768px)');
		isDesktop = mq.matches;
		const handler = (e: MediaQueryListEvent) => { isDesktop = e.matches; };
		mq.addEventListener('change', handler);
		return () => mq.removeEventListener('change', handler);
	});

	const coursedims = [
		{ value: 'rating', label: 'Rating' },
		{ value: 'name', label: 'Name' },
		{ value: 'reviews', label: 'Reviews' },
		{ value: 'difficulty', label: 'Easiness' },
		{ value: 'workload', label: 'Workload' },
		{ value: 'teaching', label: 'Teaching' },
		{ value: 'grading', label: 'Grading' },
		{ value: 'content', label: 'Content' },
	];

	const facultydims = [
		{ value: 'rating', label: 'Rating' },
		{ value: 'name', label: 'Name' },
		{ value: 'reviews', label: 'Reviews' },
		{ value: 'mentorship', label: 'Mentorship' },
		{ value: 'availability', label: 'Availability' },
		{ value: 'support', label: 'Support' },
		{ value: 'research', label: 'Research' },
		{ value: 'workload', label: 'Workload' },
	];

	const dims = $derived(kind === 'course' ? coursedims : facultydims);
	const dim = $derived(value.replace(/_(asc|desc)$/, '') || 'rating');
	const dir = $derived(value.endsWith('_asc') ? 'asc' : 'desc');
	const activelabel = $derived(dims.find((d) => d.value === dim)?.label ?? 'Rating');
	const isactive = $derived(value !== 'rating_desc');

	function selectdim(d: string) {
		value = `${d}_${dir}`;
		open = false;
		onchange?.(value);
	}

	function toggledir() {
		const newdir = dir === 'asc' ? 'desc' : 'asc';
		value = `${dim}_${newdir}`;
		onchange?.(value);
	}

	const btnbase = 'inline-flex shrink-0 items-center gap-[6px] rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms]';
	const btninactive = 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]';
	const btnactive = 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]';
</script>

{#snippet menu()}
	<div class="flex flex-col gap-[2px] p-1">
		<div class="px-3 py-[6px] text-[10px] font-medium uppercase tracking-[0.07em] text-[var(--fg-4)]" style="font-family: var(--mono);">Sort by</div>
		{#each dims as d}
			<button
				type="button"
				onclick={() => selectdim(d.value)}
				class="flex w-full items-center justify-between rounded-[5px] px-3 py-[7px] text-left text-[12px] transition-colors duration-[80ms] {d.value === dim ? 'text-[var(--accent-2)]' : 'text-[var(--fg-2)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
				style="font-family: var(--mono);"
			>
				<span>{d.label}</span>
				{#if d.value === dim}
					<svg width="11" height="9" viewBox="0 0 11 9" fill="none" aria-hidden="true">
						<path d="M1 4l3.5 3.5L10 1" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
					</svg>
				{/if}
			</button>
		{/each}
	</div>
{/snippet}

{#snippet triggerinner()}
	<span class="min-w-0 truncate">{activelabel}</span>
	<svg class="shrink-0 opacity-50" width="9" height="6" viewBox="0 0 9 6" fill="none" aria-hidden="true">
		<path d="M1 1l3.5 3.5L8 1" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
	</svg>
{/snippet}

<div class="flex shrink-0 items-center gap-[4px]">
	{#if isDesktop}
		<Popover.Root bind:open>
			<Popover.Trigger
				class="{btnbase} {isactive ? btnactive : btninactive}"
				style="font-family: var(--mono);"
				aria-label="Sort by"
			>
				{@render triggerinner()}
			</Popover.Trigger>
			<Popover.Content
				class="w-[170px] rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-inset)] p-0 shadow-lg"
				sideOffset={5}
				align="end"
			>
				{@render menu()}
			</Popover.Content>
		</Popover.Root>
	{:else}
		<Drawer.Root bind:open>
			<Drawer.Trigger
				class="{btnbase} {isactive ? btnactive : btninactive}"
				style="font-family: var(--mono);"
				aria-label="Sort by"
			>
				{@render triggerinner()}
			</Drawer.Trigger>
			<Drawer.Content class="border-[var(--border)] bg-[var(--bg-2)]">
				<div class="px-2 pb-8 pt-2">
					{@render menu()}
				</div>
			</Drawer.Content>
		</Drawer.Root>
	{/if}

	<button
		type="button"
		onclick={toggledir}
		aria-label={dir === 'asc' ? 'Switch to descending' : 'Switch to ascending'}
		class="flex h-[28px] w-[28px] shrink-0 items-center justify-center rounded-[5px] border transition-[color,background,border-color] duration-[120ms] {isactive ? btnactive : btninactive}"
	>
		{#if dir === 'asc'}
			<IconSortAscending size={14} stroke={1.75} />
		{:else}
			<IconSortDescending size={14} stroke={1.75} />
		{/if}
	</button>
</div>
