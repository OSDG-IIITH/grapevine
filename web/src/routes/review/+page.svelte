<script lang="ts">
	import { page } from '$app/state';
	import coursesData from '$lib/mock/courses.json';
	import courseDetailData from '$lib/mock/course_detail.json';
	import facultyData from '$lib/mock/faculty.json';
	import type { CourseLean, FacultyLean, CourseDetail } from '$lib/types';
	import { COURSE_AXIS_ORDER, COURSE_AXIS_LABELS, ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS } from '$lib/types';
	import SegBar from '$lib/components/SegBar.svelte';
	import Crumbs from '$lib/components/Crumbs.svelte';

	const allcourses = coursesData as CourseLean[];
	const allfaculty = facultyData as FacultyLean[];
	const coursedetails = courseDetailData as Record<string, CourseDetail>;

	function getofferings(cid: string) {
		return Object.values(coursedetails).find((d) => d.id === cid)?.offerings ?? [];
	}

	const precourse = page.url.searchParams.get('course');
	const prefaculty = page.url.searchParams.get('faculty');

	const initcourse = precourse
		? (allcourses.find((c) => c.id === precourse || c.code === precourse)?.id ?? allcourses[0].id)
		: allcourses[0].id;
	const initfaculty = prefaculty
		? (allfaculty.find((f) => f.slug === prefaculty || f.id === prefaculty)?.id ?? allfaculty[0].id)
		: allfaculty[0].id;

	let kind = $state<'course' | 'advisor'>(prefaculty ? 'advisor' : 'course');
	let courseid = $state(initcourse);
	let offeringid = $state(getofferings(initcourse)[0]?.id ?? '');
	let facultyid = $state(initfaculty);
	let axes = $state<Record<string, number>>({});
	let body = $state('');
	let anon = $state(true);
	let submitted = $state(false);

	const offerings = $derived(getofferings(courseid));
	const axisorder = $derived(kind === 'course' ? [...COURSE_AXIS_ORDER] : [...ADVISOR_AXIS_ORDER]);
	const axislabels = $derived(kind === 'course' ? COURSE_AXIS_LABELS : ADVISOR_AXIS_LABELS);
	const cansubmit = $derived(body.trim().length > 20 && axisorder.every((a) => axes[a]));
	const selectedfaculty = $derived(allfaculty.find((f) => f.id === facultyid));

	$effect(() => {
		const offs = offerings;
		if (offs.length && !offs.find((o) => o.id === offeringid)) {
			offeringid = offs[0].id;
		}
	});

	function setkind(k: 'course' | 'advisor') {
		kind = k;
		axes = {};
	}
</script>

