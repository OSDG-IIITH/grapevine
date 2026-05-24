<script lang="ts">
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { searchOpen } from '$lib/stores';
	import { getCourses, getFaculty, getLabs } from '$lib/api';
	import type { CourseLean, FacultyLean, LabLean } from '$lib/types';
	import { IconBook, IconUser, IconFlask } from '@tabler/icons-svelte';

	type ResultItem =
		| { kind: 'course'; data: CourseLean; href: string }
		| { kind: 'faculty'; data: FacultyLean; href: string }
		| { kind: 'lab'; data: LabLean; href: string };

	let q = $state('');
	let loading = $state(false);
	let courses = $state<CourseLean[]>([]);
	let faculty = $state<FacultyLean[]>([]);
	let labs = $state<LabLean[]>([]);
	let active = $state(-1);
	let timer: ReturnType<typeof setTimeout>;
	let inputel: HTMLInputElement | undefined = $state();

	const results = $derived<ResultItem[]>([
		...courses.map((d) => ({ kind: 'course' as const, data: d, href: `/courses/${d.code}` })),
		...faculty.map((d) => ({ kind: 'faculty' as const, data: d, href: `/faculty/${d.slug}` })),
		...labs.map((d) => ({ kind: 'lab' as const, data: d, href: `/labs/${d.short}` })),
	]);

	$effect(() => {
		if ($searchOpen) {
			tick().then(() => inputel?.focus());
		} else {
			q = '';
			courses = [];
			faculty = [];
			labs = [];
			active = -1;
		}
	});

	function oninput() {
		active = -1;
		clearTimeout(timer);
		if (!q.trim()) {
			courses = [];
			faculty = [];
			labs = [];
			loading = false;
			return;
		}
		loading = true;
		timer = setTimeout(async () => {
			const [c, f, l] = await Promise.all([
				getCourses({ q }),
				getFaculty({ q }),
				getLabs(q),
			]);
			courses = c ?? [];
			faculty = f ?? [];
			labs = l ?? [];
			loading = false;
		}, 200);
	}

	function navigate(href: string) {
		searchOpen.set(false);
		goto(href);
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			active = Math.min(active + 1, results.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			active = Math.max(active - 1, -1);
		} else if (e.key === 'Enter') {
			if (active >= 0 && results[active]) navigate(results[active].href);
		} else if (e.key === 'Escape') {
			searchOpen.set(false);
		}
	}

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	const sections: { label: string; kind: ResultItem['kind'] }[] = [
		{ label: 'Courses', kind: 'course' },
		{ label: 'Faculty', kind: 'faculty' },
		{ label: 'Labs', kind: 'lab' },
	];
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
						{#each sections as sec (sec.kind)}
							{@const items = results.filter((r) => r.kind === sec.kind)}
							{#if items.length > 0}
								<div
									class="px-4 pb-[6px] pt-3 text-[10px] font-medium uppercase tracking-[0.08em] text-[var(--fg-4)]"
									style="font-family: var(--mono);"
								>
									{sec.label}
								</div>
								{#each items as item (item.data.id)}
									{@const idx = results.indexOf(item)}
									<button
										type="button"
										class="flex w-full items-center gap-3 px-4 py-[9px] text-left transition-colors duration-[80ms] {idx === active ? 'bg-[var(--bg-3)]' : 'hover:bg-[var(--bg-3)]'}"
										onclick={() => navigate(item.href)}
										onmouseenter={() => (active = idx)}
									>
										<span class="shrink-0 text-[var(--fg-3)]">
											{#if item.kind === 'course'}
												<IconBook size={14} />
											{:else if item.kind === 'faculty'}
												<IconUser size={14} />
											{:else}
												<IconFlask size={14} />
											{/if}
										</span>
										<span class="flex-1 truncate text-[13px] text-[var(--fg)]">{item.data.name}</span>
										{#if item.kind === 'course'}
											<span class="shrink-0 text-[11px] text-[var(--fg-3)]" style="font-family: var(--mono);">{item.data.code}</span>
										{:else if item.kind === 'lab'}
											<span class="shrink-0 text-[11px] text-[var(--fg-3)]" style="font-family: var(--mono);">{item.data.short}</span>
										{/if}
									</button>
								{/each}
							{/if}
						{/each}
					</div>
				{/if}
			{:else}
				<div class="px-4 py-[20px] text-[13px] text-[var(--fg-4)]">Type to search…</div>
			{/if}
		</div>
	</div>
{/if}
