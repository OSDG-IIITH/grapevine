<script lang="ts">
	import { onMount } from 'svelte';
	import { getFlags, dismissFlag, deleteFlaggedReview } from '$lib/api';
	import { currentUser } from '$lib/stores';
	import type { FlagResponse } from '$lib/types';
	import Crumbs from '$lib/components/Crumbs.svelte';

	let items = $state<FlagResponse[]>([]);
	let loading = $state(true);
	let error = $state(false);

	onMount(async () => {
		const flags = await getFlags();
		if (flags === null) error = true;
		else items = flags;
		loading = false;
	});

	async function dismiss(id: string) {
		if (await dismissFlag(id)) items = items.filter((i) => i.id !== id);
	}

	async function deleteReview(id: string) {
		if (await deleteFlaggedReview(id)) items = items.filter((i) => i.id !== id);
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
					Flagged reviews
				</h1>
				<div class="flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
					<span>{items.length} pending</span>
					<span class="text-[var(--fg-4)]">·</span>
					<span>review and act — delete removes the review; dismiss closes the report</span>
				</div>
			</div>
		</div>

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
	{/if}
</div>
