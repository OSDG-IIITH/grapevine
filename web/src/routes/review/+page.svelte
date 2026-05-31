<script lang="ts">
	import { page } from '$app/state';
	import { base } from '$app/paths';
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { getCourses, getFaculty, getCourse, createCourseReview, createAdvisorReview } from '$lib/api';
	import { currentUser } from '$lib/stores';
	import { PUBLIC_API_URL } from '$env/static/public';
	import type { CourseLean, FacultyLean, Offering } from '$lib/types';
	import { COURSE_AXIS_ORDER, COURSE_AXIS_LABELS, ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS } from '$lib/types';
	import SegBar from '$lib/components/SegBar.svelte';
	import Crumbs from '$lib/components/Crumbs.svelte';

	const precourse = page.url.searchParams.get('course');
	const prefaculty = page.url.searchParams.get('faculty');

	let allcourses = $state<CourseLean[]>([]);
	let allfaculty = $state<FacultyLean[]>([]);
	let offerings = $state<Offering[]>([]);
	let loading = $state(true);

	let kind = $state<'course' | 'advisor'>(prefaculty ? 'advisor' : 'course');
	let courseid = $state('');
	let offeringid = $state('');
	let facultyid = $state('');
	let axes = $state<Record<string, number>>({});
	let body = $state('');
	let anon = $state(true);
	let submitted = $state(false);
	let submitting = $state(false);

	onMount(async () => {
		const [courses, faculty] = await Promise.all([getCourses(), getFaculty()]);
		allcourses = courses ?? [];
		allfaculty = faculty ?? [];

		if (allcourses.length) {
			const init = precourse
				? allcourses.find((c) => c.id === precourse || c.code === precourse)
				: allcourses[0];
			courseid = init?.id ?? allcourses[0].id;
		}

		if (allfaculty.length) {
			const init = prefaculty
				? allfaculty.find((f) => f.slug === prefaculty || f.id === prefaculty)
				: allfaculty[0];
			facultyid = init?.id ?? allfaculty[0].id;
		}

		loading = false;
	});

	let fetchseq = 0;
	$effect(() => {
		const id = courseid;
		if (!id || !allcourses.length) return;
		const c = allcourses.find((co) => co.id === id);
		if (!c) return;
		const seq = ++fetchseq;
		getCourse(c.code).then((detail) => {
			if (seq !== fetchseq) return;
			offerings = detail?.offerings ?? [];
			if (offerings.length) offeringid = offerings[0].id;
		});
	});

	const axisorder = $derived(kind === 'course' ? [...COURSE_AXIS_ORDER] : [...ADVISOR_AXIS_ORDER]);
	const axislabels = $derived(kind === 'course' ? COURSE_AXIS_LABELS : ADVISOR_AXIS_LABELS);
	const cansubmit = $derived(!submitting && body.trim().length > 20 && axisorder.every((a) => axes[a]));
	const selectedfaculty = $derived(allfaculty.find((f) => f.id === facultyid));
	const selectedcourse = $derived(allcourses.find((c) => c.id === courseid));

	function setkind(k: 'course' | 'advisor') {
		kind = k;
		axes = {};
	}

	async function submit() {
		if (!cansubmit) return;
		if (!get(currentUser)) {
			window.location.href = `${PUBLIC_API_URL || '/api'}/auth/login`;
			return;
		}
		submitting = true;

		let result;
		if (kind === 'course') {
			result = await createCourseReview(offeringid, {
				anonymous: anon,
				difficulty: axes.difficulty,
				teaching: axes.teaching,
				grading: axes.grading,
				content: axes.content,
				workload: axes.workload,
				body
			});
		} else {
			const fac = allfaculty.find((f) => f.id === facultyid);
			if (fac) {
				result = await createAdvisorReview(fac.slug, {
					anonymous: anon,
					research: axes.research,
					availability: axes.availability,
					mentorship: axes.mentorship,
					support: axes.support,
					workload: axes.workload,
					body
				});
			}
		}

		submitting = false;
		if (result) submitted = true;
	}
</script>

