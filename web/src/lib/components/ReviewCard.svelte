<script lang="ts">
	import type { CourseReview, AdvisorReview, EditCourseReview, EditAdvisorReview } from '$lib/types';
	import {
		voteCourseReview, unvoteCourseReview,
		voteAdvisorReview, unvoteAdvisorReview,
		flagCourseReview, flagAdvisorReview,
		deleteCourseReview, deleteAdvisorReview,
		editCourseReview, editAdvisorReview
	} from '$lib/api';
	import { currentUser } from '$lib/stores';
	import { IconFlag, IconTrash, IconPencil } from '@tabler/icons-svelte';
	import ReviewModal from './ReviewModal.svelte';
	import FlagModal from './FlagModal.svelte';

	interface Props {
		review: CourseReview | AdvisorReview;
		axisorder: string[];
		axislabels: Record<string, string>;
		showoffering?: boolean;
		offeringcode?: string;
		coursecode?: string;
		ondelete?: (id: string) => void;
		onedit?: (updated: CourseReview | AdvisorReview) => void;
	}

	let { review, axisorder, axislabels, showoffering = false, offeringcode, coursecode, ondelete, onedit }: Props = $props();

	let open = $state(false);
	// svelte-ignore state_referenced_locally
	let vote = $state<0 | 1 | -1>((review.user_vote ?? 0) as 0 | 1 | -1);
	let flagged = $state(false);
	let flagopen = $state(false);
	let flagsending = $state(false);
	let deleting = $state(false);
	let editmode = $state(false);
	let saving = $state(false);
	let editbody = $state('');
	let editvalues = $state<Record<string, number>>({});

	const kind = $derived<'course' | 'advisor'>('offering_id' in review ? 'course' : 'advisor');
	const stars = $derived(Math.round(review.overall ?? 0));
	const initialvote = $derived((review.user_vote ?? 0) as 0 | 1 | -1);
	const baseup = $derived(review.upvotes - (initialvote === 1 ? 1 : 0));
	const basedown = $derived(review.downvotes - (initialvote === -1 ? 1 : 0));
	const shownup = $derived(baseup + (vote === 1 ? 1 : 0));
	const showndown = $derived(basedown + (vote === -1 ? 1 : 0));
	const canDelete = $derived(!!$currentUser && !!review.author && $currentUser.id === review.author.id);
	const canflag = $derived(!canDelete);
	const cansave = $derived(editmode && !saving && editbody.trim().length > 20);

	function fmtdate(iso: string): string {
		return new Date(iso).toLocaleDateString('en-US', { month: 'short', year: 'numeric' });
	}

	function stop(e: Event) {
		e.stopPropagation();
	}

	function axisfromreview(): Record<string, number> {
		const out: Record<string, number> = {};
		for (const k of axisorder) out[k] = ((review as unknown) as Record<string, number>)[k] ?? 0;
		return out;
	}

	async function handlevote(newvote: 0 | 1 | -1) {
		const prev = vote;
		vote = newvote;
		let ok: boolean;
		if (newvote === 0) {
			ok = kind === 'course' ? await unvoteCourseReview(review.id) : await unvoteAdvisorReview(review.id);
		} else {
			ok = kind === 'course' ? await voteCourseReview(review.id, newvote) : await voteAdvisorReview(review.id, newvote);
		}
		if (!ok) vote = prev;
	}

	async function handleflag(reason: string) {
		if (flagged || flagsending) return;
		flagsending = true;
		const ok = kind === 'course'
			? await flagCourseReview(review.id, reason)
			: await flagAdvisorReview(review.id, reason);
		flagsending = false;
		if (ok) flagged = true;
		flagopen = false;
	}

	function startedit(e?: Event) {
		if (e) stop(e);
		if (!canDelete) return;
		editmode = true;
		editbody = review.body;
		editvalues = axisfromreview();
		open = true;
	}

	function seteditvalue(k: string, v: number) {
		editvalues = { ...editvalues, [k]: v };
	}

	async function handlesave() {
		if (!canDelete || !cansave) return;
		saving = true;
		const payload = { body: editbody.trim() } as EditCourseReview | EditAdvisorReview;
		const base = axisfromreview();
		for (const k of axisorder) (payload as Record<string, number>)[k] = editvalues[k] ?? base[k];
		const updated = kind === 'course'
			? await editCourseReview(review.id, payload as EditCourseReview)
			: await editAdvisorReview(review.id, payload as EditAdvisorReview);
		saving = false;
		if (!updated) return;
		review = updated;
		vote = (updated.user_vote ?? 0) as 0 | 1 | -1;
		onedit?.(updated);
		editmode = false;
	}

	async function handledelete() {
		if (!canDelete || deleting) return;
		deleting = true;
		const ok = kind === 'course'
			? await deleteCourseReview(review.id)
			: await deleteAdvisorReview(review.id);
		deleting = false;
		if (!ok) return;
		ondelete?.(review.id);
		editmode = false;
		saving = false;
		open = false;
	}
