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
	}

	let { items, active, mono = true, onchange }: Props = $props();
</script>

<div class="mt-2 mb-6 flex gap-0 border-b border-[var(--border)]">
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
