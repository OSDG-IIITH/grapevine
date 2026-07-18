<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { getFacultyMember, getAdvisorReviews, getOfferingReviews, getLabs, updateFaculty, deleteFaculty, submitReport, getLegacyAdvisorReviews, getExternalAdvisorReviews, deleteExternalAdvisorReview, editExternalAdvisorReview, voteExternalAdvisorReview, unvoteExternalAdvisorReview } from '$lib/api';
	import { toast } from 'svelte-sonner';
	import type { FacultyDetail, AdvisorReview, CourseReview, LabLean, LegacyAdvisorReview, ExternalAdvisorReview } from '$lib/types';
	import { ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS, COURSE_AXIS_ORDER, COURSE_AXIS_LABELS } from '$lib/types';
	import { currentUser } from '$lib/stores';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import RatingsBlock from '$lib/components/RatingsBlock.svelte';
	import Tabs from '$lib/components/Tabs.svelte';
	import ReviewCard from '$lib/components/ReviewCard.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import ReportModal from '$lib/components/ReportModal.svelte';
	import ExternalReviewModal from '$lib/components/ExternalReviewModal.svelte';
	import IconPencil from '@tabler/icons-svelte/icons/pencil';
	import IconTrash from '@tabler/icons-svelte/icons/trash';

	const slug = $derived($page.params.slug);

	let faculty = $state<FacultyDetail | null>(null);
	let advisorreviews = $state<AdvisorReview[]>([]);
	let legacyadvisorreviews = $state<LegacyAdvisorReview[]>([]);
	let externaladvisorreviews = $state<ExternalAdvisorReview[]>([]);
	let externalmodalopen = $state(false);
	let instructorreviews = $state<(CourseReview & { offeringcode: string; coursecode: string })[]>([]);
	let tab = $state('advisor');
	let error = $state('');
	let reportopen = $state(false);
	let reportsubmitting = $state(false);

	async function sendReport(reason: string) {
		if (!faculty) return;
		reportsubmitting = true;
		const ok = await submitReport('faculty', faculty.id, reason);
		reportsubmitting = false;
		if (ok) {
			toast.success('Report sent to the moderators.');
			reportopen = false;
		}
	}

	let editing = $state(false);
	let saving = $state(false);
	let editname = $state('');
	let editslug = $state('');
	let editbio = $state('');
	let editlabs = $state<{ id: string; name: string; short: string }[]>([]);
	let alllabs = $state<LabLean[]>([]);
	let labsloaded = $state(false);
	let slugmanual = $state(false);

	$effect(() => {
		const s = slug;
		if (!s) return;
		faculty = null;
		advisorreviews = [];
		legacyadvisorreviews = [];
		externaladvisorreviews = [];
		instructorreviews = [];
		reportopen = false;
		tab = 'advisor';
		error = '';
		editing = false;

		getFacultyMember(s)
			.then((f) => {
				if (!f) { error = 'Faculty member not found.'; return []; }
				faculty = f;
				return Promise.all(
					f.offerings.map((o) =>
						getOfferingReviews(o.id).then((reviews) =>
							(reviews ?? []).map((r) => ({ ...r, offeringcode: o.code, coursecode: o.course.code }))
						)
					)
				);
			})
			.then((all) => { if (all) instructorreviews = all.flat(); });

		getAdvisorReviews(s).then((r) => { if (r) advisorreviews = r; });
		getLegacyAdvisorReviews(s).then((r) => { if (r) legacyadvisorreviews = r; });
		getExternalAdvisorReviews(s).then((r) => { if (r) externaladvisorreviews = r; });
	});

	async function startEdit() {
		if (!faculty) return;
		editname = faculty.name;
		editslug = faculty.slug;
		editbio = faculty.bio;
		editlabs = faculty.labs.map((l) => ({ id: l.id, name: l.name, short: l.short }));
		slugmanual = false;
		if (!labsloaded) {
			const all = await getLabs();
			alllabs = all ?? [];
			labsloaded = true;
		}
		editing = true;
	}

	function cancelEdit() { editing = false; }

	function slugify(s: string) {
		return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
	}

	function onnameinput() {
		if (!slugmanual) editslug = slugify(editname);
	}

	function removeLab(id: string) {
		editlabs = editlabs.filter((l) => l.id !== id);
	}

	function addLab(labid: string) {
		const lab = alllabs.find((l) => l.id === labid);
		if (!lab || editlabs.find((l) => l.id === labid)) return;
		editlabs = [...editlabs, { id: lab.id, name: lab.name, short: lab.short }];
	}

	async function saveEdit() {
		if (!faculty) return;
		saving = true;
		const updated = await updateFaculty(faculty.slug, {
			name: editname,
			slug: editslug,
			bio: editbio,
			lab_ids: editlabs.map((l) => l.id)
		});
		saving = false;
		if (updated) {
			faculty = updated;
			editing = false;
			if (updated.slug !== slug) {
				goto(`${base}/faculty/${updated.slug}`, { replaceState: true });
			}
		}
	}

	const axes = $derived((() => {
		if (!advisorreviews.length) return { research: 0, availability: 0, mentorship: 0, support: 0, workload: 0 };
		const s = { research: 0, availability: 0, mentorship: 0, support: 0, workload: 0 };
		for (const r of advisorreviews) {
			s.research += r.research; s.availability += r.availability;
			s.mentorship += r.mentorship; s.support += r.support; s.workload += r.workload;
		}
		const n = advisorreviews.length;
		return { research: s.research/n, availability: s.availability/n, mentorship: s.mentorship/n, support: s.support/n, workload: s.workload/n };
	})());

	const tabs = $derived([
		{ id: 'advisor', label: 'As Advisor', count: advisorreviews.length + legacyadvisorreviews.length },
		{ id: 'instructor', label: 'As Instructor', count: instructorreviews.length },
		...($currentUser?.is_admin || externaladvisorreviews.length > 0 ? [{ id: 'external', label: 'External', count: externaladvisorreviews.length }] : [])
	]);

	const PER_PAGE = 10;
	let reviewpage = $state(1);
	const showncount = $derived(tab === 'advisor' ? advisorreviews.length + legacyadvisorreviews.length : tab === 'external' ? externaladvisorreviews.length : instructorreviews.length);
	const reviewpages = $derived(Math.max(1, Math.ceil(showncount / PER_PAGE)));

	const taughtcourses = $derived((() => {
		if (!faculty) return [];
		const seen = new Map<string, { code: string; name: string }>();
		for (const o of faculty.offerings) {
			if (!seen.has(o.course.id)) seen.set(o.course.id, { code: o.course.code, name: o.course.name });
		}
		return Array.from(seen.values());
	})());

	const availablelabs = $derived(alllabs.filter((l) => !editlabs.find((e) => e.id === l.id)));

	let confirmdel = $state(false);
	let deleting = $state(false);
	let confirmdelext = $state(false);
	let delextid = $state('');
	let editingextadv = $state<Record<string, { body: string; source_note: string }>>({});

	async function saveeditadvisor(id: string) {
		const data = editingextadv[id];
		if (!data) return;
		const updated = await editExternalAdvisorReview(id, { body: data.body, source_note: data.source_note || undefined });
		if (updated) {
			externaladvisorreviews = externaladvisorreviews.map(e => e.id === id ? updated : e);
			delete editingextadv[id];
		}
	}

	async function handlevoteadvisor(id: string, newvote: 0 | 1 | -1) {
		if (!$currentUser) return;
		const r = externaladvisorreviews.find(e => e.id === id);
		if (!r) return;
		const prev = (r.user_vote ?? 0) as 0 | 1 | -1;
		const ok = newvote === 0 ? await unvoteExternalAdvisorReview(id) : await voteExternalAdvisorReview(id, newvote as 1 | -1);
		if (ok) externaladvisorreviews = externaladvisorreviews.map(e => e.id === id ? {
			...e, user_vote: newvote || null,
			upvotes: e.upvotes + (newvote === 1 ? 1 : 0) - (prev === 1 ? 1 : 0),
			downvotes: e.downvotes + (newvote === -1 ? 1 : 0) - (prev === -1 ? 1 : 0),
			score: e.score + newvote - prev
		} : e);
	}

	async function doDelete() {
		if (!faculty) return;
		deleting = true;
		const ok = await deleteFaculty(faculty.slug);
		deleting = false;
		if (ok) {
			toast.success('Faculty member removed.');
			goto(`${base}/faculty`);
		}
	}
