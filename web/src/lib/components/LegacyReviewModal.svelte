<script lang="ts">
	import type { LegacyCourseReview, LegacyAdvisorReview } from '$lib/types';

	interface Props {
		review: LegacyCourseReview | LegacyAdvisorReview;
		vote: 0 | 1 | -1;
		shownup: number;
		showndown: number;
		onvote: (v: 0 | 1 | -1) => void;
		onclose: () => void;
	}

	let { review, vote, shownup, showndown, onvote, onclose }: Props = $props();

	const stars = $derived(review.original_rating ?? 0);

	function fmtdate(iso: string): string {
		return new Date(iso).toLocaleDateString('en-US', { month: 'short', year: 'numeric' });
	}

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	use:portal
	role="presentation"
	class="fixed inset-0 z-[200] flex items-center justify-center p-6"
	style="background: rgba(10,14,12,0.62); backdrop-filter: blur(4px); animation: fadeIn 160ms ease-out;"
	onclick={onclose}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="flex max-h-[calc(100vh-48px)] w-[min(580px,100%)] flex-col overflow-hidden rounded-xl border border-[var(--border-strong)]"
		style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.04), transparent 40%); animation: fadeUp 200ms cubic-bezier(.2,.6,.2,1);"
		onclick={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<!-- header -->
		<div class="flex items-start justify-between gap-3 border-b border-[var(--border)] p-[20px_22px_18px]">
			<div class="flex min-w-0 flex-wrap items-center gap-[5px] text-[12px] text-[var(--fg-3)]">
				<span class="text-[13px] font-medium text-[var(--fg-2)]">Archive Review</span>
				<span class="text-[var(--fg-4)]">·</span>
				<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">{fmtdate(review.created_at)}</span>
			</div>
			<div class="ml-auto mr-2 inline-flex shrink-0 items-center gap-[2px] text-[var(--accent)]">
				{#each [1, 2, 3, 4, 5] as i (i)}
					<svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" class={i <= stars ? '' : 'text-[var(--border-strong)]'}>
						<path d="M12 2l2.9 6.3 6.8.7-5.1 4.6 1.4 6.8L12 17l-6 3.4 1.4-6.8L2.3 9l6.8-.7L12 2z" />
					</svg>
				{/each}
				<span class="ml-[5px] text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{stars}.0</span>
			</div>
			<button
				type="button"
				aria-label="Close"
				onclick={onclose}
				class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-[var(--fg-3)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-4)] hover:text-[var(--fg)]"
			>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
					<path d="M18 6 6 18M6 6l12 12" />
				</svg>
			</button>
		</div>

		<!-- body -->
		<div
			class="flex-1 overflow-y-auto p-[18px_22px] text-[14px] leading-[1.7] text-[var(--fg)]"
			style="white-space: pre-wrap; text-wrap: pretty;"
		>
			{#if review.body}
				{review.body}
			{:else}
				<span class="italic text-[var(--fg-4)]">No written review.</span>
			{/if}
		</div>

		<!-- footer -->
		<div class="flex items-center gap-1 border-t border-[var(--border)] p-[12px_22px]">
			<button
				type="button"
				onclick={() => onvote(vote === 1 ? 0 : 1)}
				class="inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {vote === 1 ? 'text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
				style="font-family: var(--mono);"
				aria-label="Upvote"
			>
				<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M12 19V5M5 12l7-7 7 7" />
				</svg>
				<span class="min-w-[14px] text-left">{shownup}</span>
			</button>
			<button
				type="button"
				onclick={() => onvote(vote === -1 ? 0 : -1)}
				class="inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {vote === -1 ? 'text-[var(--danger)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
				style="font-family: var(--mono);"
				aria-label="Downvote"
			>
				<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M12 5v14M19 12l-7 7-7-7" />
				</svg>
				<span class="min-w-[14px] text-left">{showndown}</span>
			</button>
		</div>
	</div>
</div>
