<script lang="ts">
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import type { Draft } from '$lib/types';
	import { COURSE_AXIS_ORDER, ADVISOR_AXIS_ORDER } from '$lib/types';
	import { deleteDraft } from '$lib/api';
	import { ago } from '$lib/utils';
	import IconTrash from '@tabler/icons-svelte/icons/trash';

	interface Props {
		draft: Draft;
		ondelete?: (id: string) => void;
	}

	let { draft, ondelete }: Props = $props();

	let confirming = $state(false);
	let deleting = $state(false);

	const iscourse = $derived(!!draft.course_id);
	const axisorder: readonly string[] = $derived(iscourse ? COURSE_AXIS_ORDER : ADVISOR_AXIS_ORDER);
	const rated = $derived(axisorder.filter((k) => draft.data.axes?.[k]).length);
	const text = $derived((draft.data.body ?? '').replace(/[*_`#>~\[\]]/g, '').trim());
	const href = $derived(
		iscourse
			? `${base}/review?course=${encodeURIComponent(draft.course_code ?? '')}`
			: `${base}/review?faculty=${encodeURIComponent(draft.faculty_slug ?? '')}`
	);

	async function remove(e: Event) {
		e.stopPropagation();
		if (!confirming) {
			confirming = true;
			setTimeout(() => (confirming = false), 3000);
			return;
		}
		deleting = true;
		const ok = await deleteDraft(draft.id);
		deleting = false;
		if (ok) ondelete?.(draft.id);
	}
</script>

<div
	role="button"
	tabindex="0"
	class="group relative flex flex-col gap-[10px] overflow-hidden rounded-[10px] border border-[var(--border)] p-[14px_16px] text-left transition-[background,border-color] duration-[160ms] hover:border-[var(--border-2)] hover:bg-[var(--bg-3)] cursor-pointer"
	style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.03), transparent 42%);"
	onclick={() => goto(href)}
	onkeydown={(e) => e.key === 'Enter' && goto(href)}
>
	<div
		class="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-[180ms] group-hover:opacity-100"
		style="background: radial-gradient(ellipse 220px 110px at 100% 0%, rgba(107,143,111,0.05), transparent 70%);"
	></div>

	<div class="flex min-w-0 items-center justify-between gap-2">
		<div class="flex min-w-0 items-center gap-[5px] text-[11px] tracking-[0.04em] text-[var(--fg-3)]" style="font-family: var(--mono);">
			<span class="truncate">{iscourse ? draft.course_code : 'advisor'}</span>
			<span class="text-[var(--fg-4)]">·</span>
			<span class="shrink-0">{ago(draft.updated_at)}</span>
		</div>
		<div class="inline-flex shrink-0 items-center gap-[2px]" title="{rated} of {axisorder.length} rated">
			{#each axisorder as k (k)}
				<span class="h-[5px] w-[9px] rounded-[1px] {draft.data.axes?.[k] ? 'bg-[var(--accent)]' : 'bg-[var(--border-strong)]'}"></span>
			{/each}
			<span class="ml-[5px] text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{rated}/{axisorder.length}</span>
		</div>
	</div>

	<div class="truncate text-[14px] font-medium text-[var(--fg)]">{iscourse ? draft.course_name : draft.faculty_name}</div>

	<div class="line-clamp-2 flex-1 text-[13px] leading-[1.55] {text ? 'text-[var(--fg-2)]' : 'italic text-[var(--fg-4)]'}">
		{text || 'No written review yet.'}
	</div>

	<div class="mt-auto flex items-center justify-between">
		<span class="text-[12px] text-[var(--fg-3)] transition-colors duration-[120ms] group-hover:text-[var(--accent-2)]" style="font-family: var(--mono);">continue →</span>
		<button
			type="button"
			aria-label="Discard draft"
			onclick={remove}
			disabled={deleting}
			class="relative z-[2] inline-flex h-7 min-w-7 items-center justify-center gap-[5px] rounded-[5px] px-[7px] text-[12px] transition-colors duration-[120ms] {confirming ? 'bg-[var(--danger-bg)] text-[var(--danger)]' : 'text-[var(--fg-3)] hover:text-[var(--danger)]'}"
			style="font-family: var(--mono);"
		>
			<IconTrash size={13} stroke={1.7} />
			{#if confirming}discard?{/if}
		</button>
	</div>
</div>
