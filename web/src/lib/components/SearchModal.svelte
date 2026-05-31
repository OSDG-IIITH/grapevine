<script lang="ts">
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { searchOpen } from '$lib/stores';
	import { search } from '$lib/api';
	import type { SearchResult } from '$lib/types';
	import { IconBook, IconUser, IconFlask } from '@tabler/icons-svelte';

	let q = $state('');
	let loading = $state(false);
	let results = $state<SearchResult[]>([]);
	let active = $state(-1);
	let timer: ReturnType<typeof setTimeout>;
	let inputel: HTMLInputElement | undefined = $state();

	function href(r: SearchResult) {
		if (r.type === 'course') return `${base}/courses/${r.code}`;
		if (r.type === 'faculty') return `${base}/faculty/${r.slug}`;
		return `${base}/labs/${r.shortname}`;
	}

	$effect(() => {
		if ($searchOpen) {
			tick().then(() => inputel?.focus());
		} else {
			q = '';
			results = [];
			active = -1;
		}
	});

	function oninput() {
		active = -1;
		clearTimeout(timer);
		if (!q.trim()) {
			results = [];
			loading = false;
			return;
		}
		loading = true;
		timer = setTimeout(async () => {
			results = await search(q);
			loading = false;
		}, 200);
	}

	function navigate(r: SearchResult) {
		searchOpen.set(false);
		goto(href(r));
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			active = Math.min(active + 1, results.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			active = Math.max(active - 1, -1);
		} else if (e.key === 'Enter') {
			if (active >= 0 && results[active]) navigate(results[active]);
		} else if (e.key === 'Escape') {
			searchOpen.set(false);
		}
	}

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}
</script>

{#if $searchOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		use:portal
		class="fixed inset-0 z-[200] flex items-start justify-center px-4 pt-[72px]"
		style="background: rgba(0,0,0,0.55); backdrop-filter: blur(4px);"
		onclick={(e) => { if (e.target === e.currentTarget) searchOpen.set(false); }}
	>
		<div
			class="w-full max-w-[560px] overflow-hidden rounded-[12px] border border-[var(--border-2)] bg-[var(--bg-2)] shadow-2xl"
			role="dialog"
			aria-modal="true"
			aria-label="Search"
		>
			<div class="flex items-center gap-3 border-b border-[var(--border)] px-4 py-[13px]">
				<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0 text-[var(--fg-3)]" aria-hidden="true">
					<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
				</svg>
				<input
					bind:this={inputel}
					bind:value={q}
					oninput={oninput}
					onkeydown={onkeydown}
					placeholder="Search courses, faculty, labs…"
					class="flex-1 bg-transparent text-[14px] text-[var(--fg)] outline-none placeholder:text-[var(--fg-4)]"
				/>
				{#if loading}
					<div class="h-[14px] w-[14px] shrink-0 animate-spin rounded-full border-2 border-[var(--border-2)] border-t-[var(--accent-2)]"></div>
				{:else}
					<kbd
						class="shrink-0 rounded border border-[var(--border-2)] bg-[var(--bg-3)] px-[5px] py-[2px] text-[11px] text-[var(--fg-3)]"
						style="font-family: var(--mono);"
					>Esc</kbd>
				{/if}
			</div>

			{#if q.trim()}
				{#if !loading && results.length === 0}
					<div class="px-4 py-[36px] text-center text-[13px] text-[var(--fg-3)]">
						No results for "{q}"
					</div>
				{:else if results.length > 0}
					<div class="max-h-[420px] overflow-y-auto py-2">
						{#each results as r, i (i)}
							<button
								type="button"
								class="flex w-full items-center gap-3 px-4 py-[9px] text-left transition-colors duration-[80ms] {i === active ? 'bg-[var(--bg-3)]' : 'hover:bg-[var(--bg-3)]'}"
								onclick={() => navigate(r)}
								onmouseenter={() => (active = i)}
							>
								<span class="shrink-0 text-[var(--fg-3)]">
									{#if r.type === 'course'}
										<IconBook size={14} />
									{:else if r.type === 'faculty'}
										<IconUser size={14} />
									{:else}
										<IconFlask size={14} />
									{/if}
								</span>
								<span class="flex-1 truncate text-[13px] text-[var(--fg)]">{r.name}</span>
								{#if r.type === 'course' && r.code}
									<span class="shrink-0 text-[11px] text-[var(--fg-3)]" style="font-family: var(--mono);">{r.code}</span>
								{:else if r.type === 'lab' && r.shortname}
									<span class="shrink-0 text-[11px] text-[var(--fg-3)]" style="font-family: var(--mono);">{r.shortname}</span>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			{:else}
				<div class="px-4 py-[20px] text-[13px] text-[var(--fg-4)]">Type to search…</div>
			{/if}
		</div>
	</div>
{/if}
