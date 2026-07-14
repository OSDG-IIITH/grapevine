<script lang="ts">
	import { onMount } from 'svelte';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Drawer from '$lib/components/ui/drawer/index.js';
	import * as Command from '$lib/components/ui/command/index.js';
	import { cn } from '$lib/utils.js';

	type Item = { value: string; label: string };

	let {
		items,
		value = $bindable(''),
		placeholder = 'Select…',
		searchplaceholder = 'Search…',
		onselect,
		popoverwidth,
		class: className = '',
		style = ''
	}: {
		items: Item[];
		value?: string;
		placeholder?: string;
		searchplaceholder?: string;
		onselect?: (v: string) => void;
		popoverwidth?: string;
		class?: string;
		style?: string;
	} = $props();

	let open = $state(false);
	let isDesktop = $state(false);
	const selected = $derived(items.find((i) => i.value === value));

	onMount(() => {
		const mq = window.matchMedia('(min-width: 768px)');
		isDesktop = mq.matches;
		const handler = (e: MediaQueryListEvent) => { isDesktop = e.matches; };
		mq.addEventListener('change', handler);
		return () => mq.removeEventListener('change', handler);
	});
</script>

{#snippet commandlist()}
	<Command.Root class="rounded-[7px] bg-[var(--bg-inset)]">
		<Command.Input placeholder={searchplaceholder} />
		<Command.List>
			<Command.Empty>No results.</Command.Empty>
			<Command.Group>
				{#each items as item (item.value)}
					<Command.Item
						value={item.label}
						onSelect={() => { value = item.value; open = false; onselect?.(item.value); }}
					>
						<span class="min-w-0 truncate">{item.label}</span>
					</Command.Item>
				{/each}
			</Command.Group>
		</Command.List>
	</Command.Root>
{/snippet}

{#snippet trigger()}
	<span class="min-w-0 truncate flex-1">{selected?.label ?? placeholder}</span>
	<svg class="shrink-0" width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
		<path d="M2 4l3 3 3-3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
	</svg>
{/snippet}

{#if isDesktop}
	<Popover.Root bind:open>
		<Popover.Trigger
			class={cn('inline-flex items-center justify-between gap-2 text-left', className)}
			{style}
			aria-label="Select option"
		>
			{@render trigger()}
		</Popover.Trigger>
		<Popover.Content
			class="z-[220] w-auto p-0 rounded-[8px] bg-[var(--bg-inset)] border border-[var(--border-2)] shadow-lg ring-0"
			style="width: {popoverwidth ?? 'var(--bits-floating-anchor-width, 200px)'};"
			sideOffset={5}
			align="start"
		>
			{@render commandlist()}
		</Popover.Content>
	</Popover.Root>
{:else}
	<Drawer.Root bind:open>
		<Drawer.Trigger
			class={cn('inline-flex items-center justify-between gap-2 text-left', className)}
			{style}
			aria-label="Select option"
		>
			{@render trigger()}
		</Drawer.Trigger>
		<Drawer.Content class="z-[220] border-[var(--border)]" overlayClass="z-[219]">
			<div class="mt-4 border-t border-[var(--border)]">
				{@render commandlist()}
			</div>
		</Drawer.Content>
	</Drawer.Root>
{/if}
