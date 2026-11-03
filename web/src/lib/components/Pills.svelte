<script lang="ts">
	interface Props {
		items: { id: string; label: string; count?: number }[];
		active: string;
		onchange: (id: string) => void;
		onadd?: () => void;
		addtitle?: string;
	}

	let { items, active, onchange, onadd, addtitle }: Props = $props();

	const base = 'inline-flex shrink-0 items-center gap-[6px] rounded-[5px] border px-[10px] py-[5px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms]';
</script>

<div class="mb-5 flex flex-wrap items-center gap-[6px]" style="font-family: var(--mono);">
	{#each items as it (it.id)}
		<button
			type="button"
			aria-label={it.label}
			aria-pressed={it.id === active}
			onclick={() => onchange(it.id)}
			class="{base} {it.id === active ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
		>
			{it.label}
			{#if it.count != null}
				<span class={it.id === active ? 'text-[var(--accent)]' : 'text-[var(--fg-4)]'}>{it.count}</span>
			{/if}
		</button>
	{/each}
	{#if onadd}
		<button
			type="button"
			onclick={onadd}
			title={addtitle}
			aria-label={addtitle ?? 'Add item'}
			class="{base} border-dashed border-[var(--border-2)] text-[var(--fg-4)] hover:border-[var(--border-strong)] hover:text-[var(--fg-2)]"
		>
			<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
			{addtitle}
		</button>
	{/if}
</div>
