<script lang="ts">
	import { page } from '$app/stores';
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { getCourse, getCourseReviews, getCourses, updateCourse, getFaculty, createOffering, deleteOffering, updateOfferingFaculty, getProposedReviews, proposeOffering, deleteCourse, submitReport, getLegacyCourseReviews, getExternalCourseReviews, deleteExternalCourseReview, editExternalCourseReview, voteExternalCourseReview, unvoteExternalCourseReview } from '$lib/api';
	import type { CourseDetail, CourseReview, Offering, FacultyLean, CourseLean, CourseRef, CourseReportSubmission, LegacyCourseReview, ExternalCourseReview } from '$lib/types';
	import { COURSE_AXIS_ORDER, COURSE_AXIS_LABELS } from '$lib/types';
	import { currentUser } from '$lib/stores';
	import { toast } from 'svelte-sonner';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import RatingsBlock from '$lib/components/RatingsBlock.svelte';
	import Tabs from '$lib/components/Tabs.svelte';
	import ReviewCard from '$lib/components/ReviewCard.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import CourseReportModal from '$lib/components/CourseReportModal.svelte';
	import ExternalReviewModal from '$lib/components/ExternalReviewModal.svelte';
	import { rendermarkdown } from '$lib/markdown';
	import IconPencil from '@tabler/icons-svelte/icons/pencil';
	import IconTrash from '@tabler/icons-svelte/icons/trash';

	const code = $derived($page.params.code);

	let course = $state<CourseDetail | null>(null);
	let reviews = $state<CourseReview[]>([]);
	let proposedReviews = $state<CourseReview[]>([]);
	let legacyreviews = $state<LegacyCourseReview[]>([]);
	let externalreviews = $state<ExternalCourseReview[]>([]);
	let externalmodalopen = $state(false);
	let tab = $state('all');
	let error = $state('');
	let reportopen = $state(false);
	let reportsubmitting = $state(false);

	async function submitCourseReport(submission: CourseReportSubmission) {
		if (!course) return;
		reportsubmitting = true;
		let ok = false;
		if (submission.kind === 'course') {
			ok = await submitReport('course', course.id, submission.reason);
		} else {
			ok = await submitReport('offering', submission.offering_id, submission.reason, submission.faculty_ids);
		}
		reportsubmitting = false;
		if (ok) {
			toast.success('Report sent to the moderators.');
			reportopen = false;
		}
	}

	function reportCourseInfo() {
		if (!course) return;
		reportopen = true;
	}

	let editing = $state(false);
	let saving = $state(false);
	let editname = $state('');
	let editcode = $state('');
	let editdesc = $state('');
	let editshortnames = $state<string[]>([]);
	let shortnamesinput = $state('');
	let editofferings = $state<Offering[]>([]);
	let editpredecessors = $state<CourseRef[]>([]);
	let editsuccessors = $state<CourseRef[]>([]);
	let allfaculty = $state<FacultyLean[]>([]);
	let facultyloaded = $state(false);
	let allcourses = $state<CourseLean[]>([]);
	let coursesloaded = $state(false);
	let creating = $state(false);
	let crseason = $state('M');
	let cryear = $state('');
	let cryearerror = $state(false);

	let confirmdel = $state(false);
	let deleting = $state(false);
	let confirmdelext = $state(false);
	let delextid = $state('');
	let editingext = $state<Record<string, { body: string; source_note: string }>>({});

	async function saveeditcourse(id: string) {
		const data = editingext[id];
		if (!data) return;
		const updated = await editExternalCourseReview(id, { body: data.body, source_note: data.source_note || undefined });
		if (updated) {
			externalreviews = externalreviews.map(e => e.id === id ? updated : e);
			delete editingext[id];
		}
	}

	async function doDelete() {
		if (!course) return;
		deleting = true;
		const ok = await deleteCourse(course.code);
		deleting = false;
		if (ok) {
			toast.success('Course removed.');
			goto(`${base}/courses`);
		}
	}

	let proposing = $state(false);
	let proposeSeason = $state('M');
	let proposeYear = $state('');
	let proposeYearError = $state(false);
	let proposeFaculty = $state<FacultyLean[]>([]);

	function removeProposeFaculty(id: string) {
		proposeFaculty = proposeFaculty.filter((member) => member.id !== id);
	}

	function addProposeFaculty(id: string) {
		const member = allfaculty.find((faculty) => faculty.id === id);
		if (member && !proposeFaculty.some((faculty) => faculty.id === id)) {
			proposeFaculty = [...proposeFaculty, member];
		}
	}

	const availableProposeFaculty = $derived(
		allfaculty.filter((faculty) => !proposeFaculty.some((selected) => selected.id === faculty.id))
	);

	async function submitProposal() {
		if (!course) return;
		const year = parseyear(proposeYear);
		if (year === null) {
			proposeYearError = true;
			return;
		}
		proposeYearError = false;
		const result = await proposeOffering(course.code, proposeSeason, year, proposeFaculty.map((faculty) => faculty.id));
		if (result) {
			toast.success('Semester proposal submitted to moderators.');
			proposing = false;
			proposeYear = '';
			proposeFaculty = [];
			const updated = await getCourse(course.code);
			if (updated) course = updated;
		}
	}

	$effect(() => {
		const c = code;
		if (!c) return;
		error = '';
		course = null;
		reviews = [];
		proposedReviews = [];
		legacyreviews = [];
		externalreviews = [];
		reportopen = false;
		proposing = false;
		tab = 'all';
		editing = false;

		const decoded = decodeURIComponent(c);
		Promise.all([getCourse(decoded), getCourseReviews(decoded), getProposedReviews(decoded), getLegacyCourseReviews(decoded), getExternalCourseReviews(decoded)])
			.then(([d, r, pr, lr, er]) => {
				if (!d) { error = 'Course not found.'; return; }
				course = d;
				reviews = r ?? [];
				proposedReviews = pr ?? [];
				legacyreviews = lr ?? [];
				externalreviews = er ?? [];
			});

		if (!facultyloaded) {
			getFaculty().then((all) => {
				allfaculty = all ?? [];
				facultyloaded = true;
			});
		}
	});

	async function startEdit() {
		if (!course) return;
		editname = course.name;
		editcode = course.code;
		editdesc = course.description;
		editshortnames = [...(course.shortnames ?? [])];
		shortnamesinput = '';
		editofferings = course.offerings.map((o) => ({ ...o, faculty: [...o.faculty] }));
		editpredecessors = [...course.predecessors];
		editsuccessors = [...course.successors];
		editing = true;
		const loads: Promise<unknown>[] = [];
		if (!facultyloaded) loads.push(getFaculty().then((a) => { allfaculty = a ?? []; facultyloaded = true; }));
		if (!coursesloaded) loads.push(getCourses().then((a) => { allcourses = a ?? []; coursesloaded = true; }));
		await Promise.all(loads);
	}

	function cancelEdit() {
		editing = false;
		creating = false;
		cryear = '';
		cryearerror = false;
	}

	function parseyear(s: string): number | null {
		const t = s.trim();
		if (/^\d{2}$/.test(t)) return parseInt(t, 10);
		if (/^\d{4}$/.test(t)) { const y = parseInt(t, 10); return y >= 2000 && y <= 2099 ? y % 100 : null; }
		return null;
	}

	function fullyear(y: number): number { return y < 100 ? 2000 + y : y; }

	function availablefaculty(offeringId: string) {
		const o = editofferings.find((o) => o.id === offeringId);
		if (!o) return allfaculty;
		return allfaculty.filter((f) => !o.faculty.find((of) => of.id === f.id));
	}

	async function addInstructor(offeringId: string, facultyId: string) {
		const o = editofferings.find((o) => o.id === offeringId);
		if (!o || o.faculty.find((f) => f.id === facultyId)) return;
		const ids = [...o.faculty.map((f) => f.id), facultyId];
		const updated = await updateOfferingFaculty(offeringId, ids);
		if (updated) {
			editofferings = editofferings.map((o) => o.id === offeringId ? { ...o, faculty: updated.faculty } : o);
			if (course) course = { ...course, offerings: course.offerings.map((o) => o.id === offeringId ? { ...o, faculty: updated.faculty } : o) };
		}
	}

	async function removeInstructor(offeringId: string, facultyId: string) {
		const o = editofferings.find((o) => o.id === offeringId);
		if (!o) return;
		const ids = o.faculty.filter((f) => f.id !== facultyId).map((f) => f.id);
		const updated = await updateOfferingFaculty(offeringId, ids);
		if (updated) {
			editofferings = editofferings.map((o) => o.id === offeringId ? { ...o, faculty: updated.faculty } : o);
			if (course) course = { ...course, offerings: course.offerings.map((o) => o.id === offeringId ? { ...o, faculty: updated.faculty } : o) };
		}
	}

	async function removeOffering(offeringId: string) {
		const ok = await deleteOffering(offeringId);
		if (ok) {
			editofferings = editofferings.filter((o) => o.id !== offeringId);
			if (course) course = { ...course, offerings: course.offerings.filter((o) => o.id !== offeringId) };
		}
	}

	async function addOffering() {
		if (!course) return;
		const year = parseyear(cryear);
		if (year === null) { cryearerror = true; return; }
		cryearerror = false;
		const added = await createOffering(course.code, { season: crseason, year });
		if (added) {
			editofferings = [...editofferings, added];
			if (course) course = { ...course, offerings: [...course.offerings, added] };
			creating = false;
			cryear = '';
			crseason = 'M';
		}
	}

	async function saveEdit() {
		if (!course) return;
		saving = true;
		const updated = await updateCourse(course.code, {
			code: editcode,
			name: editname,
			description: editdesc,
			predecessor_ids: editpredecessors.map((c) => c.id),
			successor_ids: editsuccessors.map((c) => c.id),
			shortnames: editshortnames,
		});
		saving = false;
		if (updated) {
			course = updated;
			editing = false;
			if (updated.code !== code) {
				goto(`${base}/courses/${encodeURIComponent(updated.code)}`, { replaceState: true });
			}
		}
	}

	const availablepredecessors = $derived(
		allcourses.filter((c) => c.id !== course?.id && !editpredecessors.some((p) => p.id === c.id))
	);
	const availablesuccessors = $derived(
		allcourses.filter((c) => c.id !== course?.id && !editsuccessors.some((s) => s.id === c.id))
	);

	let predpick = $state('');
	let succpick = $state('');

	const axes = $derived((() => {
		if (!reviews.length) return { difficulty: 0, workload: 0, teaching: 0, grading: 0, content: 0 };
		const s = { difficulty: 0, workload: 0, teaching: 0, grading: 0, content: 0 };
		for (const r of reviews) {
			s.difficulty += r.difficulty; s.workload += r.workload;
			s.teaching += r.teaching; s.grading += r.grading; s.content += r.content;
		}
		const n = reviews.length;
		return { difficulty: s.difficulty/n, workload: s.workload/n, teaching: s.teaching/n, grading: s.grading/n, content: s.content/n };
	})());

	const offeringmap = $derived(
		course ? Object.fromEntries([
			...course.offerings.map((o) => [o.id, o.code]),
			...course.proposed_offerings.map((o) => [o.id, `${o.season === 'M' ? 'Monsoon' : 'Spring'} ${fullyear(o.year)}`])
		]) : {}
	);

	async function handlevotecourse(id: string, newvote: 0 | 1 | -1) {
		if (!$currentUser) return;
		const r = externalreviews.find(e => e.id === id);
		if (!r) return;
		const prev = (r.user_vote ?? 0) as 0 | 1 | -1;
		const ok = newvote === 0 ? await unvoteExternalCourseReview(id) : await voteExternalCourseReview(id, newvote as 1 | -1);
		if (ok) externalreviews = externalreviews.map(e => e.id === id ? {
			...e, user_vote: newvote || null,
			upvotes: e.upvotes + (newvote === 1 ? 1 : 0) - (prev === 1 ? 1 : 0),
			downvotes: e.downvotes + (newvote === -1 ? 1 : 0) - (prev === -1 ? 1 : 0),
			score: e.score + newvote - prev
		} : e);
	}

	const tabs = $derived(
		course ? [
			{ id: 'all', label: 'All', count: reviews.length + proposedReviews.length + legacyreviews.length + externalreviews.length },
			...course.offerings.map((o) => ({
				id: o.id,
				label: o.code,
				count: reviews.filter((r) => r.offering_id === o.id).length
			})),
			...(course.proposed_offerings.length > 0 || proposedReviews.length > 0 || legacyreviews.length > 0 ? [
				{ id: 'other', label: 'Other', count: proposedReviews.length + legacyreviews.length }
			] : []),
			...($currentUser?.is_admin || externalreviews.length > 0 ? [
				{ id: 'external', label: 'External', count: externalreviews.length }
			] : [])
		] : []
	);

	const shown = $derived(
		tab === 'all' ? [...reviews, ...proposedReviews, ...legacyreviews].sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()) :
		tab === 'other' ? [...proposedReviews, ...legacyreviews] :
		tab === 'external' ? [] :
		reviews.filter((r) => r.offering_id === tab)
	);
	const selectedoffering = $derived(course && tab !== 'all' && tab !== 'other' ? course.offerings.find((o) => o.id === tab) : null);

	const PER_PAGE = 10;
	let reviewpage = $state(1);
	const reviewpages = $derived(Math.max(1, Math.ceil(shown.length / PER_PAGE)));
	const visiblereviews = $derived(shown.slice((reviewpage - 1) * PER_PAGE, reviewpage * PER_PAGE));
