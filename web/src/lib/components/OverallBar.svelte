<script lang="ts">
	interface Props {
		value: number;
		interactive?: boolean;
		disabled?: boolean;
		onchange?: (v: number) => void;
		onhover?: (v: number | null) => void;
	}

	let { value, interactive = false, disabled = false, onchange, onhover }: Props = $props();

	let el = $state<HTMLElement | null>(null);
	let hoverval = $state<number | null>(null);

	function snap(v: number): number {
		return Math.round(Math.max(1, Math.min(5, v)) * 10) / 10;
	}

	function fromptr(e: MouseEvent): number {
		if (!el) return value;
		const r = el.getBoundingClientRect();
		return snap(1 + Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * 4);
	}

	function pct(v: number): string {
		return `${((v - 1) / 4) * 100}%`;
	}

	// treat value=0 as 1 for positioning (no solid fill shown when value=0)
	const hasrating = $derived(value > 0);
	const ev = $derived(Math.max(1, value));

	const solidend = $derived(hoverval !== null && hoverval < ev ? hoverval : ev);
	const dimstart = $derived(hoverval !== null ? Math.min(ev, hoverval) : null);
	const dimend = $derived(hoverval !== null ? Math.max(ev, hoverval) : null);
	const showdim = $derived(dimstart !== null && dimend !== null && dimend > dimstart);
	const atvalue = $derived(hoverval !== null && Math.abs(hoverval - ev) < 0.2);

	function move(e: MouseEvent) {
		const v = fromptr(e);
		hoverval = v;
		onhover?.(v);
	}

	function leave() {
		hoverval = null;
		onhover?.(null);
	}

	function onkeydown(e: KeyboardEvent) {
		if (!interactive || disabled || !onchange) return;
		if (e.key === 'ArrowRight' || e.key === 'ArrowUp') { e.preventDefault(); onchange(snap(value + 0.1)); }
		if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') { e.preventDefault(); onchange(snap(value - 0.1)); }
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={el}
	role={interactive && !disabled ? 'slider' : undefined}
	aria-valuenow={interactive ? value : undefined}
	aria-valuemin={interactive ? 1 : undefined}
	aria-valuemax={interactive ? 5 : undefined}
	tabindex={interactive && !disabled ? 0 : undefined}
	class="relative overflow-hidden rounded-[4px] transition-opacity duration-[140ms] {interactive && !disabled ? 'cursor-pointer' : 'cursor-default'} {disabled ? 'opacity-40' : ''}"
	style="height: 20px; background: var(--bg-inset); border: 1px solid var(--border);"
	onmousemove={interactive && !disabled ? move : undefined}
	onmouseleave={interactive && !disabled ? leave : undefined}
	onclick={interactive && !disabled && onchange ? (e) => onchange(fromptr(e)) : undefined}
	onkeydown={interactive && !disabled ? onkeydown : undefined}
>
	<!-- solid fill -->
	{#if hasrating}
		<div
			class="absolute inset-y-0 left-0"
			style="width: {pct(solidend)}; background: var(--accent); box-shadow: 0 0 {atvalue ? '10px rgba(107,143,111,0.55)' : '8px rgba(107,143,111,0.35)'}; transition: width 80ms, box-shadow 140ms;"
		></div>
	{/if}

	<!-- dim preview (hover) -->
	{#if showdim && dimstart !== null && dimend !== null}
		<div
			class="absolute inset-y-0"
			style="left: {pct(dimstart)}; width: {((dimend - dimstart) / 4) * 100}%; background: var(--accent-dim);"
		></div>
	{/if}
</div>