</script>

<svelte:head>
	<title>{faculty ? `${faculty.name} · grapevine` : 'Faculty · grapevine'}</title>
</svelte:head>

<div class="mx-auto w-full max-w-[1180px] px-4 pb-[120px] pt-10 sm:px-8" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	{#if error}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">{error}</div>

	{:else if faculty}
		<Crumbs items={[
			{ label: 'grapevine', href: '/' },
			{ label: 'faculty', href: '/faculty' },
			{ label: faculty.slug }
		]} />

		<!-- page head -->
		<div class="flex flex-wrap items-start justify-between gap-6">
			<div class="min-w-0 flex-1">
				{#if editing}
					<input
						bind:value={editname}
						oninput={onnameinput}
						class="mb-4 w-full min-w-0 rounded-[6px] border border-[var(--border-strong)] bg-transparent px-3 py-2 text-[var(--fg)] outline-none focus:border-[var(--accent-2)]"
						style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;"
					/>
				{:else}
					<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;">
						{faculty.name}
					</h1>
				{/if}
				{#if editing}
					<div class="mb-[22px] flex flex-wrap items-center gap-[6px]">
						<input
							bind:value={editslug}
							oninput={() => { slugmanual = true; }}
							class="rounded-[5px] border border-[var(--border-strong)] bg-transparent px-2 py-[5px] text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--accent-2)]"
							style="font-family: var(--mono); min-width: 120px;"
						/>
						{#each editlabs as lab (lab.id)}
							<span class="flex items-center gap-[5px] rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-2 py-[5px] text-[12px] text-[var(--fg-2)]">
								{lab.name}
								<button
									type="button"
									onclick={() => removeLab(lab.id)}
									aria-label="Remove {lab.name}"
									class="ml-[2px] flex h-[14px] w-[14px] items-center justify-center rounded-full text-[var(--fg-4)] transition-colors hover:text-[var(--fg-2)]"
								>
									<svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true">
										<path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
									</svg>
								</button>
							</span>
						{/each}
						{#if availablelabs.length > 0}
							<select
								onchange={(e) => { addLab((e.target as HTMLSelectElement).value); (e.target as HTMLSelectElement).value = ''; }}
								class="appearance-none rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-2 py-[5px] text-[12px] text-[var(--fg-3)] outline-none"
								style="font-family: var(--mono); width: 86px;"
							>
								<option value="">+ add lab</option>
								{#each availablelabs as l (l.id)}
									<option value={l.id}>{l.name}</option>
								{/each}
							</select>
						{/if}
					</div>
				{:else}
					<div class="mb-[22px] flex flex-wrap items-center gap-[20px] text-[13px] text-[var(--fg-2)]">
						<span class="text-[11px] tracking-[0.04em] text-[var(--fg-4)]" style="font-family: var(--mono);">{faculty.title.toLowerCase()}</span>
						{#if faculty.labs.length > 0}
							{#each faculty.labs as lab (lab.id)}
								<div class="flex items-center gap-[7px]">
									<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{lab.short}</span>
									<a href="{base}/labs/{lab.short}" class="text-[var(--fg-2)] transition-colors duration-[120ms] hover:text-[var(--fg)]">{lab.name}</a>
								</div>
							{/each}
						{:else}
							<span class="text-[var(--fg-3)]">Independent</span>
						{/if}
					</div>
				{/if}
			</div>
			<div class="flex shrink-0 items-center gap-2">
				{#if editing}
					<button
						type="button"
						onclick={cancelEdit}
						class="inline-flex items-center gap-2 self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
					>
						Cancel
					</button>
					<button
						type="button"
						onclick={saveEdit}
						disabled={saving}
						class="inline-flex items-center gap-2 self-start whitespace-nowrap rounded-[7px] px-[14px] py-2 text-[13px] font-medium transition-[background,border-color] duration-[120ms] disabled:opacity-60"
						style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
					>
						{saving ? 'Saving…' : 'Save'}
					</button>
				{:else}
					{#if $currentUser?.is_admin}
						<button
							type="button"
							onclick={() => { confirmdel = true; }}
							class="inline-flex items-center gap-[6px] self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
						>
							<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
								<path d="M3 6h18M8 6V4h8v2M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
							</svg>
							Delete
						</button>
						<button
							type="button"
							onclick={startEdit}
							class="inline-flex items-center gap-[6px] self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
						>
							<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
								<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
								<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
							</svg>
							Edit
						</button>
					{/if}
					<button
						type="button"
						onclick={() => { reportopen = true; }}
						class="inline-flex items-center gap-[6px] self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
					>Report info</button>
					<a
						href="{base}/review?faculty={faculty.slug}"
						class="inline-flex items-center gap-2 self-start whitespace-nowrap rounded-[7px] px-[14px] py-2 text-[13px] font-medium transition-[background,border-color] duration-[120ms]"
						style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
					>
						<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
							<path d="M5 12h14M12 5v14" />
						</svg>
						Write a review
					</a>
				{/if}
			</div>
		</div>

		{#if editing}
			<Textarea
				bind:value={editbio}
				rows={4}
				class="mb-7 mt-[14px] w-full resize-none border-[var(--border-strong)] text-[var(--fg-2)] focus-visible:border-[var(--accent-2)] focus-visible:ring-0 dark:bg-input/10"
				style="font-size: 15px; line-height: 1.65;"
			/>
		{:else}
			<p class="mb-7 mt-[14px] max-w-[720px] leading-[1.65] text-[var(--fg-2)]" style="font-size: 15px; text-wrap: pretty;">
				{faculty.bio}
			</p>
		{/if}

		<RatingsBlock
			overall={faculty.overall}
			{axes}
			axisorder={[...ADVISOR_AXIS_ORDER]}
			axislabels={ADVISOR_AXIS_LABELS}
			reviewcount={advisorreviews.length}
			bar="continuous"
		/>

		<Tabs items={tabs} active={tab} mono={false} onchange={(id) => { tab = id; reviewpage = 1; }} />

		{#if tab === 'advisor'}
			{#if advisorreviews.length === 0 && legacyadvisorreviews.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
					No advisor reviews yet.
				</div>
			{:else}
				{@const alladvisor = [...advisorreviews, ...legacyadvisorreviews]}
				<div class="grid grid-cols-1 gap-[12px] md:grid-cols-2">
					{#each alladvisor.slice((reviewpage - 1) * PER_PAGE, reviewpage * PER_PAGE) as r (r.id)}
						<ReviewCard
							review={r}
							axisorder={[...ADVISOR_AXIS_ORDER]}
							axislabels={ADVISOR_AXIS_LABELS}
							ondelete={(id) => (advisorreviews = advisorreviews.filter((item) => item.id !== id))}
							onedit={(updated) => (advisorreviews = advisorreviews.map((item) => item.id === updated.id ? updated as AdvisorReview : item))}
						/>
					{/each}
				</div>
			{/if}
		{:else if tab === 'external'}
			{#if $currentUser?.is_admin}
				<div class="mb-3 flex justify-end">
					<button
						type="button"
						onclick={() => externalmodalopen = true}
						class="inline-flex items-center gap-[6px] rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[14px] py-[7px] text-[12px] font-medium text-[var(--accent-2)] transition-colors hover:bg-[rgba(107,143,111,0.14)]"
					>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 5v14M5 12h14"/></svg>
						Add review
					</button>
				</div>
			{/if}
			{#if externaladvisorreviews.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">No external reviews yet.</div>
			{:else}
				<div class="grid grid-cols-1 gap-[12px] md:grid-cols-2">
					{#each externaladvisorreviews as r (r.id)}
						{@const fmtdate = new Date(r.created_at).toLocaleDateString('en-US', { month: 'short', year: 'numeric' })}
						{@const editing = !!editingextadv[r.id]}
						<div class="group relative flex flex-col gap-[10px] overflow-hidden rounded-[10px] border border-[var(--border)] p-[14px_16px] transition-[background,border-color] duration-[160ms]"
							style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.03), transparent 42%);">
							<div class="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-[180ms] group-hover:opacity-100"
								style="background: radial-gradient(ellipse 220px 110px at 100% 0%, rgba(107,143,111,0.05), transparent 70%);"></div>
							<div class="flex min-w-0 items-center gap-2">
								<div class="flex min-w-0 flex-wrap items-center gap-[5px] text-[12px] text-[var(--fg-3)]">
									<span class="text-[13px] font-medium text-[var(--fg-2)]">External Review</span>
									<span class="text-[var(--fg-4)]">·</span>
									<span class="text-[11px] tracking-[0.04em]" style="font-family: var(--mono);">{fmtdate}</span>
								</div>
							</div>
							{#if editing}
								<textarea
									bind:value={editingextadv[r.id].body}
									rows={4}
									class="w-full resize-none rounded-[7px] border border-[var(--border-strong)] bg-transparent px-3 py-2 text-[13px] leading-[1.55] text-[var(--fg-2)] outline-none focus:border-[var(--accent-2)]"
								></textarea>
								<input
									bind:value={editingextadv[r.id].source_note}
									placeholder="source note (optional)"
									class="w-full rounded-[7px] border border-[var(--border-strong)] bg-transparent px-3 py-2 text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--accent-2)]"
									style="font-family: var(--mono);"
								/>
								<div class="flex items-center gap-2">
									<button type="button" onclick={() => saveeditadvisor(r.id)}
										class="inline-flex items-center rounded-[6px] px-[12px] py-[5px] text-[12px] font-medium"
										style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.15);"
									>Save</button>
									<button type="button" onclick={() => { delete editingextadv[r.id]; }}
										class="px-2 py-1 text-[12px] text-[var(--fg-3)] transition-colors hover:text-[var(--fg)]"
									>Cancel</button>
								</div>
							{:else}
								<div class="flex-1 text-[13px] leading-[1.55] text-[var(--fg-2)]" style="white-space: pre-wrap; text-wrap: pretty;">{r.body}</div>
								<div class="mt-auto flex items-center gap-[2px]">
									<button type="button" aria-label="Upvote"
										onclick={() => handlevoteadvisor(r.id, r.user_vote === 1 ? 0 : 1)}
										class="relative z-[2] inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {r.user_vote === 1 ? 'text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
										style="font-family: var(--mono);"
									>
										<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
										<span class="min-w-[14px] text-left">{r.upvotes}</span>
									</button>
									<button type="button" aria-label="Downvote"
										onclick={() => handlevoteadvisor(r.id, r.user_vote === -1 ? 0 : -1)}
										class="relative z-[2] inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {r.user_vote === -1 ? 'text-[var(--danger)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
										style="font-family: var(--mono);"
									>
										<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14M19 12l-7 7-7-7"/></svg>
										<span class="min-w-[14px] text-left">{r.downvotes}</span>
									</button>
									{#if r.added_by_display_name}
										<button type="button" aria-label="Edit external review"
											onclick={() => { editingextadv[r.id] = { body: r.body, source_note: r.source_note ?? '' }; }}
											class="relative z-[2] ml-auto inline-flex h-7 w-7 items-center justify-center rounded-[5px] transition-colors duration-[120ms] text-[var(--fg-3)] hover:text-[var(--fg)]"
										>
											<IconPencil size={13} stroke={1.7} />
										</button>
										<button type="button" aria-label="Delete external review"
											onclick={() => { delextid = r.id; confirmdelext = true; }}
											class="relative z-[2] inline-flex h-7 w-7 items-center justify-center rounded-[5px] transition-colors duration-[120ms] text-[var(--fg-3)] hover:text-[var(--danger)]"
										>
											<IconTrash size={13} stroke={1.7} />
										</button>
									{/if}
								</div>
								{#if r.added_by_display_name}
									<div class="flex flex-wrap items-center gap-[6px]">
										{#if r.source_note}<span class="rounded-[4px] border border-[var(--border)] px-[6px] py-[1px] text-[11px] text-[var(--fg-3)]" style="font-family: var(--mono);">{r.source_note}</span>{/if}
										<span class="rounded-[4px] border border-[var(--border)] px-[6px] py-[1px] text-[11px] text-[var(--fg-3)]">by {r.added_by_display_name}</span>
									</div>
								{/if}
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		{:else}
			{#if taughtcourses.length}
				<div class="mb-[18px] flex flex-wrap items-center gap-[14px_18px] rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[18px] py-[14px] text-[13px]">
					<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Teaches</span>
					<div class="flex flex-wrap items-center gap-[10px_16px]">
						{#each taughtcourses as c (`${c.code}-${c.name}`)}
							<a
								href={`${base}/courses/${encodeURIComponent(c.code)}`}
								class="flex items-center gap-[8px] text-[var(--fg-2)] transition-colors duration-[120ms] hover:text-[var(--accent-2)]"
							>
								<span class="rounded-[5px] border border-[var(--border-strong)] px-[7px] py-[2px] text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{c.code}</span>
								<span class="text-[var(--fg-2)]">{c.name}</span>
							</a>
						{/each}
					</div>
				</div>
			{/if}
			{#if instructorreviews.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
					No instructor reviews yet.
				</div>
			{:else}
				<div class="grid grid-cols-1 gap-[12px] md:grid-cols-2">
					{#each instructorreviews.slice((reviewpage - 1) * PER_PAGE, reviewpage * PER_PAGE) as r (`${r.id}-${r.offeringcode}`)}
						<ReviewCard
							review={r}
							axisorder={[...COURSE_AXIS_ORDER]}
							axislabels={COURSE_AXIS_LABELS}
							showoffering={true}
							offeringcode={r.offeringcode}
							coursecode={r.coursecode}
							ondelete={(id) => (instructorreviews = instructorreviews.filter((item) => item.id !== id))}
							onedit={(updated) => (instructorreviews = instructorreviews.map((item) => item.id === updated.id ? { ...item, ...updated } : item))}
						/>
					{/each}
				</div>
			{/if}
		{/if}

		<Pager page={reviewpage} totalpages={reviewpages} totalitems={showncount} onchange={(p) => (reviewpage = p)} />

	{:else}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
	{/if}
</div>

{#if reportopen && faculty}
	<ReportModal target={faculty.name} submitting={reportsubmitting} onclose={() => { if (!reportsubmitting) reportopen = false; }} onsubmit={sendReport} />
{/if}

{#if externalmodalopen && faculty}
	<ExternalReviewModal
		facultyslug={faculty.slug}
		oncreate={(r) => { externaladvisorreviews = [r as ExternalAdvisorReview, ...externaladvisorreviews]; externalmodalopen = false; }}
		onclose={() => { externalmodalopen = false; }}
	/>
{/if}

{#if confirmdel}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true">
		<div class="w-full max-w-[340px] rounded-[12px] border border-[var(--border)] bg-[var(--bg-2)] p-6 shadow-xl">
			<p class="mb-1 text-[15px] font-medium text-[var(--fg)]">Delete faculty member?</p>
			<p class="mb-5 text-[13px] text-[var(--fg-3)]">They will be hidden from all listings. Reviews are preserved and can be restored later.</p>
			<div class="flex justify-end gap-2">
				<button
					type="button"
					onclick={() => { confirmdel = false; }}
					class="rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-4)]"
				>Cancel</button>
				<button
					type="button"
					onclick={doDelete}
					disabled={deleting}
					class="rounded-[7px] px-[14px] py-2 text-[13px] font-medium text-white transition-colors disabled:opacity-60"
					style="background: var(--danger);"
				>{deleting ? 'Deleting…' : 'Delete'}</button>
			</div>
		</div>
	</div>
{/if}

{#if confirmdelext}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true">
		<div class="w-full max-w-[340px] rounded-[12px] border border-[var(--border)] bg-[var(--bg-2)] p-6 shadow-xl">
			<p class="mb-1 text-[15px] font-medium text-[var(--fg)]">Delete external review?</p>
			<p class="mb-5 text-[13px] text-[var(--fg-3)]">This action cannot be undone.</p>
			<div class="flex justify-end gap-2">
				<button type="button" onclick={() => { confirmdelext = false; delextid = ''; }} class="rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-4)]">Cancel</button>
				<button type="button" onclick={async () => {
					if (await deleteExternalAdvisorReview(delextid)) externaladvisorreviews = externaladvisorreviews.filter(e => e.id !== delextid);
					confirmdelext = false; delextid = '';
				}} class="rounded-[7px] px-[14px] py-2 text-[13px] font-medium text-white" style="background: var(--danger);">Delete</button>
			</div>
		</div>
	</div>
{/if}