{#if submitted}
	<div class="mx-auto w-full px-6 pb-[120px] pt-10 sm:px-8" style="max-width: 820px; animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">
		<div
			class="mx-auto mt-[60px] max-w-[560px] rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[40px] py-[60px] text-center"
			style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);"
		>
			<div class="mb-2 text-[var(--fg)]" style="font-family: var(--serif); font-size: 44px;">Thank you.</div>
			<div class="mb-7 text-[13px] text-[var(--fg-3)]">Your review has been submitted. It will appear here in a moment.</div>
			<a
				href={kind === 'course' ? `${base}/courses/${selectedcourse?.code ?? ''}` : `${base}/faculty/${selectedfaculty?.slug ?? ''}`}
				class="inline-flex items-center gap-2 rounded-[7px] px-[14px] py-2 text-[13px] font-medium transition-[background,border-color] duration-[120ms]"
				style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
			>
				View {kind === 'course' ? 'course' : 'faculty'} page
			</a>
		</div>
	</div>
{:else}
	<div class="mx-auto w-full px-6 pb-[120px] pt-10 sm:px-8" style="max-width: 820px; animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">
		<Crumbs items={[{ label: 'grapevine', href: '/' }, { label: 'write a review' }]} />

		<h1 class="mb-2 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: 44px; line-height: 1.05; letter-spacing: -0.015em;">
			Write a review
		</h1>

		{#if loading}
			<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
		{:else}
			<!-- Kind row -->
			<div class="grid grid-cols-1 gap-4 border-b border-[var(--border)] py-6 sm:grid-cols-[200px_1fr] sm:items-start sm:gap-[32px]">
				<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
					Reviewing
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
				<div class="grid grid-cols-1 gap-4 border-b border-[var(--border)] py-6 sm:grid-cols-[200px_1fr] sm:items-start sm:gap-[32px]">
					<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
						Course & offering
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
				<div class="grid grid-cols-1 gap-4 border-b border-[var(--border)] py-6 sm:grid-cols-[200px_1fr] sm:items-start sm:gap-[32px]">
					<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
						Faculty
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
			<div class="grid grid-cols-1 gap-4 border-b border-[var(--border)] py-6 sm:grid-cols-[200px_1fr] sm:items-start sm:gap-[32px]">
				<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
					Ratings
					<span class="mt-[6px] block text-[12px] normal-case tracking-normal text-[var(--fg-4)]">
						1 is bad. 5 is good.
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
			<div class="grid grid-cols-1 gap-4 border-b border-[var(--border)] py-6 sm:grid-cols-[200px_1fr] sm:items-start sm:gap-[32px]">
				<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
					Your review
				</div>
				<div class="min-w-0">
					<textarea
						class="w-full resize-y rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[16px] py-[14px] text-[14px] leading-[1.65] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
						style="min-height: 180px;"
						placeholder="Share your experience."
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
			<div class="grid grid-cols-1 gap-4 py-6 sm:grid-cols-[200px_1fr] sm:items-center sm:gap-[32px]">
				<div class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
					Visibility
				</div>
				<div class="flex min-w-0 items-center justify-between gap-4">
					<label class="inline-flex cursor-pointer items-center gap-[10px] text-[13px] text-[var(--fg-2)]">
						<input type="checkbox" class="peer sr-only" bind:checked={anon} />
						<span
							class="relative flex h-[18px] w-[18px] items-center justify-center rounded-[5px] border transition-[background,border-color,box-shadow] duration-[120ms] {anon ? 'border-[var(--accent)] bg-[var(--accent-bg)]' : 'border-[var(--border)] bg-[var(--bg-inset)]'}"
							style="{anon ? 'box-shadow: inset 0 0 0 1px rgba(0,0,0,0.08)' : ''}"
						>
							<svg
								viewBox="0 0 16 16"
								class="h-[11px] w-[11px] transition-[opacity,transform] duration-[120ms] {anon ? 'opacity-100 scale-100' : 'opacity-0 scale-90'}"
								fill="none"
								aria-hidden="true"
							>
								<path
									d="M3.5 8.5l2.6 2.6L12.5 4.9"
									stroke="var(--accent)"
									stroke-width="1.8"
									stroke-linecap="round"
									stroke-linejoin="round"
								/>
							</svg>
						</span>
						<span>Post anonymously</span>
					</label>
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
							onclick={submit}
						>{submitting ? 'Submitting…' : 'Submit review'}</button>
					</div>
				</div>
			</div>
		{/if}
	</div>
{/if}
