<script lang="ts">
	import type { CourseReview, AdvisorReview } from '$lib/types';
	import { IconFlag, IconTrash, IconPencil, IconCheck } from '@tabler/icons-svelte';
	import SegBar from './SegBar.svelte';
	import OverallBar from './OverallBar.svelte';

	interface Props {
		review: CourseReview | AdvisorReview;
		axisorder: string[];
		axislabels: Record<string, string>;
		showoffering?: boolean;
		offeringcode?: string;
		coursecode?: string;
		vote: 0 | 1 | -1;
		flagged: boolean;
		canEdit: boolean;
		canDelete: boolean;
		deleting: boolean;
		editing: boolean;
		saving: boolean;
		editbody: string;
		editvalues: Record<string, number>;
		editoverall: number | null;
		onvote: (v: 0 | 1 | -1) => void;
		onflag: () => void;
		oneditstart: () => void;
		oneditvalue: (k: string, v: number) => void;
		oneditoverall: (v: number) => void;
		oneditbody: (v: string) => void;
		onsaved: () => void;
		ondelete: () => void;
		onclose: () => void;
	}

	let { review, axisorder, axislabels, showoffering = false, offeringcode, coursecode, vote, flagged, canEdit, canDelete, deleting, editing, saving, editbody, editvalues, editoverall, onvote, onflag, oneditstart, oneditvalue, oneditoverall, oneditbody, onsaved, ondelete, onclose }: Props = $props();

	const editcalcavg = $derived.by(() => {
		if (!editing) return null;
		const vals = axisorder.map((k) => editvalues[k] ?? axisval(k));
		return vals.reduce((s, v) => s + v, 0) / vals.length;
	});
	const editshownoverall = $derived(editoverall !== null ? editoverall : editcalcavg);

	let hoveredoverall = $state<number | null>(null);

	const stars = $derived(Math.round(review.overall ?? 0));
	const initialvote = $derived((review.user_vote ?? 0) as 0 | 1 | -1);
	const baseup = $derived(review.upvotes - (initialvote === 1 ? 1 : 0));
	const basedown = $derived(review.downvotes - (initialvote === -1 ? 1 : 0));
	const shownup = $derived(baseup + (vote === 1 ? 1 : 0));
	const showndown = $derived(basedown + (vote === -1 ? 1 : 0));
	const canflag = $derived(!canEdit);
	const cansave = $derived(editing && !saving && editbody.trim().length > 20);

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	function fmtdate(iso: string): string {
		return new Date(iso).toLocaleDateString('en-US', { month: 'short', year: 'numeric' });
	}

	function axisval(k: string): number {
		return ((review as unknown) as Record<string, number>)[k] ?? 0;
	}
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
			<div class="flex flex-col gap-2">
				<div class="flex items-center justify-between gap-2">
					<div class="flex min-w-0 flex-wrap items-center gap-[5px] text-[12px] text-[var(--fg-3)]">
						{#if review.anonymous || !review.author}
							<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">anonymous</span>
						{:else}
							<span class="text-[13px] font-medium text-[var(--fg-2)]">{review.author.display_name}</span>
						{/if}
						<span class="text-[var(--fg-4)]">·</span>
						<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">{fmtdate(review.created_at)}</span>
						{#if showoffering && (coursecode || offeringcode)}
							<span class="text-[var(--fg-4)]">·</span>
							{#if coursecode}
								<a
									href={`/courses/${encodeURIComponent(coursecode)}`}
									class="text-[11px] tracking-[0.04em] text-[var(--fg-2)] transition-colors duration-[120ms] hover:text-[var(--fg)]"
									style="font-family: var(--mono);"
								>{coursecode}</a>
							{:else}
								<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">{offeringcode}</span>
							{/if}
							{#if coursecode && offeringcode}
								<span class="text-[var(--fg-4)]">·</span>
								<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">{offeringcode}</span>
							{/if}
						{/if}
					</div>
					<div class="inline-flex shrink-0 items-center gap-[2px] text-[var(--accent)]">
						{#each [1, 2, 3, 4, 5] as i (i)}
							<svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor" class={i <= stars ? '' : 'text-[var(--border-strong)]'}>
								<path d="M12 2l2.9 6.3 6.8.7-5.1 4.6 1.4 6.8L12 17l-6 3.4 1.4-6.8L2.3 9l6.8-.7L12 2z" />
							</svg>
						{/each}
						<span class="ml-[5px] text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{(review.overall ?? 0).toFixed(1)}</span>
					</div>
				</div>
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

		<!-- axes -->
		<div
			class="grid items-center gap-[10px_14px] border-b border-[var(--border)] p-[18px_22px]"
			style="grid-template-columns: 90px 1fr 30px;"
		>
			{#each axisorder as k (k)}
				<span class="text-[12px] text-[var(--fg-3)]">{axislabels[k] ?? k}</span>
				{#if editing}
					<SegBar score={editvalues[k] ?? axisval(k)} size="md" interactive onchange={(v) => oneditvalue(k, v)} />
					<span class="text-right text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{(editvalues[k] ?? axisval(k)).toFixed(1)}</span>
				{:else}
					<SegBar score={axisval(k)} size="sm" />
					<span class="text-right text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{axisval(k).toFixed(1)}</span>
				{/if}
			{/each}
			{#if editing}
				<div class="col-span-3 mt-1 flex flex-col gap-[6px]">
					<div class="flex items-center justify-between gap-3 text-[11px] text-[var(--fg-3)]" style="font-family: var(--mono);">
						<span>overall {editoverall !== null ? '(adjusted)' : '(average)'}</span>
						<span class="flex items-center gap-[6px]">
							{#if hoveredoverall !== null && editshownoverall !== null && Math.abs(hoveredoverall - editshownoverall) >= 0.05}
								<span class="line-through text-[var(--fg-4)]">{editshownoverall.toFixed(1)}</span>
								<span class="text-[var(--accent-2)]">{hoveredoverall.toFixed(1)}</span>
							{:else}
								<span class="text-[var(--accent-2)]">{editshownoverall !== null ? editshownoverall.toFixed(1) : '—'}</span>
							{/if}
						</span>
					</div>
					<OverallBar
						value={editshownoverall ?? 0}
						interactive
						onchange={(v) => oneditoverall(v)}
						onhover={(v) => { hoveredoverall = v; }}
					/>
				</div>
			{/if}
		</div>

		<!-- body -->
		<div
			class="flex-1 overflow-y-auto p-[18px_22px] text-[14px] leading-[1.7] text-[var(--fg)]"
			style="white-space: pre-wrap; text-wrap: pretty;"
		>
			{#if editing}
				<textarea
					class="w-full resize-y rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[12px] text-[14px] leading-[1.6] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
					style="min-height: 140px;"
					placeholder="Share your experience."
					value={editbody}
					oninput={(e) => oneditbody((e.target as HTMLTextAreaElement).value)}
				></textarea>
			{:else}
				{review.body}
			{/if}
		</div>

		<!-- footer -->
		<div class="flex items-center justify-between gap-2 border-t border-[var(--border)] p-[12px_22px]">
			<div class="flex items-center gap-1">
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
			<div class="flex items-center gap-2">
				{#if canflag}
					<button
						type="button"
						onclick={onflag}
						class="inline-flex items-center gap-[6px] rounded-md px-3 py-[6px] text-[12px] transition-[color,background] duration-[120ms] {flagged ? 'bg-[var(--danger-bg)] text-[var(--danger)]' : 'text-[var(--fg-3)] hover:bg-[var(--danger-bg)] hover:text-[var(--danger)]'}"
						style="font-family: var(--mono);"
						aria-label={flagged ? 'Flagged' : 'Flag review'}
					>
						<IconFlag size={13} stroke={1.7} />
						{flagged ? 'flagged' : 'flag'}
					</button>
				{/if}
				{#if canEdit}
					{#if editing}
						<button
							type="button"
							onclick={onsaved}
							disabled={!cansave}
							class="inline-flex items-center gap-[6px] rounded-md px-3 py-[6px] text-[12px] transition-[color,background] duration-[120ms] {cansave ? 'text-[var(--fg-2)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]' : 'text-[var(--fg-4)]'}"
							style="font-family: var(--mono);"
							aria-label="Save review"
						>
							<IconCheck size={13} stroke={1.7} />
							{saving ? 'saving…' : 'save'}
						</button>
					{:else}
						<button
							type="button"
							onclick={oneditstart}
							class="inline-flex items-center gap-[6px] rounded-md px-3 py-[6px] text-[12px] text-[var(--fg-3)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
							style="font-family: var(--mono);"
							aria-label="Edit review"
						>
							<IconPencil size={13} stroke={1.7} />
							edit
						</button>
					{/if}
				{/if}
				{#if canDelete}
					<button
						type="button"
						onclick={ondelete}
						disabled={deleting}
						class="inline-flex items-center gap-[6px] rounded-md px-3 py-[6px] text-[12px] text-[var(--fg-3)] transition-[color,background] duration-[120ms] hover:bg-[var(--danger-bg)] hover:text-[var(--danger)]"
						style="font-family: var(--mono);"
						aria-label="Delete review"
					>
						<IconTrash size={13} stroke={1.7} />
						{deleting ? 'deleting…' : 'delete'}
					</button>
				{/if}
			</div>
		</div>
	</div>
</div>
