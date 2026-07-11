<script lang="ts">
	import { onMount } from 'svelte';
	import { getFlags, dismissFlag, deleteFlaggedReview, exportSeedData, getProposedOfferings, approveProposedOffering, rejectProposedOffering, getDeletedCourses, restoreCourse, getDeletedFaculty, restoreFaculty, getDeletedLabs, restoreLab } from '$lib/api';
	import { currentUser } from '$lib/stores';
	import type { FlagResponse, ProposedOfferingResponse } from '$lib/types';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import Tabs from '$lib/components/Tabs.svelte';

	let items = $state<FlagResponse[]>([]);
	let proposed = $state<ProposedOfferingResponse[]>([]);
	let deleted = $state<{ code: string; name: string; deleted_at: string }[]>([]);
	let deletedfaculty = $state<{ slug: string; name: string; deleted_at: string }[]>([]);
	let deletedlabs = $state<{ shortname: string; name: string; deleted_at: string }[]>([]);
	let loading = $state(true);
	let error = $state(false);
	let activetab = $state<'flagged' | 'proposed' | 'deleted'>('flagged');

	onMount(async () => {
		const [flags, props, del, delfac, dellabs] = await Promise.all([
			getFlags(), getProposedOfferings(), getDeletedCourses(), getDeletedFaculty(), getDeletedLabs()
		]);
		if (flags === null || props === null) error = true;
		else {
			items = flags;
			proposed = props;
			deleted = del ?? [];
			deletedfaculty = delfac ?? [];
			deletedlabs = dellabs ?? [];
		}
		loading = false;
	});

	async function doRestore(code: string) {
		if (await restoreCourse(code)) deleted = deleted.filter((d) => d.code !== code);
	}

	async function doRestoreFaculty(slug: string) {
		if (await restoreFaculty(slug)) deletedfaculty = deletedfaculty.filter((d) => d.slug !== slug);
	}

	async function doRestoreLab(shortname: string) {
		if (await restoreLab(shortname)) deletedlabs = deletedlabs.filter((d) => d.shortname !== shortname);
	}

	async function dismiss(id: string) {
		if (await dismissFlag(id)) items = items.filter((i) => i.id !== id);
	}

	async function deleteReview(id: string) {
		if (await deleteFlaggedReview(id)) items = items.filter((i) => i.id !== id);
	}

	async function approveProp(id: string) {
		if (await approveProposedOffering(id)) {
			proposed = proposed.filter((p) => p.id !== id);
		}
	}

	async function rejectProp(id: string) {
		if (await rejectProposedOffering(id)) {
			proposed = proposed.filter((p) => p.id !== id);
		}
	}

	let exporting = $state(false);

	async function doexport() {
		exporting = true;
		const data = await exportSeedData();
		exporting = false;
		if (!data) return;
		for (const key of ['labs', 'faculty', 'courses', 'offerings'] as const) {
			const blob = new Blob([JSON.stringify(data[key], null, 2)], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url; a.download = `${key}.json`; a.click();
			URL.revokeObjectURL(url);
			await new Promise((r) => setTimeout(r, 80));
		}
	}

	function reltime(iso: string): string {
		const d = Math.floor((Date.now() - new Date(iso).getTime()) / 86400000);
		if (d === 0) return 'today';
		if (d === 1) return 'yesterday';
		if (d < 7) return `${d} days ago`;
		if (d < 30) return `${Math.floor(d / 7)}w ago`;
		return new Date(iso).toLocaleDateString('en-IN', { day: 'numeric', month: 'short' });
	}
</script>

<svelte:head>
	<title>Admin · grapevine</title>
</svelte:head>

<div class="mx-auto w-full px-8 pb-[120px] pt-10" style="max-width: 920px; animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">
	<Crumbs items={[{ label: 'grapevine', href: '/' }, { label: 'admin' }, { label: 'flagged' }]} />

	{#if loading}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>

	{:else if error || !$currentUser?.is_admin}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
			style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
			Access denied.
		</div>

	{:else}
		<div class="mb-8 flex flex-wrap items-start justify-between gap-6">
			<div>
				<h1 class="mb-2 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: 56px; line-height: 1.05; letter-spacing: -0.015em;">
					Moderator inbox
				</h1>
				<div class="flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
					<span>{items.length + proposed.length} pending items</span>
				</div>
			</div>
			<button
				type="button"
				onclick={doexport}
				disabled={exporting}
				class="inline-flex items-center gap-[6px] self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)] disabled:opacity-50"
			>
				<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
					<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
				</svg>
				{exporting ? 'Exporting…' : 'Export seed data'}
			</button>
		</div>

		<div class="mb-6">
			<Tabs items={[
				{ id: 'flagged', label: 'Flagged reviews', count: items.length },
				{ id: 'proposed', label: 'Proposed offerings', count: proposed.length },
				{ id: 'deleted', label: 'Deleted', count: deleted.length + deletedfaculty.length + deletedlabs.length }
			]} active={activetab} onchange={(id) => { activetab = id as 'flagged' | 'proposed' | 'deleted'; }} />
		</div>

		{#if activetab === 'flagged'}
			{#if items.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					Inbox zero. Nothing flagged.
				</div>
			{:else}
				{#each items as it (it.id)}
					<div
						class="mb-3 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] p-[22px]"
						style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);"
					>
						<!-- head -->
						<div class="mb-3 flex items-center gap-3 text-[12px] text-[var(--fg-3)]">
							<span
								class="rounded px-2 py-[2px] text-[11px] text-[var(--danger)] border border-[rgba(217,138,138,0.2)] bg-[var(--danger-bg)]"
								style="font-family: var(--mono);"
							>{it.reason}</span>

							{#if it.review_type === 'course' && it.course_code}
								<span
									class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] tracking-[0.02em] text-[var(--fg-2)]"
									style="font-family: var(--mono);"
								>{it.course_code}</span>
								{#if it.offering_code}
									<span style="font-family: var(--mono); color: var(--fg-3);">{it.offering_code}</span>
								{/if}
							{/if}

							{#if it.review_type === 'advisor' && it.faculty_name}
								<span>advisor review · <span class="text-[var(--fg-2)]">{it.faculty_name}</span></span>
							{/if}

							<span class="ml-auto text-[var(--fg-4)]" style="font-family: var(--mono); font-size: 11px;">
								flagged {reltime(it.created_at)}
							</span>
						</div>

						<!-- reporter -->
						<div class="mb-3 text-[12px] text-[var(--fg-3)]">
							reported by <span class="text-[var(--fg-2)]">{it.reporter.display_name}</span>
						</div>

						<!-- body -->
						<div class="text-[14px] leading-[1.65] text-[var(--fg)]">{it.review_body}</div>

						<!-- actions -->
						<div class="mt-4 flex gap-2">
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border px-[14px] py-2 text-[13px] whitespace-nowrap transition-[background,border-color] duration-[120ms] border-[rgba(217,138,138,0.2)] bg-[var(--danger-bg)] text-[var(--danger)] hover:bg-[rgba(217,138,138,0.14)] hover:border-[rgba(217,138,138,0.32)]"
								onclick={() => deleteReview(it.id)}
							>Delete review</button>
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] whitespace-nowrap text-[var(--fg)] transition-[background,border-color] duration-[120ms] hover:bg-[var(--bg-3)] hover:border-[var(--border-strong)]"
								onclick={() => dismiss(it.id)}
							>Dismiss flag</button>
						</div>
					</div>
				{/each}
			{/if}
		{:else if activetab === 'proposed'}
			{#if proposed.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					Inbox zero. No proposed offerings.
				</div>
			{:else}
				{#each proposed as p (p.id)}
					<div
						class="mb-3 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] p-[22px]"
						style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);"
					>
						<!-- head -->
						<div class="mb-3 flex items-center gap-3 text-[12px] text-[var(--fg-3)]">
							<span
								class="rounded px-2 py-[2px] text-[11px] text-[var(--accent-2)] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)]"
								style="font-family: var(--mono);"
							>PROPOSED</span>
							<span
								class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] tracking-[0.02em] text-[var(--fg-2)]"
								style="font-family: var(--mono);"
							>{p.course_code}</span>
							<span style="font-family: var(--mono); color: var(--fg-2);">{p.season === 'M' ? 'Monsoon' : 'Spring'} 20{p.year}</span>
							{#if p.faculty && p.faculty.length > 0}
								<span class="text-[var(--fg-4)]">·</span>
								<span class="text-[var(--fg-3)]">Taught by {p.faculty.join(', ')}</span>
							{/if}
						</div>
						<div class="mb-3 text-[13px] font-semibold text-[var(--fg)]">{p.course_name}</div>
						
						{#if p.reviews.length === 0}
							<div class="text-[13px] italic text-[var(--fg-4)] mb-3">Proposed directly (no review content)</div>
						{:else}
							{#each p.reviews as r (r.id)}
								<div class="border-t border-[var(--border)] pt-3 mt-3">
									<div class="flex items-center gap-2 mb-2 text-[11px] text-[var(--fg-3)]">
										<span>Review by {r.author_name ?? 'Anonymous'}</span>
										<span>·</span>
										<span class="text-[11px] text-[var(--fg-4)]">{reltime(r.created_at)}</span>
									</div>
									<div class="grid grid-cols-5 gap-2 text-[11px] text-[var(--fg-2)] mb-2" style="font-family: var(--mono);">
										<div>diff: {r.difficulty}/5</div>
										<div>teach: {r.teaching}/5</div>
										<div>grad: {r.grading}/5</div>
										<div>cont: {r.content}/5</div>
										<div>work: {r.workload}/5</div>
									</div>
									<div class="text-[13px] leading-[1.6] text-[var(--fg-2)]">{r.body}</div>
								</div>
							{/each}
						{/if}

						<!-- actions -->
						<div class="mt-4 flex gap-2">
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border px-[14px] py-2 text-[13px] whitespace-nowrap transition-[background,border-color] duration-[120ms] border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] text-[var(--accent-2)] hover:bg-[rgba(107,143,111,0.14)] hover:border-[rgba(107,143,111,0.32)]"
								onclick={() => approveProp(p.id)}
							>Approve</button>
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] whitespace-nowrap text-[var(--fg)] transition-[background,border-color] duration-[120ms] hover:bg-[var(--bg-3)] hover:border-[var(--border-strong)]"
								onclick={() => rejectProp(p.id)}
							>Reject</button>
						</div>
					</div>
				{/each}
			{/if}
		{:else if activetab === 'deleted'}
			{#if deleted.length + deletedfaculty.length + deletedlabs.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					Nothing deleted.
				</div>
			{:else}
				{#if deleted.length > 0}
					<div class="mb-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Courses</div>
					{#each deleted as d (d.code)}
						<div class="mb-3 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[22px] py-[16px]"
							style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{d.code}</span>
							<span class="flex-1 text-[14px] text-[var(--fg)]">{d.name}</span>
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(d.deleted_at)}</span>
							<button type="button" onclick={() => doRestore(d.code)}
								class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[12px] py-[6px] text-[12px] font-medium text-[var(--accent-2)] transition-colors hover:bg-[rgba(107,143,111,0.14)]"
							>Restore</button>
						</div>
					{/each}
				{/if}
				{#if deletedfaculty.length > 0}
					<div class="mb-2 mt-4 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Faculty</div>
					{#each deletedfaculty as d (d.slug)}
						<div class="mb-3 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[22px] py-[16px]"
							style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{d.slug}</span>
							<span class="flex-1 text-[14px] text-[var(--fg)]">{d.name}</span>
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(d.deleted_at)}</span>
							<button type="button" onclick={() => doRestoreFaculty(d.slug)}
								class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[12px] py-[6px] text-[12px] font-medium text-[var(--accent-2)] transition-colors hover:bg-[rgba(107,143,111,0.14)]"
							>Restore</button>
						</div>
					{/each}
				{/if}
				{#if deletedlabs.length > 0}
					<div class="mb-2 mt-4 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Labs</div>
					{#each deletedlabs as d (d.shortname)}
						<div class="mb-3 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[22px] py-[16px]"
							style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{d.shortname}</span>
							<span class="flex-1 text-[14px] text-[var(--fg)]">{d.name}</span>
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(d.deleted_at)}</span>
							<button type="button" onclick={() => doRestoreLab(d.shortname)}
								class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[12px] py-[6px] text-[12px] font-medium text-[var(--accent-2)] transition-colors hover:bg-[rgba(107,143,111,0.14)]"
							>Restore</button>
						</div>
					{/each}
				{/if}
			{/if}
		{/if}
	{/if}
</div>
