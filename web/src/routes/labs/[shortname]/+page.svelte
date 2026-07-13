<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { getLab, updateLab, deleteLab, submitReport } from '$lib/api';
	import { toast } from 'svelte-sonner';
	import type { LabDetail } from '$lib/types';
	import { ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS } from '$lib/types';
	import { currentUser } from '$lib/stores';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import RatingsBlock from '$lib/components/RatingsBlock.svelte';
	import SegBar from '$lib/components/SegBar.svelte';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import ReportModal from '$lib/components/ReportModal.svelte';

	const shortname = $derived($page.params.shortname);

	let lab = $state<LabDetail | null>(null);
	let error = $state('');
	let reportopen = $state(false);
	let reportsubmitting = $state(false);

	async function sendReport(reason: string) {
		if (!lab) return;
		reportsubmitting = true;
		const ok = await submitReport('lab', lab.id, reason);
		reportsubmitting = false;
		if (ok) {
			toast.success('Report sent to the moderators.');
			reportopen = false;
		}
	}

	let editing = $state(false);
	let saving = $state(false);
	let editname = $state('');
	let editshort = $state('');
	let editdesc = $state('');

	$effect(() => {
		const s = shortname;
		if (!s) return;
		lab = null;
		reportopen = false;
		error = '';
		editing = false;
		getLab(s).then((data) => {
			if (!data) { error = 'Lab not found.'; return; }
			lab = data;
		});
	});

	function startEdit() {
		if (!lab) return;
		editname = lab.name;
		editshort = lab.short;
		editdesc = lab.description;
		editing = true;
	}

	function cancelEdit() { editing = false; }

	let confirmdel = $state(false);
	let deleting = $state(false);

	async function doDelete() {
		if (!lab) return;
		deleting = true;
		const ok = await deleteLab(lab.short);
		deleting = false;
		if (ok) {
			toast.success('Lab removed.');
			goto(`${base}/labs`);
		}
	}

	async function saveEdit() {
		if (!lab) return;
		saving = true;
		const updated = await updateLab(lab.short, { name: editname, short: editshort, description: editdesc });
		saving = false;
		if (updated) {
			lab = updated;
			editing = false;
			if (updated.short !== shortname) {
				goto(`${base}/labs/${updated.short}`, { replaceState: true });
			}
		}
	}
</script>

<svelte:head>
	<title>{lab ? `${lab.name} · grapevine` : 'Lab · grapevine'}</title>
</svelte:head>

<div class="mx-auto w-full max-w-[1180px] px-4 pb-[120px] pt-10 sm:px-8" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	{#if error}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">{error}</div>

	{:else if lab}
		<Crumbs items={[
			{ label: 'grapevine', href: '/' },
			{ label: 'labs', href: '/labs' },
			{ label: lab.short.toLowerCase() }
		]} />

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
						{lab.name}
					</h1>
				{/if}
				<div class="mb-[22px] flex flex-wrap items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
					{#if editing}
						<input
							bind:value={editshort}
							class="rounded-[5px] border border-[var(--border-strong)] bg-transparent px-2 py-[3px] text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--accent-2)]"
							style="font-family: var(--mono); min-width: 80px;"
						/>
					{:else}
						<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{lab.short}</span>
					{/if}
					<span>{lab.facultycount} faculty</span>
				</div>
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
				{/if}
			</div>
		</div>

		{#if editing}
			<Textarea
				bind:value={editdesc}
				rows={4}
				class="mb-7 mt-[14px] w-full resize-none border-[var(--border-strong)] text-[var(--fg-2)] focus-visible:border-[var(--accent-2)] focus-visible:ring-0 dark:bg-input/10"
				style="font-size: 15px; line-height: 1.65;"
			/>
		{:else}
			<p class="mb-7 mt-[14px] max-w-[720px] leading-[1.65] text-[var(--fg-2)]" style="font-size: 15px; text-wrap: pretty;">
				{lab.description}
			</p>
		{/if}

		<RatingsBlock
			overall={lab.overall}
			axes={lab.axes}
			axisorder={[...ADVISOR_AXIS_ORDER]}
			axislabels={ADVISOR_AXIS_LABELS}
			bar="continuous"
		/>

		<!-- faculty list -->
		<div class="mb-3 mt-6 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
			Faculty members ({lab.faculty.length})
		</div>

		{#if lab.faculty.length === 0}
			<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
				No faculty profiles available yet.
			</div>
		{:else}
			<div
				class="flex flex-col overflow-hidden rounded-[10px] border border-[var(--border)]"
				style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107, 143, 111, 0.035), transparent 42%);"
			>
				{#each lab.faculty as m (m.id)}
					<a
						href="{base}/faculty/{m.slug}"
						class="flex flex-wrap items-center gap-x-5 gap-y-2 border-b border-[var(--border)] px-5 py-4 transition-colors duration-[120ms] last:border-b-0 hover:bg-[var(--bg-3)]"
					>
						<div class="min-w-[140px] flex-1">
							<div class="text-[14px] text-[var(--fg)]">{m.name}</div>
							<div class="mt-[2px] text-[12px] text-[var(--fg-3)]">{m.title}</div>
						</div>
						<div class="flex min-w-[120px] flex-1 items-center gap-3">
							<div class="flex-1"><SegBar score={m.overall ?? 0} size="sm" /></div>
							<span class="shrink-0 text-[12px] text-[var(--fg)]" style="font-family: var(--mono);">{(m.overall ?? 0).toFixed(1)}</span>
						</div>
					</a>
				{/each}
			</div>
		{/if}

	{:else}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
	{/if}
</div>

{#if confirmdel}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true">
		<div class="w-full max-w-[340px] rounded-[12px] border border-[var(--border)] bg-[var(--bg-2)] p-6 shadow-xl">
			<p class="mb-1 text-[15px] font-medium text-[var(--fg)]">Delete lab?</p>
			<p class="mb-5 text-[13px] text-[var(--fg-3)]">The lab will be hidden from all listings and can be restored later.</p>
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

{#if reportopen && lab}
	<ReportModal target={`${lab.short} · ${lab.name}`} submitting={reportsubmitting} onclose={() => { if (!reportsubmitting) reportopen = false; }} onsubmit={sendReport} />
{/if}
