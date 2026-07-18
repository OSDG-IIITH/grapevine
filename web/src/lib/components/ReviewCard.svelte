<script lang="ts">
	import { base } from '$app/paths';
	import type { CourseReview, AdvisorReview, LegacyCourseReview, LegacyAdvisorReview, EditCourseReview, EditAdvisorReview } from '$lib/types';
	import {
		voteCourseReview, unvoteCourseReview,
		voteAdvisorReview, unvoteAdvisorReview,
		voteLegacyCourseReview, unvoteLegacyCourseReview,
		voteLegacyAdvisorReview, unvoteLegacyAdvisorReview,
		flagCourseReview, flagAdvisorReview,
		deleteCourseReview, deleteAdvisorReview,
		deleteLegacyCourseReview, deleteLegacyAdvisorReview,
		editCourseReview, editAdvisorReview
	} from '$lib/api';
	import { currentUser } from '$lib/stores';
	import IconFlag from '@tabler/icons-svelte/icons/flag';
	import IconTrash from '@tabler/icons-svelte/icons/trash';
	import IconPencil from '@tabler/icons-svelte/icons/pencil';
	import ReviewModal from './ReviewModal.svelte';
	import LegacyReviewModal from './LegacyReviewModal.svelte';
	import FlagModal from './FlagModal.svelte';
	import { rendermarkdown } from '$lib/markdown';

	type AnyReview = CourseReview | AdvisorReview | LegacyCourseReview | LegacyAdvisorReview;
	type RegularReview = CourseReview | AdvisorReview;

	interface Props {
		review: AnyReview;
		axisorder: string[];
		axislabels: Record<string, string>;
		showoffering?: boolean;
		offeringcode?: string;
		coursecode?: string;
		ondelete?: (id: string) => void;
		onedit?: (updated: RegularReview) => void;
	}

	let { review, axisorder, axislabels, showoffering = false, offeringcode, coursecode, ondelete, onedit }: Props = $props();

	function islegacyfn(r: AnyReview): r is LegacyCourseReview | LegacyAdvisorReview {
		return !('anonymous' in r);
	}

	const islegacy = $derived(islegacyfn(review));
	const reg = $derived(islegacy ? null : review as RegularReview);

	let open = $state(false);
	// svelte-ignore state_referenced_locally
	let vote = $state<0 | 1 | -1>(((review as { user_vote?: number | null }).user_vote ?? 0) as 0 | 1 | -1);
	let flagged = $state(false);
	let flagopen = $state(false);
	let flagsending = $state(false);
	let deleting = $state(false);
	let editmode = $state(false);
	let saving = $state(false);
	let editbody = $state('');
	let editvalues = $state<Record<string, number>>({});
	let editoverall = $state<number | null>(null);

	const kind = $derived<'course' | 'advisor'>(
		!islegacy && 'offering_id' in review ? 'course' : 'advisor'
	);
	const legacykind = $derived<'course' | 'advisor'>(
		axisorder.includes('research') ? 'advisor' : 'course'
	);
	const stars = $derived(
		islegacy
			? ((review as LegacyCourseReview).original_rating ?? 0)
			: Math.round((reg?.overall ?? 0))
	);
	const overallval = $derived(
		islegacy
			? ((review as LegacyCourseReview).original_rating ?? 0)
			: (reg?.overall ?? 0)
	);
	const initialvote = $derived(((review as { user_vote?: number | null }).user_vote ?? 0) as 0 | 1 | -1);
	const allupvotes = $derived(islegacy ? (review as LegacyCourseReview).upvotes : (reg?.upvotes ?? 0));
	const alldownvotes = $derived(islegacy ? (review as LegacyCourseReview).downvotes : (reg?.downvotes ?? 0));
	const baseup = $derived(allupvotes - (initialvote === 1 ? 1 : 0));
	const basedown = $derived(alldownvotes - (initialvote === -1 ? 1 : 0));
	const shownup = $derived(baseup + (vote === 1 ? 1 : 0));
	const showndown = $derived(basedown + (vote === -1 ? 1 : 0));
	const isAuthor = $derived(!islegacy && !!$currentUser && !!reg?.author && $currentUser.id === reg.author.id);
	const isModerator = $derived(!!$currentUser?.is_admin);
	const canDelete = $derived(isAuthor || isModerator);
	const canEdit = $derived(isAuthor);
	const canflag = $derived(!islegacy && !isAuthor);
	const canvote = $derived(!!$currentUser);
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
		if (!canvote) return;
		const prev = vote;
		vote = newvote;
		let ok: boolean;
		if (islegacy) {
			if (newvote === 0) {
				ok = legacykind === 'course' ? await unvoteLegacyCourseReview(review.id) : await unvoteLegacyAdvisorReview(review.id);
			} else {
				ok = legacykind === 'course' ? await voteLegacyCourseReview(review.id, newvote) : await voteLegacyAdvisorReview(review.id, newvote);
			}
		} else {
			if (newvote === 0) {
				ok = kind === 'course' ? await unvoteCourseReview(review.id) : await unvoteAdvisorReview(review.id);
			} else {
				ok = kind === 'course' ? await voteCourseReview(review.id, newvote) : await voteAdvisorReview(review.id, newvote);
			}
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
		if (!canEdit) return;
		editmode = true;
		editbody = review.body ?? '';
		editvalues = axisfromreview();
		editoverall = null;
		open = true;
	}

	function seteditvalue(k: string, v: number) {
		editvalues = { ...editvalues, [k]: v };
	}

	async function handlesave() {
		if (!canEdit || !cansave) return;
		saving = true;
		const payload = { body: editbody.trim() } as EditCourseReview | EditAdvisorReview;
		const base = axisfromreview();
		for (const k of axisorder) (payload as Record<string, number>)[k] = editvalues[k] ?? base[k];
		const axisavg = axisorder.reduce((s, k) => s + ((payload as Record<string, number>)[k] ?? 0), 0) / axisorder.length;
		(payload as Record<string, number>).overall = editoverall !== null ? editoverall : axisavg;
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
		if (islegacy) {
			if (!isModerator || deleting) return;
			deleting = true;
			const ok = legacykind === 'course'
				? await deleteLegacyCourseReview(review.id)
				: await deleteLegacyAdvisorReview(review.id);
			deleting = false;
			if (!ok) return;
			ondelete?.(review.id);
			open = false;
		} else {
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
			{#if islegacy}
				<span class="text-[13px] font-medium text-[var(--fg-2)]">Archive Review</span>
			{:else if reg?.anonymous || !reg?.author}
				<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">anonymous</span>
			{:else}
				<span class="text-[13px] font-medium text-[var(--fg-2)]">{reg.author.display_name}</span>
			{/if}
			{#if !islegacy && showoffering && (coursecode || offeringcode)}
				<span class="text-[var(--fg-4)]">·</span>
				{#if coursecode}
					<a
						href={`${base}/courses/${encodeURIComponent(coursecode)}`}
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
			<span class="ml-[5px] text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{islegacy ? `${overallval}.0` : overallval.toFixed(1)}</span>
		</div>
	</div>

	<!-- excerpt -->
	<div class="line-clamp-3 flex-1 text-[13px] leading-[1.55] text-[var(--fg-2)]">
		{#if review.body}
			<div class="review-prose">{@html rendermarkdown(review.body)}</div>
		{:else}
			<span class="italic text-[var(--fg-4)]">No written review.</span>
		{/if}
	</div>

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
		{/if}
		{#if canEdit}
			<button
				type="button"
				aria-label="Edit review"
				onclick={startedit}
				class="relative z-[2] {canflag ? '' : 'ml-auto'} inline-flex h-7 w-7 items-center justify-center rounded-[5px] transition-colors duration-[120ms] text-[var(--fg-3)] hover:text-[var(--fg)]"
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
				class="relative z-[2] {(canflag || canEdit) ? '' : 'ml-auto'} inline-flex h-7 w-7 items-center justify-center rounded-[5px] transition-colors duration-[120ms] text-[var(--fg-3)] hover:text-[var(--danger)]"
			>
				<IconTrash size={13} stroke={1.7} />
			</button>
		{/if}
	</div>
</div>

{#if open}
	{#if islegacy}
		<LegacyReviewModal
			review={review as LegacyCourseReview | LegacyAdvisorReview}
			{vote}
			{shownup}
			{showndown}
			isModerator={isModerator}
			deleting={deleting}
			onvote={(v) => handlevote(v)}
			ondelete={handledelete}
			onclose={() => (open = false)}
		/>
	{:else}
		<ReviewModal
			review={reg!}
			{axisorder}
			{axislabels}
			{showoffering}
			{offeringcode}
			{coursecode}
			{vote}
			{flagged}
			{canEdit}
			{canDelete}
			{deleting}
			editing={editmode}
			saving={saving}
			editbody={editbody}
			editvalues={editvalues}
			editoverall={editoverall}
			onvote={(v) => handlevote(v)}
			onflag={() => { if (canflag && !flagged) flagopen = true; }}
			oneditstart={startedit}
			oneditvalue={seteditvalue}
			oneditoverall={(v) => (editoverall = v)}
			oneditbody={(v) => (editbody = v)}
			onsaved={handlesave}
			ondelete={handledelete}
			onclose={() => { open = false; editmode = false; saving = false; }}
		/>
	{/if}
{/if}

{#if flagopen}
	<FlagModal
		submitting={flagsending}
		onsubmit={handleflag}
		onclose={() => (flagopen = false)}
	/>
{/if}
