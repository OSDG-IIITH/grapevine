<script lang="ts">
	interface TabItem {
		id: string;
		label: string;
		count?: number;
	}

	interface Props {
		items: TabItem[];
		active: string;
		mono?: boolean;
		onchange?: (id: string) => void;
		onadd?: () => void;
		addtitle?: string;
	}

	let { items, active, mono = true, onchange, onadd, addtitle }: Props = $props();
</script>

<div class="mt-2 mb-6 flex items-center gap-1 border-b border-[var(--border)]">
	<div class="flex gap-0">
		{#each items as it (it.id)}
			<button
				type="button"
				onclick={() => onchange?.(it.id)}
				class="inline-flex items-baseline gap-2 border-b-[1.5px] -mb-px px-4 py-3 text-[13px] transition-[color,border-color] duration-[120ms]
					{it.id === active
					? 'text-[var(--fg)] border-[var(--accent)]'
					: 'text-[var(--fg-3)] border-transparent hover:text-[var(--fg)]'}"
				style={mono ? 'font-family: var(--mono);' : ''}
				aria-label={it.label}
			>
				{it.label}
				{#if it.count != null}
					<span class="text-[11px] {it.id === active ? 'text-[var(--accent)]' : 'text-[var(--fg-4)]'}"
						>{it.count}</span
					>
				{/if}
			</button>
		{/each}
	</div>
	{#if onadd}
		<button
			type="button"
			onclick={onadd}
			title={addtitle}
			class="flex h-[28px] w-[28px] items-center justify-center rounded-[6px] text-[var(--fg-3)] transition-colors hover:bg-[var(--bg-3)] hover:text-[var(--fg)] -mb-[1.5px]"
			aria-label={addtitle ?? 'Add item'}
		>
			<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
				<line x1="12" y1="5" x2="12" y2="19"></line>
				<line x1="5" y1="12" x2="19" y2="12"></line>
			</svg>
		</button>
	{/if}
</div>
