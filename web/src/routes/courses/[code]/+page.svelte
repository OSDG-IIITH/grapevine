<script lang="ts">
	import { page } from '$app/stores';
	import { getCourse, getCourseReviews } from '$lib/api';
	import type { CourseDetail, CourseReview } from '$lib/types';
	import { COURSE_AXIS_ORDER, COURSE_AXIS_LABELS } from '$lib/types';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import RatingsBlock from '$lib/components/RatingsBlock.svelte';
	import Tabs from '$lib/components/Tabs.svelte';
	import ReviewCard from '$lib/components/ReviewCard.svelte';

	const code = $derived($page.params.code);

	let course = $state<CourseDetail | null>(null);
	let reviews = $state<CourseReview[]>([]);
	let tab = $state('all');
	let error = $state('');

	$effect(() => {
		const c = code;
		if (!c) return;
		error = '';
		course = null;
		reviews = [];
		tab = 'all';

		const decoded = decodeURIComponent(c);
		Promise.all([getCourse(decoded), getCourseReviews(decoded)])
			.then(([d, r]) => { course = d; reviews = r; })
			.catch(() => { error = 'Course not found.'; });
	});

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
		course ? Object.fromEntries(course.offerings.map((o) => [o.id, o.code])) : {}
	);

	const tabs = $derived(
		course ? [
			{ id: 'all', label: 'All', count: reviews.length },
			...course.offerings.map((o) => ({
				id: o.id,
				label: o.code,
				count: reviews.filter((r) => r.offering_id === o.id).length
			}))
		] : []
	);

	const shown = $derived(tab === 'all' ? reviews : reviews.filter((r) => r.offering_id === tab));
	const selectedoffering = $derived(course && tab !== 'all' ? course.offerings.find((o) => o.id === tab) : null);
</script>

<div class="mx-auto w-full max-w-[1180px] px-8 pb-[120px] pt-10" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	{#if error}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">{error}</div>

	{:else if course}
		<Crumbs items={[{ label: 'grapevine', href: '/' }, { label: 'courses', href: '/courses' }, { label: course.code }]} />

		<!-- page head -->
		<div class="flex flex-wrap items-start justify-between gap-6">
			<div>
				<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: 56px; line-height: 1.05; letter-spacing: -0.015em;">
					{course.name}
				</h1>
				<div class="mb-[22px] flex flex-wrap items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
					<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{course.code}</span>
					<span class="text-[var(--fg-4)]">·</span>
					<span class="text-[11px] tracking-[0.04em] {course.type === 'core' ? 'text-[var(--accent-2)]' : 'text-[var(--fg-4)]'}" style="font-family: var(--mono);">{course.type}</span>
				</div>
			</div>
			<a
				href="/review?course={course.code}"
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
			{course.description}
		</p>

		<RatingsBlock
			overall={course.overall}
			{axes}
			axisorder={[...COURSE_AXIS_ORDER]}
			axislabels={COURSE_AXIS_LABELS}
			reviewcount={reviews.length}
			bar="continuous"
		/>

		<Tabs items={tabs} active={tab} onchange={(id) => (tab = id)} />

		<!-- taught-by banner -->
		{#if selectedoffering}
			<div class="mb-[18px] flex items-center gap-[14px] rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[18px] py-[14px] text-[13px]">
				<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Taught by</span>
				{#each selectedoffering.faculty as f, i (f.id)}
					<a href="/faculty/{f.slug ?? f.id}" class="text-[var(--fg)] transition-colors duration-[120ms] hover:text-[var(--accent-2)]">
						{f.name}{i < selectedoffering.faculty.length - 1 ? ',' : ''}
					</a>
				{/each}
			</div>
		{/if}

		<!-- reviews -->
		{#if shown.length === 0}
			<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
				No reviews for this offering yet.
			</div>
		{:else}
			<div style="display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px;">
				{#each shown as r (r.id)}
					<ReviewCard
						review={r}
						axisorder={[...COURSE_AXIS_ORDER]}
						axislabels={COURSE_AXIS_LABELS}
						showoffering={tab === 'all'}
						offeringcode={offeringmap[r.offering_id]}
					/>
				{/each}
			</div>
		{/if}

	{:else}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
	{/if}
</div>