{#if submitted}
	<div class="mx-auto w-full px-8 pb-[120px] pt-10" style="max-width: 820px; animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">
		<div
			class="mx-auto mt-[60px] max-w-[560px] rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[40px] py-[60px] text-center"
			style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);"
		>
			<div class="mb-2 text-[var(--fg)]" style="font-family: var(--serif); font-size: 44px;">Thank you.</div>
			<div class="mb-7 text-[13px] text-[var(--fg-3)]">Your review has been submitted. It will appear here in a moment.</div>
			<a
				href={kind === 'course' ? `/courses/${courseid}` : `/faculty/${selectedfaculty?.slug ?? ''}`}
				class="inline-flex items-center gap-2 rounded-[7px] px-[14px] py-2 text-[13px] font-medium transition-[background,border-color] duration-[120ms]"
				style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
			>
				View {kind === 'course' ? 'course' : 'faculty'} page
			</a>
		</div>
	</div>
{:else}
	<div class="mx-auto w-full px-8 pb-[120px] pt-10" style="max-width: 820px; animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">
		<Crumbs items={[{ label: 'grapevine', href: '/' }, { label: 'write a review' }]} />

		<h1 class="mb-2 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: 44px; line-height: 1.05; letter-spacing: -0.015em;">
			Write a review
		</h1>
		<div class="mb-8 flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
			<span>Honest, specific, civil.</span>
			<span class="text-[var(--fg-4)]">·</span>
			<span>Your name is hidden by default.</span>
		</div>

		<!-- Kind row -->
		<div class="items-start border-b border-[var(--border)] py-6" style="display: grid; grid-template-columns: 200px 1fr; gap: 32px;">
			<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
				Reviewing
				<span class="mt-[6px] block text-[12px] normal-case tracking-normal text-[var(--fg-4)]">
					Choose what kind of review you're writing.
				</span>
			</div>
			<div class="min-w-0">
				<div class="inline-flex overflow-hidden rounded-[7px] border border-[var(--border)] bg-[var(--bg-2)]">
					<button
						type="button"
						class="border-r border-[var(--border)] px-[18px] py-[9px] text-[13px] transition-[background,color] duration-[120ms] {kind === 'course' ? 'bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
						onclick={() => setkind('course')}
					>Course offering</button>
					<button
						type="button"
						class="px-[18px] py-[9px] text-[13px] transition-[background,color] duration-[120ms] {kind === 'advisor' ? 'bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
						onclick={() => setkind('advisor')}
					>Faculty as advisor</button>
				</div>
			</div>
		</div>

		<!-- Course & offering / Faculty row -->
		{#if kind === 'course'}
			<div class="items-start border-b border-[var(--border)] py-6" style="display: grid; grid-template-columns: 200px 1fr; gap: 32px;">
				<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
					Course & offering
					<span class="mt-[6px] block text-[12px] normal-case tracking-normal text-[var(--fg-4)]">
						Which class in which semester.
					</span>
				</div>
				<div class="min-w-0" style="display: grid; grid-template-columns: 2fr 1fr; gap: 12px;">
					<div class="relative">
						<select
							class="w-full appearance-none rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[10px] pl-[14px] pr-8 text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
							bind:value={courseid}
						>
							{#each allcourses as c (c.id)}
								<option value={c.id}>{c.code} — {c.name}</option>
							{/each}
						</select>
						<div class="pointer-events-none absolute inset-y-0 right-3 flex items-center">
							<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
								<path d="M2 4l3 3 3-3" stroke="var(--fg-3)" stroke-width="1.4" stroke-linecap="round" />
							</svg>
						</div>
					</div>
					<div class="relative">
						<select
							class="w-full appearance-none rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[10px] pl-[14px] pr-8 text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
							bind:value={offeringid}
						>
							{#each offerings as o (o.id)}
								<option value={o.id}>{o.season} {o.year}</option>
							{/each}
						</select>
						<div class="pointer-events-none absolute inset-y-0 right-3 flex items-center">
							<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
								<path d="M2 4l3 3 3-3" stroke="var(--fg-3)" stroke-width="1.4" stroke-linecap="round" />
							</svg>
						</div>
					</div>
				</div>
			</div>
		{:else}
			<div class="items-start border-b border-[var(--border)] py-6" style="display: grid; grid-template-columns: 200px 1fr; gap: 32px;">
				<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
					Faculty
					<span class="mt-[6px] block text-[12px] normal-case tracking-normal text-[var(--fg-4)]">
						The advisor you worked with.
					</span>
				</div>
				<div class="min-w-0">
					<div class="relative">
						<select
							class="w-full appearance-none rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[10px] pl-[14px] pr-8 text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
							bind:value={facultyid}
						>
							{#each allfaculty as f (f.id)}
								<option value={f.id}>{f.name}</option>
							{/each}
						</select>
						<div class="pointer-events-none absolute inset-y-0 right-3 flex items-center">
							<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
								<path d="M2 4l3 3 3-3" stroke="var(--fg-3)" stroke-width="1.4" stroke-linecap="round" />
							</svg>
						</div>
					</div>
				</div>
			</div>
		{/if}

		<!-- Ratings row -->
		<div class="items-start border-b border-[var(--border)] py-6" style="display: grid; grid-template-columns: 200px 1fr; gap: 32px;">
			<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
				Ratings
				<span class="mt-[6px] block text-[12px] normal-case tracking-normal text-[var(--fg-4)]">
					Click a segment to set each axis (1 – 5).
				</span>
			</div>
			<div class="min-w-0" style="display: grid; grid-template-columns: 110px 1fr 30px; gap: 16px 18px; align-items: center;">
				{#each axisorder as k (k)}
					<div class="text-[13px] text-[var(--fg-2)]">{axislabels[k]}</div>
					<SegBar score={axes[k] ?? 0} interactive onchange={(v) => (axes = { ...axes, [k]: v })} />
					<div class="text-right text-[12px] text-[var(--fg)]" style="font-family: var(--mono);">
						{axes[k] ? axes[k].toFixed(1) : '—'}
					</div>
				{/each}
			</div>
		</div>

		<!-- Review text row -->
		<div class="items-start border-b border-[var(--border)] py-6" style="display: grid; grid-template-columns: 200px 1fr; gap: 32px;">
			<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
				Your review
				<span class="mt-[6px] block text-[12px] normal-case tracking-normal text-[var(--fg-4)]">
					Be specific. Future you would have wanted this.
				</span>
			</div>
			<div class="min-w-0">
				<textarea
					class="w-full resize-y rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[16px] py-[14px] text-[14px] leading-[1.65] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
					style="min-height: 180px;"
					placeholder="What was the workload like? What kinds of assignments? What surprised you?"
					bind:value={body}
				></textarea>
				<div
					class="mt-2 text-right text-[11px] {body.trim().length > 20 ? 'text-[var(--accent)]' : 'text-[var(--fg-4)]'}"
					style="font-family: var(--mono);"
				>
					{body.trim().length} characters {body.trim().length > 20 ? '✓' : '(min 20)'}
				</div>
			</div>
		</div>

		<!-- Visibility + actions row -->
		<div class="items-center py-6" style="display: grid; grid-template-columns: 200px 1fr; gap: 32px;">
			<div class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
				Visibility
				<span class="mt-[6px] block text-[12px] normal-case tracking-normal text-[var(--fg-4)]">
					Your name is hidden by default.
				</span>
			</div>
			<div class="flex min-w-0 items-center justify-between gap-4">
				<button
					type="button"
					class="inline-flex cursor-pointer items-center gap-[10px] text-[13px] text-[var(--fg-2)]"
					onclick={() => (anon = !anon)}
					aria-label="Toggle anonymous posting"
				>
					<span
						class="relative h-[18px] w-8 rounded-full border transition-[background,border-color] duration-[120ms] {anon ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)]' : 'border-[var(--border)] bg-[var(--bg-inset)]'}"
					>
						<span
							class="absolute top-[2px] h-3 w-3 rounded-full transition-[transform,background] duration-[140ms] {anon ? 'translate-x-[14px] bg-[var(--accent)]' : 'translate-x-[2px] bg-[var(--fg-3)]'}"
						></span>
					</span>
					Post anonymously
				</button>
				<div class="flex gap-2">
					<button
						type="button"
						class="inline-flex items-center gap-2 rounded-[7px] border border-transparent bg-transparent px-[14px] py-2 text-[13px] whitespace-nowrap text-[var(--fg-2)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
						onclick={() => history.back()}
					>Cancel</button>
					<button
						type="button"
						class="inline-flex items-center gap-2 rounded-[7px] border px-[14px] py-2 text-[13px] whitespace-nowrap transition-[background,border-color] duration-[120ms] {cansubmit ? 'font-medium' : ''}"
						style="{cansubmit ? 'background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);' : 'border-color: var(--border-2); background: var(--bg-2); color: var(--fg); opacity: 0.5; cursor: not-allowed;'}"
						disabled={!cansubmit}
						onclick={() => cansubmit && (submitted = true)}
					>Submit review</button>
				</div>
			</div>
		</div>
	</div>
{/if}
