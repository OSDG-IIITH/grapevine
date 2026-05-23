<script lang="ts">
	import SegBar from './SegBar.svelte';

	interface Props {
		overall: number;
		axes: Record<string, number>;
		axisorder: string[];
		axislabels: Record<string, string>;
		reviewcount?: number;
		bar?: 'continuous' | 'segmented';
	}

	let { overall, axes, axisorder, axislabels, reviewcount, bar = 'continuous' }: Props = $props();

	const stars = $derived(Math.round(overall));
</script>

<div
	class="mb-8 grid overflow-hidden rounded-[10px] border border-[var(--border)]"
	style="grid-template-columns: 280px 1fr; background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107, 143, 111, 0.035), transparent 42%);"
>
	<!-- overall score -->
	<div class="flex flex-col justify-center gap-[6px] border-r border-[var(--border)] p-[26px_28px]">
		<div
			class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]"
			style="font-family: var(--mono);"
		>
			Overall
		</div>
		<div
			class="flex items-baseline gap-1 text-[64px] leading-none text-[var(--fg)]"
			style="font-family: var(--serif); text-shadow: 0 0 28px rgba(127, 165, 131, 0.20);"
		>
			{overall.toFixed(1)}<span class="text-[18px] text-[var(--fg-3)]">/5</span>
		</div>
		<div class="mt-[6px] inline-flex gap-[2px] text-[13px] text-[var(--accent)]">
			{#each [1, 2, 3, 4, 5] as i (i)}
				<svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" class={i <= stars ? '' : 'text-[var(--fg-4)]'}>
					<path d="M12 2l2.9 6.3 6.8.7-5.1 4.6 1.4 6.8L12 17l-6 3.4 1.4-6.8L2.3 9l6.8-.7L12 2z" />
				</svg>
			{/each}
		</div>
		{#if reviewcount != null}
			<div class="mt-2 text-[12px] text-[var(--fg-3)]">{reviewcount} reviews</div>
		{/if}
	</div>

	<!-- axes -->
	<div
		class="grid items-center gap-[12px_18px] p-[22px_28px]"
		style="grid-template-columns: 110px 1fr 40px;"
	>
		{#each axisorder as k (k)}
			<div class="text-[13px] text-[var(--fg-2)]">{axislabels[k] ?? k}</div>
			{#if bar === 'segmented'}
				<SegBar score={axes[k] ?? 0} />
			{:else}
				<div
					class="relative h-2 overflow-hidden rounded-full border border-[var(--border)] bg-[var(--bg-inset)]"
					aria-label="{axes[k]?.toFixed(1)} of 5"
				>
					<div
						class="absolute inset-y-0 left-0 rounded-full transition-[width] duration-[320ms]"
						style="width: {Math.max(0, Math.min(100, ((axes[k] ?? 0) / 5) * 100))}%; background: linear-gradient(90deg, var(--accent-3), var(--accent)); box-shadow: 0 0 12px rgba(127,165,131,0.30);"
					></div>
				</div>
			{/if}
			<div
				class="text-right text-[12px] text-[var(--fg-2)]"
				style="font-family: var(--mono);"
			>
				{(axes[k] ?? 0).toFixed(1)}
			</div>
		{/each}
	</div>
</div>