</script>

<svelte:head>
	<title>{course ? `${course.name} · grapevine` : 'Course · grapevine'}</title>
</svelte:head>

<div class="mx-auto w-full max-w-[1180px] px-4 pb-[120px] pt-10 sm:px-8" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	{#if error}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">{error}</div>

	{:else if course}
		<Crumbs items={[{ label: 'grapevine', href: '/' }, { label: 'courses', href: '/courses' }, { label: course.code }]} />

		<!-- page head -->
		<div class="flex flex-wrap items-start justify-between gap-6">
			<div class="min-w-0 flex-1">
				{#if editing}
					<input
						bind:value={editname}
						class="mb-4 w-full min-w-0 rounded-[6px] border border-[var(--border-strong)] bg-transparent px-3 py-2 text-[var(--fg)] outline-none focus:border-[var(--accent-2)]"
						style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;"
					/>
				{:else}
					<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;">
						{course.name}
					</h1>
				{/if}
				{#if editing}
					<div class="mb-[22px] flex flex-wrap items-center gap-[6px]">
						<input
							bind:value={editcode}
							class="rounded-[5px] border border-[var(--border-strong)] bg-transparent px-2 py-[5px] text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--accent-2)]"
							style="font-family: var(--mono); min-width: 100px;"
						/>
					</div>
				{:else}
					<div class="mb-[22px] flex flex-wrap items-center gap-[8px] text-[13px] text-[var(--fg-2)]">
						<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{course.code}</span>
						{#each course.shortnames ?? [] as sn (sn)}
							<span class="rounded-[5px] border border-[var(--border)] px-2 py-[3px] text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">{sn}</span>
						{/each}
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
						onclick={reportCourseInfo}
						class="inline-flex items-center gap-[6px] self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
					>Report course info</button>
					<a
						href="{base}/review?course={course.code}"
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
			<!-- shortnames -->
			<div class="mb-4 mt-[14px] flex flex-wrap items-center gap-[6px]">
				<span class="mr-1 shrink-0 text-[12px] text-[var(--fg-3)]">Aliases</span>
				{#each editshortnames as sn (sn)}
					<span class="flex items-center gap-[4px] rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[8px] py-[4px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">
						{sn}
						<button
							type="button"
							onclick={() => { editshortnames = editshortnames.filter((x) => x !== sn); }}
							aria-label="Remove {sn}"
							class="ml-[2px] flex h-[14px] w-[14px] items-center justify-center rounded-full text-[var(--fg-4)] transition-colors hover:text-[var(--fg)]"
						>
							<svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true">
								<path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
							</svg>
						</button>
					</span>
				{/each}
				<input
					bind:value={shortnamesinput}
					placeholder="+ alias"
					onkeydown={(e) => {
						if ((e.key === 'Enter' || e.key === ',') && shortnamesinput.trim()) {
							e.preventDefault();
							const v = shortnamesinput.trim().toUpperCase();
							if (!editshortnames.includes(v)) editshortnames = [...editshortnames, v];
							shortnamesinput = '';
						}
					}}
					class="rounded-[5px] border border-dashed border-[var(--border-strong)] bg-transparent px-[8px] py-[4px] text-[12px] text-[var(--fg-4)] outline-none transition-colors hover:border-[var(--fg-4)] focus:border-[var(--accent-2)] focus:text-[var(--fg-2)]"
					style="font-family: var(--mono); width: 90px;"
				/>
			</div>
			<Textarea
				bind:value={editdesc}
				rows={4}
				class="mb-7 w-full resize-none border-[var(--border-strong)] text-[var(--fg-2)] focus-visible:border-[var(--accent-2)] focus-visible:ring-0 dark:bg-input/10"
				style="font-size: 15px; line-height: 1.65;"
			/>

			<!-- offerings editor -->
			<div class="mb-7 overflow-hidden rounded-[10px] border border-[var(--border)]" style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107, 143, 111, 0.04), transparent 48%);">

				<!-- header -->
				<div class="flex items-center justify-between border-b border-[var(--border)] px-5 py-[13px]">
					<div class="flex items-center gap-[10px]">
						<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Offerings</span>
						{#if editofferings.length > 0}
							<span class="rounded-full bg-[var(--bg-3)] px-[7px] py-[1px] text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{editofferings.length}</span>
						{/if}
					</div>
					{#if !creating}
						<button
							type="button"
							onclick={() => { creating = true; }}
							class="inline-flex items-center gap-[5px] rounded-[6px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[10px] py-[5px] text-[12px] font-medium text-[var(--fg-2)] transition-colors duration-[120ms] hover:bg-[var(--bg-4)] hover:text-[var(--fg)]"
						>
							<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
								<path d="M5 1v8M1 5h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
							</svg>
							New offering
						</button>
					{/if}
				</div>

				<!-- offering rows -->
				{#if editofferings.length === 0 && !creating}
					<div class="px-5 py-[44px] text-center text-[13px] text-[var(--fg-4)]">No offerings yet.</div>
				{:else}
					{#each editofferings as o (o.id)}
						<div class="flex items-center gap-[12px] border-b border-[var(--border)] px-5 py-[13px]">
							<!-- season + year -->
							<div class="w-[118px] shrink-0">
								<span class="text-[13px] font-medium text-[var(--fg)]">{o.season === 'M' ? 'Monsoon' : 'Spring'} {fullyear(o.year)}</span>
							</div>
							<!-- instructors -->
							<div class="flex flex-1 flex-wrap items-center gap-[6px]">
								{#each o.faculty as f (f.id)}
									<span class="flex items-center gap-[4px] rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[8px] py-[4px] text-[12px] text-[var(--fg-2)]">
										{f.name}
										<button
											type="button"
											onclick={() => removeInstructor(o.id, f.id)}
											aria-label="Remove {f.name}"
											class="ml-[2px] flex h-[14px] w-[14px] items-center justify-center rounded-full text-[var(--fg-4)] transition-colors hover:text-[var(--fg)]"
										>
											<svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true">
												<path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
											</svg>
										</button>
									</span>
								{/each}
								{#if facultyloaded && availablefaculty(o.id).length > 0}
									<select
										onchange={(e) => { const v = (e.target as HTMLSelectElement).value; if (v) { addInstructor(o.id, v); (e.target as HTMLSelectElement).value = ''; } }}
										class="appearance-none rounded-[5px] border border-dashed border-[var(--border-strong)] bg-transparent px-[8px] py-[4px] text-[12px] text-[var(--fg-4)] outline-none transition-colors hover:border-[var(--fg-4)] hover:text-[var(--fg-3)]"
										style="font-family: var(--mono); width: 110px;"
									>
										<option value="">+ instructor</option>
										{#each availablefaculty(o.id) as f (f.id)}
											<option value={f.id}>{f.name}</option>
										{/each}
									</select>
								{/if}
							</div>
							<!-- delete -->
							<button
								type="button"
								onclick={() => removeOffering(o.id)}
								aria-label="Delete offering"
								class="flex h-[28px] w-[28px] shrink-0 items-center justify-center rounded-[6px] text-[var(--fg-4)] transition-colors duration-[120ms] hover:bg-[var(--danger-bg)] hover:text-[var(--danger)]"
							>
								<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
									<path d="M3 6h18M8 6V4h8v2M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
								</svg>
							</button>
						</div>
					{/each}

					<!-- creation row -->
					{#if creating}
						<div class="flex flex-wrap items-center gap-[8px] border-t border-[var(--border)] px-5 py-[13px]" style="background: rgba(107, 143, 111, 0.035);">
							<!-- season toggle -->
							<div class="flex rounded-[6px] border border-[var(--border-strong)] p-[2px]" style="background: var(--bg-3);">
								<button
									type="button"
									onclick={() => { crseason = 'M'; }}
									class="rounded-[4px] px-[9px] py-[3px] text-[12px] font-medium transition-colors duration-[100ms] {crseason === 'M' ? 'text-[var(--fg)]' : 'text-[var(--fg-4)] hover:text-[var(--fg-3)]'}"
									style="font-family: var(--mono); {crseason === 'M' ? 'background: var(--bg-4);' : ''}"
								>M</button>
								<button
									type="button"
									onclick={() => { crseason = 'S'; }}
									class="rounded-[4px] px-[9px] py-[3px] text-[12px] font-medium transition-colors duration-[100ms] {crseason === 'S' ? 'text-[var(--fg)]' : 'text-[var(--fg-4)] hover:text-[var(--fg-3)]'}"
									style="font-family: var(--mono); {crseason === 'S' ? 'background: var(--bg-4);' : ''}"
								>S</button>
							</div>
							<!-- year input -->
							<input
								bind:value={cryear}
								placeholder="26"
								oninput={() => { cryearerror = false; }}
								onkeydown={(e) => { if (e.key === 'Enter') addOffering(); if (e.key === 'Escape') { creating = false; cryear = ''; cryearerror = false; crseason = 'M'; } }}
								class="rounded-[5px] border bg-transparent px-[8px] py-[4px] text-[12px] outline-none transition-colors duration-[100ms] {cryearerror ? 'border-[var(--danger)] text-[var(--danger)]' : 'border-[var(--border-strong)] text-[var(--fg-2)] focus:border-[var(--accent-2)]'}"
								style="font-family: var(--mono); width: 52px;"
							/>
							{#if cryearerror}
								<span class="text-[11px] text-[var(--danger)]">Enter a valid year (e.g. 26 or 2026)</span>
							{/if}
							<div class="flex items-center gap-[6px]">
								<button
									type="button"
									onclick={addOffering}
									class="inline-flex items-center rounded-[6px] px-[12px] py-[5px] text-[12px] font-medium transition-colors duration-[120ms]"
									style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.15);"
								>Add</button>
								<button
									type="button"
									onclick={() => { creating = false; cryear = ''; cryearerror = false; crseason = 'M'; }}
									aria-label="Cancel"
									class="flex h-[28px] w-[28px] items-center justify-center rounded-[6px] border border-[var(--border-strong)] text-[var(--fg-4)] transition-colors hover:bg-[var(--bg-4)] hover:text-[var(--fg-3)]"
								>
									<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
										<path d="M1 1l9 9M10 1L1 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
									</svg>
								</button>
							</div>
						</div>
					{/if}
				{/if}
			</div>

			<!-- succession editor -->
			<div class="mb-7 overflow-hidden rounded-[10px] border border-[var(--border)]" style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107, 143, 111, 0.04), transparent 48%);">
				<div class="border-b border-[var(--border)] px-5 py-[13px]">
					<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Succession</span>
				</div>
				<!-- predecessors -->
				<div class="flex flex-wrap items-center gap-[6px] border-b border-[var(--border)] px-5 py-[13px]">
					<span class="mr-1 shrink-0 text-[12px] text-[var(--fg-3)]">Replaces</span>
					{#each editpredecessors as c (c.id)}
						<span class="flex items-center gap-[4px] rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[8px] py-[4px] text-[12px] text-[var(--fg-2)]">
							<span style="font-family: var(--mono); font-size: 11px; color: var(--fg-3);">{c.code}</span>
							<span class="ml-[3px]">{c.name}</span>
							<button
								type="button"
								onclick={() => { editpredecessors = editpredecessors.filter((x) => x.id !== c.id); }}
								aria-label="Remove {c.name}"
								class="ml-[2px] flex h-[14px] w-[14px] items-center justify-center rounded-full text-[var(--fg-4)] transition-colors hover:text-[var(--fg)]"
							>
								<svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true">
									<path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
								</svg>
							</button>
						</span>
					{/each}
					{#if availablepredecessors.length > 0}
						<Combobox
							items={availablepredecessors.map((c) => ({ value: c.id, label: c.name }))}
							bind:value={predpick}
							placeholder="+ predecessor"
							searchplaceholder="search courses…"
							popoverwidth="260px"
							onselect={(id) => {
								const c = allcourses.find((x) => x.id === id);
								if (c) editpredecessors = [...editpredecessors, { id: c.id, code: c.code, name: c.name }];
								predpick = '';
							}}
							class="rounded-[5px] border border-dashed border-[var(--border-strong)] bg-transparent px-[8px] py-[4px] text-[12px] text-[var(--fg-4)] transition-colors hover:border-[var(--fg-4)] hover:text-[var(--fg-3)]"
							style="font-family: var(--mono);"
						/>
					{/if}
				</div>
				<!-- successors -->
				<div class="flex flex-wrap items-center gap-[6px] px-5 py-[13px]">
					<span class="mr-1 shrink-0 text-[12px] text-[var(--fg-3)]">Replaced by</span>
					{#each editsuccessors as c (c.id)}
						<span class="flex items-center gap-[4px] rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[8px] py-[4px] text-[12px] text-[var(--fg-2)]">
							<span style="font-family: var(--mono); font-size: 11px; color: var(--fg-3);">{c.code}</span>
							<span class="ml-[3px]">{c.name}</span>
							<button
								type="button"
								onclick={() => { editsuccessors = editsuccessors.filter((x) => x.id !== c.id); }}
								aria-label="Remove {c.name}"
								class="ml-[2px] flex h-[14px] w-[14px] items-center justify-center rounded-full text-[var(--fg-4)] transition-colors hover:text-[var(--fg)]"
							>
								<svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true">
									<path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
								</svg>
							</button>
						</span>
					{/each}
					{#if availablesuccessors.length > 0}
						<Combobox
							items={availablesuccessors.map((c) => ({ value: c.id, label: c.name }))}
							bind:value={succpick}
							placeholder="+ successor"
							searchplaceholder="search courses…"
							popoverwidth="260px"
							onselect={(id) => {
								const c = allcourses.find((x) => x.id === id);
								if (c) editsuccessors = [...editsuccessors, { id: c.id, code: c.code, name: c.name }];
								succpick = '';
							}}
							class="rounded-[5px] border border-dashed border-[var(--border-strong)] bg-transparent px-[8px] py-[4px] text-[12px] text-[var(--fg-4)] transition-colors hover:border-[var(--fg-4)] hover:text-[var(--fg-3)]"
							style="font-family: var(--mono);"
						/>
					{/if}
				</div>
			</div>
		{:else}
			<p class="mb-7 mt-[14px] max-w-[720px] leading-[1.65] text-[var(--fg-2)]" style="font-size: 15px; text-wrap: pretty;">
				{course.description}
			</p>
			{#if course.predecessors.length > 0 || course.successors.length > 0}
				<div class="mb-7 flex flex-wrap gap-x-6 gap-y-2 text-[13px]">
					{#if course.predecessors.length > 0}
						<span class="flex flex-wrap items-center gap-[6px] text-[var(--fg-3)]">
							Replaces
							{#each course.predecessors as p (p.id)}
								<a href="{base}/courses/{p.code}" class="flex items-center gap-[5px] rounded-[5px] border border-[var(--border)] bg-[var(--bg-2)] px-[8px] py-[3px] text-[12px] text-[var(--fg-2)] transition-colors hover:text-[var(--fg)]">
									<span style="font-family: var(--mono); font-size: 11px; color: var(--fg-3);">{p.code}</span>
									<span>{p.name}</span>
								</a>
							{/each}
						</span>
					{/if}
					{#if course.successors.length > 0}
						<span class="flex flex-wrap items-center gap-[6px] text-[var(--fg-3)]">
							Replaced by
							{#each course.successors as s (s.id)}
								<a href="{base}/courses/{s.code}" class="flex items-center gap-[5px] rounded-[5px] border border-[var(--border)] bg-[var(--bg-2)] px-[8px] py-[3px] text-[12px] text-[var(--fg-2)] transition-colors hover:text-[var(--fg)]">
									<span style="font-family: var(--mono); font-size: 11px; color: var(--fg-3);">{s.code}</span>
									<span>{s.name}</span>
								</a>
							{/each}
						</span>
					{/if}
				</div>
			{/if}
		{/if}

		<RatingsBlock
			overall={course.overall}
			{axes}
			axisorder={[...COURSE_AXIS_ORDER]}
			axislabels={COURSE_AXIS_LABELS}
			reviewcount={reviews.length}
			bar="continuous"
		/>

		<Tabs
			items={tabs}
			active={tab}
			onchange={(id) => { tab = id; reviewpage = 1; }}
			onadd={() => (proposing = !proposing)}
			addtitle="Propose semester"
		/>

		{#if proposing}
			<div class="mb-6 flex flex-wrap items-center gap-[8px] rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-4 py-3" style="animation: fadeUp 200ms cubic-bezier(.2,.6,.2,1) both;">
				<span class="mr-1 text-[13px] font-medium text-[var(--fg-2)]">Propose semester:</span>
				<div class="flex rounded-[6px] border border-[var(--border-strong)] p-[2px]" style="background: var(--bg-3);">
					<button type="button" onclick={() => { proposeSeason = 'M'; }} class="rounded-[4px] px-[9px] py-[3px] text-[12px] font-medium transition-colors {proposeSeason === 'M' ? 'text-[var(--fg)]' : 'text-[var(--fg-4)] hover:text-[var(--fg-3)]'}" style="font-family: var(--mono); {proposeSeason === 'M' ? 'background: var(--bg-4);' : ''}">Monsoon</button>
					<button type="button" onclick={() => { proposeSeason = 'S'; }} class="rounded-[4px] px-[9px] py-[3px] text-[12px] font-medium transition-colors {proposeSeason === 'S' ? 'text-[var(--fg)]' : 'text-[var(--fg-4)] hover:text-[var(--fg-3)]'}" style="font-family: var(--mono); {proposeSeason === 'S' ? 'background: var(--bg-4);' : ''}">Spring</button>
				</div>
				<input
					bind:value={proposeYear}
					placeholder="2026"
					oninput={() => { proposeYearError = false; }}
					class="rounded-[5px] border bg-transparent px-[8px] py-[4px] text-[12px] outline-none {proposeYearError ? 'border-[var(--danger)] text-[var(--danger)]' : 'border-[var(--border-strong)] text-[var(--fg-2)] focus:border-[var(--accent-2)]'}"
					style="font-family: var(--mono); width: 64px;"
				/>

				{#each proposeFaculty as faculty (faculty.id)}
					<span class="flex items-center gap-[4px] rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[8px] py-[4px] text-[12px] text-[var(--fg-2)]">
						{faculty.name}
						<button type="button" onclick={() => removeProposeFaculty(faculty.id)} aria-label="Remove {faculty.name}" class="ml-[2px] flex h-[14px] w-[14px] items-center justify-center rounded-full text-[var(--fg-4)] transition-colors hover:text-[var(--fg)]">
							<svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true"><path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" /></svg>
						</button>
					</span>
				{/each}

				{#if availableProposeFaculty.length > 0}
					<select onchange={(event) => { const id = event.currentTarget.value; if (id) { addProposeFaculty(id); event.currentTarget.value = ''; } }} class="appearance-none rounded-[5px] border border-dashed border-[var(--border-strong)] bg-transparent px-[8px] py-[4px] text-[12px] text-[var(--fg-4)] outline-none hover:border-[var(--fg-4)] hover:text-[var(--fg-3)]" style="font-family: var(--mono); width: 110px;">
						<option value="">+ instructor</option>
						{#each availableProposeFaculty as faculty (faculty.id)}
							<option value={faculty.id}>{faculty.name}</option>
						{/each}
					</select>
				{/if}

				<button type="button" onclick={submitProposal} class="ml-auto inline-flex items-center rounded-[6px] px-[12px] py-[5px] text-[12px] font-medium" style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.15);">Propose</button>
				<button type="button" onclick={() => { proposing = false; proposeYear = ''; proposeYearError = false; proposeFaculty = []; }} class="px-2 py-1 text-[13px] text-[var(--fg-3)] transition-colors hover:text-[var(--fg)]">Cancel</button>
			</div>
		{/if}

		<!-- taught-by banner -->
		{#if selectedoffering}
			<div class="mb-[18px] flex flex-wrap items-center gap-[14px] rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[18px] py-[14px] text-[13px]">
				<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Taught by</span>
				{#each selectedoffering.faculty as f, i (f.id)}
					<a href="{base}/faculty/{f.slug ?? f.id}" class="text-[var(--fg)] transition-colors duration-[120ms] hover:text-[var(--accent-2)]">
						{f.name}{i < selectedoffering.faculty.length - 1 ? ',' : ''}
					</a>
				{/each}
			</div>
		{/if}

		{#snippet extcard(r: import('$lib/types').ExternalCourseReview)}
			{@const fmtdate = new Date(r.created_at).toLocaleDateString('en-US', { month: 'short', year: 'numeric' })}
			{@const editing = !!editingext[r.id]}
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
						bind:value={editingext[r.id].body}
						rows={4}
						class="w-full resize-none rounded-[7px] border border-[var(--border-strong)] bg-transparent px-3 py-2 text-[13px] leading-[1.55] text-[var(--fg-2)] outline-none focus:border-[var(--accent-2)]"
					></textarea>
					<input
						bind:value={editingext[r.id].source_note}
						placeholder="source note (optional)"
						class="w-full rounded-[7px] border border-[var(--border-strong)] bg-transparent px-3 py-2 text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--accent-2)]"
						style="font-family: var(--mono);"
					/>
					<div class="flex items-center gap-2">
						<button type="button" onclick={() => saveeditcourse(r.id)}
							class="inline-flex items-center rounded-[6px] px-[12px] py-[5px] text-[12px] font-medium"
							style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.15);"
						>Save</button>
						<button type="button" onclick={() => { delete editingext[r.id]; }}
							class="px-2 py-1 text-[12px] text-[var(--fg-3)] transition-colors hover:text-[var(--fg)]"
						>Cancel</button>
					</div>
				{:else}
					<div class="flex-1 text-[13px] leading-[1.55] text-[var(--fg-2)]" style="text-wrap: pretty;"><div class="review-prose">{@html rendermarkdown(r.body)}</div></div>
					<div class="mt-auto flex items-center gap-[2px]">
						<button type="button" aria-label="Upvote"
							onclick={() => handlevotecourse(r.id, r.user_vote === 1 ? 0 : 1)}
							class="relative z-[2] inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {r.user_vote === 1 ? 'text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
							style="font-family: var(--mono);"
						>
							<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
							<span class="min-w-[14px] text-left">{r.upvotes}</span>
						</button>
						<button type="button" aria-label="Downvote"
							onclick={() => handlevotecourse(r.id, r.user_vote === -1 ? 0 : -1)}
							class="relative z-[2] inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {r.user_vote === -1 ? 'text-[var(--danger)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
							style="font-family: var(--mono);"
						>
							<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14M19 12l-7 7-7-7"/></svg>
							<span class="min-w-[14px] text-left">{r.downvotes}</span>
						</button>
						{#if r.added_by_display_name}
							<button type="button" aria-label="Edit external review"
								onclick={() => { editingext[r.id] = { body: r.body, source_note: r.source_note ?? '' }; }}
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
		{/snippet}

		<!-- reviews -->
		{#if tab === 'external'}
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
			{#if externalreviews.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">No external reviews yet.</div>
			{:else}
				<div class="grid grid-cols-1 gap-[12px] md:grid-cols-2">
					{#each externalreviews as r (r.id)}{@render extcard(r)}{/each}
				</div>
			{/if}
		{:else if shown.length === 0 && (tab !== 'all' || externalreviews.length === 0)}
			<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
				{#if tab === 'other'}
					No reviews yet.
				{:else}
					No reviews for this offering yet.
				{/if}
			</div>
		{:else}
			<div class="grid grid-cols-1 gap-[12px] md:grid-cols-2">
				{#each visiblereviews as r (r.id)}
					<ReviewCard
						review={r}
						axisorder={[...COURSE_AXIS_ORDER]}
						axislabels={COURSE_AXIS_LABELS}
						showoffering={tab === 'all' || tab === 'other'}
						offeringcode={'offering_id' in r ? offeringmap[(r as CourseReview).offering_id] : undefined}
						ondelete={(id) => {
							reviews = reviews.filter((item) => item.id !== id);
							proposedReviews = proposedReviews.filter((item) => item.id !== id);
						}}
						onedit={(updated) => {
							reviews = reviews.map((item) => item.id === updated.id ? updated as CourseReview : item);
							proposedReviews = proposedReviews.map((item) => item.id === updated.id ? updated as CourseReview : item);
						}}
					/>
				{/each}
				{#if tab === 'all'}
					{#each externalreviews as r (r.id)}{@render extcard(r)}{/each}
				{/if}
			</div>
		{/if}

		{#if tab !== 'external'}
			<Pager page={reviewpage} totalpages={reviewpages} totalitems={shown.length} onchange={(p) => (reviewpage = p)} />
		{/if}

	{:else}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
	{/if}
</div>

{#if reportopen && course}
	<CourseReportModal {course} faculty={allfaculty} initialofferingid={selectedoffering?.id} submitting={reportsubmitting} onclose={() => { if (!reportsubmitting) reportopen = false; }} onsubmit={submitCourseReport} />
{/if}

{#if externalmodalopen && course}
	<ExternalReviewModal
		coursecode={course.code}
		oncreate={(r) => { externalreviews = [r as ExternalCourseReview, ...externalreviews]; externalmodalopen = false; }}
		onclose={() => { externalmodalopen = false; }}
	/>
{/if}

{#if confirmdel}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true">
		<div class="w-full max-w-[340px] rounded-[12px] border border-[var(--border)] bg-[var(--bg-2)] p-6 shadow-xl">
			<p class="mb-1 text-[15px] font-medium text-[var(--fg)]">Delete course?</p>
			<p class="mb-5 text-[13px] text-[var(--fg-3)]">The course will be hidden from all listings. Reviews are preserved and can be restored later.</p>
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
					if (await deleteExternalCourseReview(delextid)) externalreviews = externalreviews.filter(e => e.id !== delextid);
					confirmdelext = false; delextid = '';
				}} class="rounded-[7px] px-[14px] py-2 text-[13px] font-medium text-white" style="background: var(--danger);">Delete</button>
			</div>
		</div>
	</div>
{/if}
