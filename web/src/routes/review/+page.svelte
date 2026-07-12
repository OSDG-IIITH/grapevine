<script lang="ts">
	import { page } from '$app/state';
	import { base } from '$app/paths';
	import { onMount, untrack } from 'svelte';
	import { getCourses, getFaculty, getCourse, createCourseReview, createAdvisorReview, proposeReview } from '$lib/api';
	import type { CourseLean, FacultyLean, Offering } from '$lib/types';
	import {
		COURSE_AXIS_ORDER,
		COURSE_AXIS_LABELS,
		COURSE_AXIS_SCALE_LABELS,
		ADVISOR_AXIS_ORDER,
		ADVISOR_AXIS_LABELS,
		ADVISOR_AXIS_SCALE_LABELS
	} from '$lib/types';
	import SegBar from '$lib/components/SegBar.svelte';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import Loader from "@lucide/svelte/icons/loader";

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

	let proposeSeason = $state('M');
	let proposeYear = $state('');
	let proposeYearError = $state(false);
	let proposeFaculty = $state<FacultyLean[]>([]);

	function removeProposeFaculty(fid: string) {
		proposeFaculty = proposeFaculty.filter((f) => f.id !== fid);
	}

	function addProposeFaculty(fid: string) {
		const f = allfaculty.find((fac) => fac.id === fid);
		if (f && !proposeFaculty.some((fac) => fac.id === fid)) {
			proposeFaculty = [...proposeFaculty, f];
		}
	}

	const availableProposeFaculty = $derived(
		allfaculty.filter((f) => !proposeFaculty.some((pf) => pf.id === f.id))
	);

	function parseyear(s: string): number | null {
		const t = s.trim();
		if (/^\d{2}$/.test(t)) return parseInt(t, 10);
		if (/^\d{4}$/.test(t)) { const y = parseInt(t, 10); return y >= 2000 && y <= 2099 ? y % 100 : null; }
		return null;
	}

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
				? allfaculty.find((f) => f.id === prefaculty || f.slug === prefaculty)
				: allfaculty[0];
			facultyid = init?.id ?? allfaculty[0].id;
		}

		loading = false;
	});

	let fetchseq = 0;
	let loadingOfferings = false;
	$effect(() => {
		const id = courseid;
		if (!id || !allcourses.length) return;
		const c = allcourses.find((co) => co.id === id);
		if (!c) return;
		const seq = ++fetchseq;
		untrack(() => {
			loadingOfferings = false;
		});
		getCourse(c.code).then((detail) => {
			untrack(() => {
				loadingOfferings = true;
			});
			if (seq !== fetchseq) return;
			offerings = detail?.offerings ?? [];
			if (offerings.length) offeringid = offerings[0].id;
			else offeringid = 'propose';
		});
	});

	const axisorder = $derived(kind === 'course' ? [...COURSE_AXIS_ORDER] : [...ADVISOR_AXIS_ORDER]);
	const axislabels = $derived(kind === 'course' ? COURSE_AXIS_LABELS : ADVISOR_AXIS_LABELS);
	const axisscalelabels = $derived(kind === 'course' ? COURSE_AXIS_SCALE_LABELS : ADVISOR_AXIS_SCALE_LABELS);
	const cansubmit = $derived(
		!submitting &&
		body.trim().length > 20 &&
		axisorder.every((a) => axes[a]) &&
		(kind !== 'course' || offeringid !== 'propose' || parseyear(proposeYear) !== null)
	);
	const selectedfaculty = $derived(allfaculty.find((f) => f.id === facultyid));
	const selectedcourse = $derived(allcourses.find((c) => c.id === courseid));
	const overall = $derived.by(() => {
		const filled = axisorder.filter((k) => axes[k]);
		return filled.length ? filled.reduce((s, k) => s + axes[k], 0) / filled.length : null;
	});

	function setkind(k: 'course' | 'advisor') {
		kind = k;
		axes = {};
	}

	async function submit() {
		if (!cansubmit) return;
		submitting = true;

		let result;
		if (kind === 'course') {
			if (offeringid === 'propose') {
				const year = parseyear(proposeYear);
				if (year === null) { proposeYearError = true; submitting = false; return; }
				proposeYearError = false;
				result = await proposeReview(selectedcourse!.code, {
					season: proposeSeason,
					year,
					anonymous: anon,
					difficulty: axes.difficulty,
					teaching: axes.teaching,
					grading: axes.grading,
					content: axes.content,
					workload: axes.workload,
					body,
					faculty_ids: proposeFaculty.map((f) => f.id)
				});
			} else {
				result = await createCourseReview(offeringid, {
					anonymous: anon,
					difficulty: axes.difficulty,
					teaching: axes.teaching,
					grading: axes.grading,
					content: axes.content,
					workload: axes.workload,
					body
				});
			}
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

<svelte:head>
	<title>Write Review · grapevine</title>
</svelte:head>

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
						<Combobox
							items={allcourses.map((c) => ({ value: c.id, label: c.name }))}
							bind:value={courseid}
							placeholder="Select course…"
							searchplaceholder="Search courses…"
							class="w-full rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[10px] pl-[14px] pr-4 text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)]"
						/>
						<div class="relative">
							<select
								class="w-full appearance-none rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[10px] pl-[14px] pr-8 text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
								bind:value={offeringid}
								disabled={loadingOfferings}
							>
								{#each offerings as o (o.id)}
									<option value={o.id}>{o.season === 'M' ? 'Monsoon' : 'Spring'} 20{o.year}</option>
								{/each}
								<option value="propose">Other</option>
							</select>
							{#if loadingOfferings}
								<div class="pointer-events-none absolute inset-y-0 right-6 flex items-center">
									<Loader class="animate-spin size-3" style="stroke: var(--fg-3)" />
								</div>
							{/if}
							<div class="pointer-events-none absolute inset-y-0 right-3 flex items-center">
								<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
									<path d="M2 4l3 3 3-3" stroke="var(--fg-3)" stroke-width="1.4" stroke-linecap="round" />
								</svg>
							</div>
						</div>
						{#if offeringid === 'propose'}
							<div class="col-span-2 mt-3 flex flex-wrap items-center gap-[8px] rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-4 py-3" style="animation: fadeUp 200ms cubic-bezier(.2,.6,.2,1) both;">
								<span class="text-[13px] font-medium text-[var(--fg-2)] mr-1">Proposed semester:</span>
								<!-- season toggle -->
								<div class="flex rounded-[6px] border border-[var(--border-strong)] p-[2px]" style="background: var(--bg-3);">
									<button
										type="button"
										onclick={() => { proposeSeason = 'M'; }}
										class="rounded-[4px] px-[9px] py-[3px] text-[12px] font-medium transition-colors duration-[100ms] {proposeSeason === 'M' ? 'text-[var(--fg)]' : 'text-[var(--fg-4)] hover:text-[var(--fg-3)]'}"
										style="font-family: var(--mono); {proposeSeason === 'M' ? 'background: var(--bg-4);' : ''}"
									>Monsoon</button>
									<button
										type="button"
										onclick={() => { proposeSeason = 'S'; }}
										class="rounded-[4px] px-[9px] py-[3px] text-[12px] font-medium transition-colors duration-[100ms] {proposeSeason === 'S' ? 'text-[var(--fg)]' : 'text-[var(--fg-4)] hover:text-[var(--fg-3)]'}"
										style="font-family: var(--mono); {proposeSeason === 'S' ? 'background: var(--bg-4);' : ''}"
									>Spring</button>
								</div>
								<!-- year input -->
								<input
									bind:value={proposeYear}
									placeholder="2026"
									oninput={() => { proposeYearError = false; }}
									class="rounded-[5px] border bg-transparent px-[8px] py-[4px] text-[12px] outline-none transition-colors duration-[100ms] {proposeYearError ? 'border-[var(--danger)] text-[var(--danger)]' : 'border-[var(--border-strong)] text-[var(--fg-2)] focus:border-[var(--accent-2)]'}"
									style="font-family: var(--mono); width: 64px;"
								/>

								<!-- instructors list -->
								{#each proposeFaculty as f (f.id)}
									<span class="flex items-center gap-[4px] rounded-[5px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[8px] py-[4px] text-[12px] text-[var(--fg-2)]">
										{f.name}
										<button
											type="button"
											onclick={() => removeProposeFaculty(f.id)}
											aria-label="Remove {f.name}"
											class="ml-[2px] flex h-[14px] w-[14px] items-center justify-center rounded-full text-[var(--fg-4)] transition-colors hover:text-[var(--fg)]"
										>
											<svg width="8" height="8" viewBox="0 0 8 8" fill="none" aria-hidden="true">
												<path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
											</svg>
										</button>
									</span>
								{/each}

								{#if availableProposeFaculty.length > 0}
									<select
										onchange={(e) => { const v = (e.target as HTMLSelectElement).value; if (v) { addProposeFaculty(v); (e.target as HTMLSelectElement).value = ''; } }}
										class="appearance-none rounded-[5px] border border-dashed border-[var(--border-strong)] bg-transparent px-[8px] py-[4px] text-[12px] text-[var(--fg-4)] outline-none transition-colors hover:border-[var(--fg-4)] hover:text-[var(--fg-3)]"
										style="font-family: var(--mono); width: 110px;"
									>
										<option value="">+ instructor</option>
										{#each availableProposeFaculty as f (f.id)}
											<option value={f.id}>{f.name}</option>
										{/each}
									</select>
								{/if}

								{#if proposeYearError}
									<span class="text-[11px] text-[var(--danger)] ml-2">Enter valid year</span>
								{/if}
							</div>
						{/if}
					</div>
				</div>
			{:else}
				<div class="grid grid-cols-1 gap-4 border-b border-[var(--border)] py-6 sm:grid-cols-[200px_1fr] sm:items-start sm:gap-[32px]">
					<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
						Faculty
					</div>
					<div class="min-w-0">
						<Combobox
							items={allfaculty.map((f) => ({ value: f.id, label: f.name }))}
							bind:value={facultyid}
							placeholder="Select faculty…"
							searchplaceholder="Search faculty…"
							class="w-full rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[10px] pl-[14px] pr-4 text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)]"
						/>
					</div>
				</div>
			{/if}

			<!-- Ratings row -->
			<div class="grid grid-cols-1 gap-4 border-b border-[var(--border)] py-6 sm:grid-cols-[200px_1fr] sm:items-start sm:gap-[32px]">
				<div class="pt-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
					Ratings
				</div>
				<div class="min-w-0" style="display: grid; grid-template-columns: 110px 1fr 30px; gap: 18px; align-items: start;">
					{#each axisorder as k (k)}
						<div class="pt-[3px] text-[13px] text-[var(--fg-2)]">{axislabels[k]}</div>
						<div class="min-w-0">
							<SegBar score={axes[k] ?? 0} interactive onchange={(v) => (axes = { ...axes, [k]: v })} />
							<div class="mt-[5px] grid grid-cols-5 gap-[5px] text-[10px] leading-none text-[var(--fg-4)]" style="font-family: var(--mono);">
								<span class="whitespace-nowrap">{axisscalelabels[k].low}</span>
								<span class="col-start-5 whitespace-nowrap text-right">{axisscalelabels[k].high}</span>
							</div>
						</div>
						<div class="pt-[3px] text-right text-[12px] text-[var(--fg)]" style="font-family: var(--mono);">
							{axes[k] ? axes[k].toFixed(1) : '—'}
						</div>
					{/each}
					<div class="col-span-3 mt-1 flex items-center justify-end gap-3">
						<div class="text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">overall</div>
						<div class="text-[13px] font-medium text-[var(--fg)]" style="font-family: var(--mono);">
							{overall !== null ? overall.toFixed(1) : '—'}
						</div>
					</div>
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
