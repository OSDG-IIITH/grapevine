<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { getFacultyMember, getAdvisorReviews, getOfferingReviews, getLabs, updateFaculty, deleteFaculty } from '$lib/api';
	import { toast } from 'svelte-sonner';
	import type { FacultyDetail, AdvisorReview, CourseReview, LabLean } from '$lib/types';
	import { ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS, COURSE_AXIS_ORDER, COURSE_AXIS_LABELS } from '$lib/types';
	import { currentUser } from '$lib/stores';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import RatingsBlock from '$lib/components/RatingsBlock.svelte';
	import Tabs from '$lib/components/Tabs.svelte';
	import ReviewCard from '$lib/components/ReviewCard.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import { Textarea } from '$lib/components/ui/textarea/index.js';

	const slug = $derived($page.params.slug);

	let faculty = $state<FacultyDetail | null>(null);
	let advisorreviews = $state<AdvisorReview[]>([]);
	let instructorreviews = $state<(CourseReview & { offeringcode: string; coursecode: string })[]>([]);
	let tab = $state('advisor');
	let error = $state('');

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
		instructorreviews = [];
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

	const availablelabs = $derived(alllabs.filter((l) => !editlabs.find((e) => e.id === l.id)));

	let confirmdel = $state(false);
	let deleting = $state(false);

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
							onedit={(updated) => (advisorreviews = advisorreviews.map((item) => item.id === updated.id ? updated as AdvisorReview : item))}
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