</script>

<div
	role="button"
	tabindex="0"
	class="group relative flex flex-col gap-[10px] overflow-hidden rounded-[10px] border border-[var(--border)] p-[14px_16px] text-left transition-[background,border-color] duration-[160ms] hover:border-[var(--border-2)] hover:bg-[var(--bg-3)] cursor-pointer"
	style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.03), transparent 42%);"
	onclick={() => (open = true)}
	onkeydown={(e) => e.key === 'Enter' && (open = true)}
>
	<!-- hover glow -->
	<div
		class="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-[180ms] group-hover:opacity-100"
		style="background: radial-gradient(ellipse 220px 110px at 100% 0%, rgba(107,143,111,0.05), transparent 70%);"
	></div>

	<!-- meta row -->
	<div class="flex min-w-0 items-center justify-between gap-2">
		<div class="flex min-w-0 flex-wrap items-center gap-[5px] text-[12px] text-[var(--fg-3)]">
			{#if review.anonymous || !review.author}
				<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">anonymous</span>
			{:else}
				<span class="text-[13px] font-medium text-[var(--fg-2)]">{review.author.display_name}</span>
			{/if}
			{#if showoffering && (coursecode || offeringcode)}
				<span class="text-[var(--fg-4)]">·</span>
				{#if coursecode}
					<a
						href={`/courses/${encodeURIComponent(coursecode)}`}
						onclick={stop}
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
			<span class="text-[var(--fg-4)]">·</span>
			<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">{fmtdate(review.created_at)}</span>
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

	<!-- excerpt -->
	<div class="line-clamp-3 flex-1 text-[13px] leading-[1.55] text-[var(--fg-2)]">{review.body}</div>

	<!-- actions -->
	<div class="mt-auto flex items-center gap-[2px]">
		<button
			type="button"
			aria-label="Upvote"
			onclick={(e) => { stop(e); handlevote(vote === 1 ? 0 : 1); }}
			class="relative z-[2] inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {vote === 1 ? 'text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
			style="font-family: var(--mono);"
		>
			<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M12 19V5M5 12l7-7 7 7" />
			</svg>
			<span class="min-w-[14px] text-left">{shownup}</span>
		</button>
		<button
			type="button"
			aria-label="Downvote"
			onclick={(e) => { stop(e); handlevote(vote === -1 ? 0 : -1); }}
			class="relative z-[2] inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {vote === -1 ? 'text-[var(--danger)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
			style="font-family: var(--mono);"
		>
			<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M12 5v14M19 12l-7 7-7-7" />
			</svg>
			<span class="min-w-[14px] text-left">{showndown}</span>
		</button>
		{#if canflag}
			<button
				type="button"
				aria-label={flagged ? 'Flagged' : 'Flag review'}
				onclick={(e) => { stop(e); if (!flagged) flagopen = true; }}
				class="relative z-[2] ml-auto inline-flex h-7 w-7 items-center justify-center rounded-[5px] transition-colors duration-[120ms] {flagged ? 'text-[var(--danger)]' : 'text-[var(--fg-3)] hover:text-[var(--danger)]'}"
			>
				<IconFlag size={13} stroke={1.7} fill={flagged ? 'currentColor' : 'none'} />
			</button>
		{:else}
			<button
				type="button"
				aria-label="Edit review"
				onclick={startedit}
				class="relative z-[2] ml-auto inline-flex h-7 w-7 items-center justify-center rounded-[5px] transition-colors duration-[120ms] text-[var(--fg-3)] hover:text-[var(--fg)]"
			>
				<IconPencil size={13} stroke={1.7} />
			</button>
		{/if}
		{#if canDelete}
			<button
				type="button"
				aria-label="Delete review"
				onclick={(e) => { stop(e); handledelete(); }}
				disabled={deleting}
				class="relative z-[2] inline-flex h-7 w-7 items-center justify-center rounded-[5px] transition-colors duration-[120ms] text-[var(--fg-3)] hover:text-[var(--danger)]"
			>
				<IconTrash size={13} stroke={1.7} />
			</button>
		{/if}
	</div>
</div>

{#if open}
	<ReviewModal
		{review}
		{axisorder}
		{axislabels}
		{showoffering}
		{offeringcode}
		{coursecode}
		{vote}
		{flagged}
		{canDelete}
		{deleting}
		editing={editmode}
		saving={saving}
		editbody={editbody}
		editvalues={editvalues}
		onvote={(v) => handlevote(v)}
		onflag={() => { if (canflag && !flagged) flagopen = true; }}
		oneditstart={startedit}
		oneditvalue={seteditvalue}
		oneditbody={(v) => (editbody = v)}
		onsaved={handlesave}
		ondelete={handledelete}
		onclose={() => { open = false; editmode = false; saving = false; }}
	/>
{/if}

{#if flagopen}
	<FlagModal
		submitting={flagsending}
		onsubmit={handleflag}
		onclose={() => (flagopen = false)}
	/>
{/if}
