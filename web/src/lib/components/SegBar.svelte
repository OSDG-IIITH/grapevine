<script lang="ts">
	interface Props {
		score: number;
		interactive?: boolean;
		size?: 'sm' | 'md';
		onchange?: (v: number) => void;
	}

	let { score, interactive = false, size = 'md', onchange }: Props = $props();

	const full = $derived(Math.floor(score));
	const hasfrac = $derived(score % 1 > 0.001);

	function segcls(i: number): string {
		if (i <= full) return 'bg-[var(--accent)] border-transparent shadow-[0_0_8px_rgba(107,143,111,0.35)]';
		if (hasfrac && i === full + 1) return 'bg-[rgba(107,143,111,0.32)] border border-[var(--accent-dim)]';
		return 'bg-[var(--bg-inset)] border border-[var(--border)]';
	}
</script>

<div
	style="display: grid; grid-template-columns: repeat(5, 1fr); gap: {size === 'sm' ? '3px' : '4px'}; height: {size === 'sm' ? '5px' : '8px'};"
	role={interactive ? 'group' : undefined}
>
	{#each [1, 2, 3, 4, 5] as i (i)}
		<button
			type="button"
			class="rounded-[2px] transition-[background,border-color,box-shadow] duration-[140ms] {segcls(i)} {interactive ? 'cursor-pointer hover:bg-[var(--accent-dim)] hover:border-[var(--accent-dim)]' : 'cursor-default'}"
			style="height: 100%;"
			tabindex={interactive ? 0 : -1}
			aria-label="{i} of 5"
			onclick={interactive && onchange ? () => onchange(i) : undefined}
		></button>
	{/each}
</div>
