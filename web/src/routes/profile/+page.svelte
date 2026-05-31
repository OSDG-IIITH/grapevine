<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { toast } from 'svelte-sonner';
	import { currentUser } from '$lib/stores';
	import { getMe, updateMe, getMyReviews } from '$lib/api';
	import type { MyReviews } from '$lib/types';
	import { COURSE_AXIS_ORDER, COURSE_AXIS_LABELS, ADVISOR_AXIS_ORDER, ADVISOR_AXIS_LABELS } from '$lib/types';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import Tabs from '$lib/components/Tabs.svelte';
	import ReviewCard from '$lib/components/ReviewCard.svelte';

	let reviews = $state<MyReviews | null>(null);
	let tab = $state('course');
	let editopen = $state(false);
	let editname = $state('');
	let saving = $state(false);

	onMount(async () => {
		let user = $currentUser;
		if (!user) {
			user = await getMe();
			if (user) currentUser.set(user);
		}
		if (!user) { goto(base + '/'); return; }
		const data = await getMyReviews();
		if (data) reviews = data;
	});

	function openedit() {
		editname = $currentUser?.display_name ?? '';
		editopen = true;
	}

	async function save(e: SubmitEvent) {
		e.preventDefault();
		const name = editname.trim();
		if (!name || saving) return;
		saving = true;
		const updated = await updateMe(name);
		saving = false;
		if (updated) {
			currentUser.set(updated);
			editopen = false;
			toast.success('Display name updated');
		}
	}

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	const tabs = $derived([
		{ id: 'course', label: 'Course reviews', count: reviews?.course.length ?? 0 },
		{ id: 'advisor', label: 'Advisor reviews', count: reviews?.advisor.length ?? 0 }
	]);

	const shown = $derived(tab === 'course' ? (reviews?.course ?? []) : (reviews?.advisor ?? []));
</script>

<div class="mx-auto w-full max-w-[1180px] px-4 pb-[120px] pt-10 sm:px-8" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">
	<Crumbs items={[{ label: 'grapevine', href: '/' }, { label: 'profile' }]} />

	<div class="mb-8 flex items-start justify-between gap-4">
		<h1
			class="m-0 font-normal text-[var(--fg)]"
			style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;"
		>
			{$currentUser?.display_name ?? ''}
		</h1>
		<button
			type="button"
			aria-label="Edit display name"
			onclick={openedit}
			class="mt-2 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-[7px] border border-[var(--border)] text-[var(--fg-3)] transition-[color,background,border-color] duration-[120ms] hover:border-[var(--border-2)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
		>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
				<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
				<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
			</svg>
		</button>
	</div>

	<Tabs items={tabs} active={tab} onchange={(id) => (tab = id)} />

	{#if reviews === null}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
	{:else if shown.length === 0}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
			No reviews yet.
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-[12px] md:grid-cols-2">
			{#each shown as r (r.id)}
				{#if tab === 'course'}
					<ReviewCard
						review={r}
						axisorder={[...COURSE_AXIS_ORDER]}
						axislabels={COURSE_AXIS_LABELS}
						ondelete={(id) => { if (reviews) reviews.course = reviews.course.filter((x) => x.id !== id); }}
					/>
				{:else}
					<ReviewCard
						review={r}
						axisorder={[...ADVISOR_AXIS_ORDER]}
						axislabels={ADVISOR_AXIS_LABELS}
						ondelete={(id) => { if (reviews) reviews.advisor = reviews.advisor.filter((x) => x.id !== id); }}
					/>
				{/if}
			{/each}
		</div>
	{/if}
</div>

{#if editopen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		use:portal
		role="presentation"
		class="fixed inset-0 z-[210] flex items-center justify-center p-6"
		style="background: rgba(10,14,12,0.62); backdrop-filter: blur(4px); animation: fadeIn 160ms ease-out;"
		onclick={() => (editopen = false)}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			role="dialog"
			aria-modal="true"
			tabindex="-1"
			class="flex w-[min(420px,100%)] flex-col overflow-hidden rounded-xl border border-[var(--border-strong)]"
			style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.04), transparent 40%); animation: fadeUp 200ms cubic-bezier(.2,.6,.2,1);"
			onclick={(e) => e.stopPropagation()}
		>
			<form onsubmit={save}>
				<div class="flex items-center justify-between gap-3 border-b border-[var(--border)] p-[18px_20px]">
					<div class="text-[24px] text-[var(--fg)]" style="font-family: var(--serif);">Edit display name</div>
					<button
						type="button"
						aria-label="Close"
						onclick={() => (editopen = false)}
						class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-[var(--fg-3)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-4)] hover:text-[var(--fg)]"
					>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
							<path d="M18 6 6 18M6 6l12 12" />
						</svg>
					</button>
				</div>

				<div class="p-[18px_20px_20px]">
					<label class="block text-[12px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="editname">Display name</label>
					<input
						id="editname"
						type="text"
						bind:value={editname}
						maxlength="80"
						placeholder="Your name"
						autofocus
						class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
					/>
				</div>

				<div class="flex items-center justify-end gap-2 border-t border-[var(--border)] p-[12px_20px]">
					<button
						type="button"
						onclick={() => (editopen = false)}
						class="inline-flex items-center rounded-[7px] border border-transparent px-[12px] py-[6px] text-[12px] text-[var(--fg-2)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
						style="font-family: var(--mono);"
					>Cancel</button>
					<button
						type="submit"
						disabled={saving || !editname.trim()}
						class="inline-flex items-center rounded-[7px] border px-[12px] py-[6px] text-[12px] font-semibold transition-[background,border-color,opacity] duration-[120ms] disabled:cursor-not-allowed disabled:opacity-50"
						style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
					>
						{saving ? 'Saving…' : 'Save'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
