<script lang="ts">
	import { base } from '$app/paths';
	import { goto } from '$app/navigation';
	import { getLabs, createLab } from '$lib/api';
	import type { LabLean } from '$lib/types';
	import { currentUser } from '$lib/stores';
	import BrowseCard from '$lib/components/BrowseCard.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import { toast } from 'svelte-sonner';

	const PER_PAGE = 9;

	let all = $state<LabLean[]>([]);
	let q = $state('');
	let page = $state(1);

	let adding = $state(false);
	let newname = $state('');
	let newshort = $state('');
	let saving = $state(false);

	$effect(() => {
		getLabs().then((data) => { if (Array.isArray(data)) all = data; });
	});

	const filtered = $derived(
		all.filter((l) => {
			if (!q.trim()) return true;
			const s = q.toLowerCase();
			return l.name.toLowerCase().includes(s) || l.short.toLowerCase().includes(s);
		})
	);

	const totalpages = $derived(Math.max(1, Math.ceil(filtered.length / PER_PAGE)));
	const visible = $derived(filtered.slice((page - 1) * PER_PAGE, page * PER_PAGE));

	function openadding() {
		newname = '';
		newshort = '';
		adding = true;
	}

	async function submitnew(e: SubmitEvent) {
		e.preventDefault();
		if (!newname.trim() || !newshort.trim()) return;
		saving = true;
		const res = await createLab({ name: newname.trim(), short: newshort.trim().toUpperCase() });
		saving = false;
		if (res) {
			toast.success('Lab created.');
			adding = false;
			goto(`${base}/labs/${res.short}`);
		}
	}
</script>

<svelte:head>
	<title>Labs · grapevine</title>
</svelte:head>

<div class="mx-auto w-full max-w-[1180px] px-4 pb-[120px] pt-10 sm:px-8" style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">

	<!-- breadcrumbs -->
	<div class="mb-[18px] flex items-center gap-2 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">
		<a href="{base}/" class="transition-colors duration-[120ms] hover:text-[var(--fg)]">grapevine</a>
		<span class="text-[var(--fg-4)]">/</span>
		<span class="text-[var(--fg-2)]">labs</span>
	</div>

	<!-- page head -->
	<div class="flex flex-wrap items-start justify-between gap-6">
		<div>
			<h1 class="m-0 mb-4 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: clamp(30px, 5vw, 56px); line-height: 1.05; letter-spacing: -0.015em;">
				Research Labs
			</h1>
			<div class="mb-[22px] flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
				<span>{all.length} research centers</span>
				<span class="text-[var(--fg-4)]">·</span>
				<span>aggregated advisor ratings across all members</span>
			</div>
		</div>
		{#if $currentUser?.is_admin}
			<button
				type="button"
				onclick={openadding}
				class="inline-flex items-center gap-2 self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
			>
				<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
					<path d="M5 12h14M12 5v14" />
				</svg>
				Add lab
			</button>
		{/if}
	</div>

	<!-- toolbar -->
	<div class="mb-[22px] mt-[18px]">
		<div class="flex w-full items-center gap-2 rounded-[7px] border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px] sm:w-[260px]">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 text-[var(--fg-4)]" aria-hidden="true">
				<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
			</svg>
			<input
				class="w-full bg-transparent text-[13px] outline-none placeholder:text-[var(--fg-4)]"
				placeholder="Search labs…"
				bind:value={q}
				oninput={() => (page = 1)}
			/>
		</div>
	</div>

	<!-- grid -->
	{#if visible.length === 0}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
			No labs match this search.
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-[14px] sm:grid-cols-2 lg:grid-cols-3">
			{#each visible as l (l.id)}
				<BrowseCard kind="lab" item={l} />
			{/each}
		</div>
	{/if}

	<Pager {page} totalpages={totalpages} totalitems={filtered.length} onchange={(p) => (page = p)} />
</div>

<!-- add lab modal -->
{#if adding}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-[2px]"
		onkeydown={(e) => { if (e.key === 'Escape') adding = false; }}
		onclick={(e) => { if (e.target === e.currentTarget) adding = false; }}
	>
		<div class="w-full max-w-[420px] rounded-[12px] border border-[var(--border-strong)] bg-[var(--bg-2)] p-6 shadow-2xl">
			<div class="mb-5 flex items-center justify-between">
				<h2 class="m-0 text-[16px] font-medium text-[var(--fg)]">Add lab</h2>
				<button
					type="button"
					onclick={() => (adding = false)}
					aria-label="Close"
					class="flex h-7 w-7 items-center justify-center rounded-[5px] text-[var(--fg-3)] transition-colors hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
				>
					<svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
						<path d="M1 1l12 12M13 1L1 13" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
					</svg>
				</button>
			</div>

			<form onsubmit={submitnew} class="flex flex-col gap-[14px]">
				<div class="flex flex-col gap-[6px]">
					<label class="text-[11px] text-[var(--fg-3)]" for="newname">Name</label>
					<input
						id="newname"
						bind:value={newname}
						required
						class="rounded-[6px] border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px] text-[13px] text-[var(--fg)] outline-none focus:border-[var(--accent-dim)]"
					/>
				</div>

				<div class="flex flex-col gap-[6px]">
					<label class="text-[11px] text-[var(--fg-3)]" for="newshort">Short name</label>
					<input
						id="newshort"
						bind:value={newshort}
						required
						class="rounded-[6px] border border-[var(--border)] bg-[var(--bg-inset)] px-3 py-[7px] text-[13px] text-[var(--fg)] outline-none focus:border-[var(--accent-dim)]"
						style="font-family: var(--mono);"
					/>
				</div>

				<div class="mt-1 flex justify-end gap-2">
					<button
						type="button"
						onclick={() => (adding = false)}
						class="rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={saving || !newname.trim() || !newshort.trim()}
						class="inline-flex items-center gap-2 rounded-[7px] px-[14px] py-2 text-[13px] font-medium transition-[background,border-color] duration-[120ms] disabled:opacity-60"
						style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
					>
						{saving ? 'Creating…' : 'Create lab'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
