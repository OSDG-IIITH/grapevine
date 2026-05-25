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
	const hasrating = $derived(score > 0);
	let hovered = $state(0);
	const solidcls = 'bg-[var(--accent)] border-transparent shadow-[0_0_8px_rgba(107,143,111,0.35)]';
	const dimcls = 'bg-[var(--accent-dim)] border-transparent';
	const glowcls = 'shadow-[0_0_10px_rgba(107,143,111,0.55)]';

	function basecls(i: number): string {
		if (i <= full) return solidcls;
		if (hasfrac && i === full + 1) return 'bg-[rgba(107,143,111,0.32)] border border-[var(--accent-dim)]';
		return 'bg-[var(--bg-inset)] border border-[var(--border)]';
	}

	function segcls(i: number): string {
		if (!hovered) return basecls(i);
		if (!hasrating) return i <= hovered ? dimcls : basecls(i);
		if (!hasfrac && hovered === full) return i === hovered ? `${basecls(i)} ${glowcls}` : basecls(i);
		if (hovered < score) {
			if (i <= hovered) return solidcls;
			if (i <= score) return dimcls;
			return basecls(i);
		}
		if (hovered > score) {
			if (i <= score) return basecls(i);
			if (i <= hovered) return dimcls;
			return basecls(i);
		}
		return basecls(i);
	}
</script>


<div
	style="display: grid; grid-template-columns: repeat(5, 1fr); gap: {size === 'sm' ? '3px' : '4px'}; height: {size === 'sm' ? '5px' : '10px'};"
	role={interactive ? 'group' : undefined}
>
	{#each [1, 2, 3, 4, 5] as i (i)}
		<button
			type="button"
			class="rounded-[2px] transition-[background,border-color,box-shadow] duration-[140ms] {segcls(i)} {interactive ? 'cursor-pointer' : 'cursor-default'}"
			style="height: 100%;"
			tabindex={interactive ? 0 : -1}
			aria-label="{i} of 5"
			onclick={interactive && onchange ? () => onchange(i) : undefined}
			onmouseenter={interactive ? () => (hovered = i) : undefined}
			onmouseleave={interactive ? () => (hovered = 0) : undefined}
			onfocus={interactive ? () => (hovered = i) : undefined}
			onblur={interactive ? () => (hovered = 0) : undefined}
		></button>
	{/each}
</div>
