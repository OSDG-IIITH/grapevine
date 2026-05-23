<script lang="ts">
	interface Props {
		page: number;
		totalpages: number;
		totalitems: number;
		onchange?: (p: number) => void;
	}

	let { page, totalpages, totalitems, onchange }: Props = $props();

	const pages = $derived(Array.from({ length: totalpages }, (_, i) => i + 1));
</script>

<div
	class="mt-7 flex items-center justify-between text-[12px] text-[var(--fg-3)]"
	style="font-family: var(--mono);"
>
	<span>{totalitems} results</span>
	<div class="flex gap-1">
		<button
			type="button"
			aria-label="Previous page"
			onclick={() => onchange?.(Math.max(1, page - 1))}
			disabled={page === 1}
			class="inline-flex h-7 w-7 items-center justify-center rounded-md border border-transparent transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
			style={page === 1 ? 'opacity: 0.4;' : ''}
		>‹</button>

		{#each pages as p (p)}
			<button
				type="button"
				aria-label="Page {p}"
				onclick={() => onchange?.(p)}
				class="inline-flex h-7 w-7 items-center justify-center rounded-md border transition-[color,background,border-color] duration-[120ms]
					{p === page
					? 'text-[var(--accent-2)] bg-[var(--accent-bg)] border-[var(--accent-dim)]'
					: 'border-transparent text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
			>{p}</button>
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
