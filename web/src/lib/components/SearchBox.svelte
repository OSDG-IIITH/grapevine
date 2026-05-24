<script lang="ts">
	import { goto } from '$app/navigation';
	import { getCourses, getFaculty, getLabs } from '$lib/api';
	import type { CourseLean, FacultyLean, LabLean } from '$lib/types';
	import { IconBook, IconUser, IconFlask } from '@tabler/icons-svelte';

	interface Hint {
		label: string;
		href: string;
	}

	interface Props {
		hints?: Hint[];
		placeholder?: string;
	}

	let { hints = [], placeholder = 'Search a course, faculty, or lab…' }: Props = $props();

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
	let container: HTMLDivElement | undefined = $state();

	const results = $derived<ResultItem[]>([
		...courses.map((d) => ({ kind: 'course' as const, data: d, href: `/courses/${d.code}` })),
		...faculty.map((d) => ({ kind: 'faculty' as const, data: d, href: `/faculty/${d.slug}` })),
		...labs.map((d) => ({ kind: 'lab' as const, data: d, href: `/labs/${d.short}` })),
	]);

	const hasquery = $derived(q.trim().length > 0);

	const sections: { label: string; kind: ResultItem['kind'] }[] = [
		{ label: 'Courses', kind: 'course' },
		{ label: 'Faculty', kind: 'faculty' },
		{ label: 'Labs', kind: 'lab' },
	];

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
			const [c, f, l] = await Promise.all([getCourses({ q }), getFaculty({ q }), getLabs(q)]);
			courses = c ?? [];
			faculty = f ?? [];
			labs = l ?? [];
			loading = false;
		}, 200);
	}

	function navigate(href: string) {
		q = '';
		courses = [];
		faculty = [];
		labs = [];
		goto(href);
	}

	function onkeydown(e: KeyboardEvent) {
		if (!hasquery) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			active = Math.min(active + 1, results.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			active = Math.max(active - 1, -1);
		} else if (e.key === 'Enter' && active >= 0 && results[active]) {
			navigate(results[active].href);
		} else if (e.key === 'Escape') {
			q = '';
			courses = [];
			faculty = [];
			labs = [];
		}
	}

	function onwindowclick(e: MouseEvent) {
		if (container && !container.contains(e.target as Node)) {
			q = '';
			courses = [];
			faculty = [];
			labs = [];
		}
	}
</script>

<svelte:window onclick={onwindowclick} />

<div bind:this={container} class="w-full">
	<!-- input + results anchored together -->
	<div class="relative">
	<!-- input bar -->
	<div
		class="relative z-[1] flex w-full items-center gap-[14px] rounded-xl border border-[var(--border-strong)] bg-[var(--bg-2)] px-[22px] py-[18px] text-[16px] text-[var(--fg)] {hasquery ? 'rounded-b-none border-b-[var(--border)]' : ''}"
	>
		<svg
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="shrink-0 text-[var(--fg-3)]"
			aria-hidden="true"
		>
			<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
		</svg>
		<input
			class="flex-1 border-0 bg-transparent text-[16px] outline-none placeholder:text-[var(--fg-4)]"
			{placeholder}
			bind:value={q}
			oninput={oninput}
			onkeydown={onkeydown}
			autocomplete="off"
			spellcheck="false"
		/>
		{#if loading}
			<div class="h-[15px] w-[15px] shrink-0 animate-spin rounded-full border-2 border-[var(--border-2)] border-t-[var(--accent-2)]"></div>
		{/if}
	</div>

	<!-- results panel -->
	{#if hasquery}
		<div
			class="absolute left-0 right-0 top-full z-[2] overflow-hidden rounded-b-xl border border-t-0 border-[var(--border-strong)] bg-[var(--bg-2)]"
			style="box-shadow: 0 16px 40px rgba(0,0,0,0.4);"
		>
			{#if loading && results.length === 0}
				<div class="px-6 py-4 text-[14px] text-[var(--fg-4)]">Searching…</div>
			{:else if !loading && results.length === 0}
				<div class="px-6 py-5 text-[14px] text-[var(--fg-3)]">No results for "{q}"</div>
			{:else}
				<div class="py-2">
					{#each sections as sec (sec.kind)}
						{@const items = results.filter((r) => r.kind === sec.kind)}
						{#if items.length > 0}
							<div
								class="px-6 pb-[5px] pt-[10px] text-[10px] font-medium uppercase tracking-[0.08em] text-[var(--fg-4)]"
								style="font-family: var(--mono);"
							>
								{sec.label}
							</div>
							{#each items as item (item.data.id)}
								{@const idx = results.indexOf(item)}
								<button
									type="button"
									class="flex w-full items-center gap-3 px-6 py-[10px] text-left transition-colors duration-[80ms] {idx === active ? 'bg-[var(--bg-3)]' : 'hover:bg-[var(--bg-3)]'}"
									onclick={() => navigate(item.href)}
									onmouseenter={() => (active = idx)}
								>
									<span class="shrink-0 text-[var(--fg-3)]">
										{#if item.kind === 'course'}
											<IconBook size={15} />
										{:else if item.kind === 'faculty'}
											<IconUser size={15} />
										{:else}
											<IconFlask size={15} />
										{/if}
									</span>
									<span class="flex-1 truncate text-[14px] text-[var(--fg)]">{item.data.name}</span>
									{#if item.kind === 'course'}
										<span class="shrink-0 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">{item.data.code}</span>
									{:else if item.kind === 'lab'}
										<span class="shrink-0 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">{item.data.short}</span>
									{/if}
								</button>
							{/each}
						{/if}
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	</div><!-- end input+results wrapper -->

	<!-- hints (always in flow; sits below the input, results panel overlays via absolute) -->
	{#if hints.length > 0}
		<div
			class="mt-[18px] flex flex-wrap justify-center gap-[6px] text-[11px] text-[var(--fg-3)]"
			style="font-family: var(--mono);"
		>
			{#each hints as h (h.href)}
				<a
					href={h.href}
					class="rounded-[5px] border border-[var(--border-2)] px-[11px] py-[5px] transition-all duration-[150ms] hover:border-[var(--accent-line)] hover:bg-[var(--accent-bg)] hover:text-[var(--accent-2)] hover:shadow-[0_0_10px_rgba(107,143,111,0.12)]"
				>
					{h.label}
				</a>
			{/each}
		</div>
	{/if}
</div>
