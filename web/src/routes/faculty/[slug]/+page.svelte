<script lang="ts">
	import { page } from '$app/stores';
	import { base } from '$app/paths';
	import { getFacultyMember, getAdvisorReviews, getOfferingReviews } from '$lib/api';
	import type { FacultyDetail, AdvisorReview, CourseReview } from '$lib/types';
	import { ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS, COURSE_AXIS_ORDER, COURSE_AXIS_LABELS } from '$lib/types';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import RatingsBlock from '$lib/components/RatingsBlock.svelte';
	import Tabs from '$lib/components/Tabs.svelte';
	import ReviewCard from '$lib/components/ReviewCard.svelte';
	import Pager from '$lib/components/Pager.svelte';

	const slug = $derived($page.params.slug);

	let faculty = $state<FacultyDetail | null>(null);
	let advisorreviews = $state<AdvisorReview[]>([]);
	let instructorreviews = $state<(CourseReview & { offeringcode: string; coursecode: string })[]>([]);
	let tab = $state('advisor');
	let error = $state('');

	$effect(() => {
		const s = slug;
		if (!s) return;
		faculty = null;
		advisorreviews = [];
		instructorreviews = [];
		tab = 'advisor';
		error = '';

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
	});

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
		{ id: 'advisor', label: 'As Advisor', count: advisorreviews.length },
		{ id: 'instructor', label: 'As Instructor', count: instructorreviews.length }
	]);

	const PER_PAGE = 10;
	let reviewpage = $state(1);
	const showncount = $derived(tab === 'advisor' ? advisorreviews.length : instructorreviews.length);
	const reviewpages = $derived(Math.max(1, Math.ceil(showncount / PER_PAGE)));

	const taughtcourses = $derived((() => {
		if (!faculty) return [];
		const seen = new Map<string, { code: string; name: string }>();
		for (const o of faculty.offerings) {
			if (!seen.has(o.course.id)) seen.set(o.course.id, { code: o.course.code, name: o.course.name });
		}
		return Array.from(seen.values());
	})());
</script>

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
			<div>
				<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;">
					{faculty.name}
				</h1>
				<div class="mb-[22px] flex flex-wrap items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
					<span class="text-[11px] tracking-[0.04em] text-[var(--fg-4)]" style="font-family: var(--mono);">{faculty.title.toLowerCase()}</span>
					{#if faculty.labs.length > 0}
						{#each faculty.labs as lab, i (lab.id)}
							{#if i > 0}<span class="text-[var(--fg-4)]">·</span>{/if}
							<a href="{base}/labs/{lab.short}" class="text-[var(--fg-2)] transition-colors duration-[120ms] hover:text-[var(--fg)]">{lab.name}</a>
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{lab.short}</span>
						{/each}
					{:else}
						<span class="text-[var(--fg-3)]">Independent</span>
					{/if}
				</div>
			</div>
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
		</div>

		<p class="mb-7 mt-[14px] max-w-[720px] leading-[1.65] text-[var(--fg-2)]" style="font-size: 15px; text-wrap: pretty;">
			{faculty.bio}
		</p>

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
			{#if advisorreviews.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
					No advisor reviews yet.
				</div>
			{:else}
				<div class="grid grid-cols-1 gap-[12px] md:grid-cols-2">
					{#each advisorreviews.slice((reviewpage - 1) * PER_PAGE, reviewpage * PER_PAGE) as r (r.id)}
						<ReviewCard
							review={r}
							axisorder={[...ADVISOR_AXIS_ORDER]}
							axislabels={ADVISOR_AXIS_LABELS}
							ondelete={(id) => (advisorreviews = advisorreviews.filter((item) => item.id !== id))}
							onedit={(updated) => (advisorreviews = advisorreviews.map((item) => item.id === updated.id ? updated : item))}
						/>
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
