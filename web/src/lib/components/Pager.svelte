<script lang="ts">
	interface Props {
		page: number;
		totalpages: number;
		totalitems: number;
		onchange?: (p: number) => void;
	}

	let { page, totalpages, totalitems, onchange }: Props = $props();

	const items = $derived((() => {
		if (totalpages <= 7) return Array.from({ length: totalpages }, (_, i) => i + 1) as (number | '...')[];
		const r: (number | '...')[] = [1];
		if (page > 3) r.push('...');
		const lo = Math.max(2, page - 1);
		const hi = Math.min(totalpages - 1, page + 1);
		for (let i = lo; i <= hi; i++) r.push(i);
		if (page < totalpages - 2) r.push('...');
		r.push(totalpages);
		return r;
	})());
</script>

<div
	class="mt-7 flex items-center justify-between text-[12px] text-[var(--fg-3)]"
	style="font-family: var(--mono);"
>
	<span>{totalitems} results</span>
	<div class="flex items-center gap-1">
		<button
			type="button"
			aria-label="Previous page"
			onclick={() => onchange?.(Math.max(1, page - 1))}
			disabled={page === 1}
			class="inline-flex h-7 w-7 items-center justify-center rounded-md border border-transparent transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
			style={page === 1 ? 'opacity: 0.4;' : ''}
		>‹</button>

		{#each items as item, i (i)}
			{#if item === '...'}
				<span class="inline-flex h-7 w-7 items-center justify-center text-[var(--fg-4)]">…</span>
			{:else}
				<button
					type="button"
					aria-label="Page {item}"
					onclick={() => onchange?.(item as number)}
					class="inline-flex h-7 w-7 items-center justify-center rounded-md border transition-[color,background,border-color] duration-[120ms]
						{item === page
						? 'text-[var(--accent-2)] bg-[var(--accent-bg)] border-[var(--accent-dim)]'
						: 'border-transparent text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
				>{item}</button>
			{/if}
		{/each}

		<button
			type="button"
			aria-label="Next page"
			onclick={() => onchange?.(Math.min(totalpages, page + 1))}
			disabled={page === totalpages}
			class="inline-flex h-7 w-7 items-center justify-center rounded-md border border-transparent transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
			style={page === totalpages ? 'opacity: 0.4;' : ''}
		>›</button>
	</div>
</div>
