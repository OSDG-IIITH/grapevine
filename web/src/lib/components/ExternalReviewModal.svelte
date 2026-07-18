<script lang="ts">
	import { untrack } from 'svelte';
	import { getCourse, getFacultyMember, createExternalCourseReview, createExternalAdvisorReview } from '$lib/api';
	import type { ExternalCourseReview, ExternalAdvisorReview } from '$lib/types';

	interface Props {
		/** pre-fill for course page — hides type/target selector */
		coursecode?: string;
		/** pre-fill for faculty page — hides type/target selector */
		facultyslug?: string;
		oncreate: (r: ExternalCourseReview | ExternalAdvisorReview) => void;
		onclose: () => void;
	}

	let { coursecode, facultyslug, oncreate, onclose }: Props = $props();

	const prefilled = $derived(coursecode !== undefined || facultyslug !== undefined);
	let type = $state<'course' | 'advisor'>(untrack(() => facultyslug !== undefined ? 'advisor' : 'course'));
	let target = $state(untrack(() => coursecode ?? facultyslug ?? ''));
	let body = $state('');
	let sourcenote = $state('');
	let submitting = $state(false);
	let error = $state('');

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	async function submit() {
		if (!body.trim()) { error = 'Body is required.'; return; }
		if (!target.trim()) { error = `${type === 'course' ? 'Course code' : 'Faculty slug'} is required.`; return; }
		error = '';
		submitting = true;
		let r: ExternalCourseReview | ExternalAdvisorReview | null = null;
		if (type === 'course') {
			const course = await getCourse(target.trim());
			if (!course) { error = 'Course not found.'; submitting = false; return; }
			r = await createExternalCourseReview({ course_id: course.id, body: body.trim(), source_note: sourcenote.trim() || undefined });
		} else {
			const fac = await getFacultyMember(target.trim());
			if (!fac) { error = 'Faculty member not found.'; submitting = false; return; }
			r = await createExternalAdvisorReview({ faculty_id: fac.id, body: body.trim(), source_note: sourcenote.trim() || undefined });
		}
		submitting = false;
		if (r) oncreate(r);
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	use:portal
	role="presentation"
	class="fixed inset-0 z-[200] flex items-center justify-center p-6"
	style="background: rgba(10,14,12,0.62); backdrop-filter: blur(4px); animation: fadeIn 160ms ease-out;"
	onclick={onclose}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="flex max-h-[calc(100vh-48px)] w-[min(540px,100%)] flex-col overflow-hidden rounded-xl border border-[var(--border-strong)]"
		style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.04), transparent 40%); animation: fadeUp 200ms cubic-bezier(.2,.6,.2,1);"
		onclick={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class="flex items-center justify-between border-b border-[var(--border)] p-[20px_22px_18px]">
			<span class="text-[13px] font-medium text-[var(--fg-2)]">Add External Review</span>
			<button
				type="button"
				aria-label="Close"
				onclick={onclose}
				class="inline-flex h-7 w-7 items-center justify-center rounded-md text-[var(--fg-3)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-4)] hover:text-[var(--fg)]"
			>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
					<path d="M18 6 6 18M6 6l12 12" />
				</svg>
			</button>
		</div>

		<div class="flex-1 overflow-y-auto p-[18px_22px] space-y-4">
			{#if !prefilled}
				<div class="flex gap-2">
					<button
						type="button"
						onclick={() => { type = 'course'; target = ''; error = ''; }}
						class="rounded-[6px] border px-3 py-[5px] text-[12px] transition-colors {type === 'course' ? 'border-[rgba(107,143,111,0.3)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] bg-transparent text-[var(--fg-3)] hover:text-[var(--fg)]'}"
					>Course</button>
					<button
						type="button"
						onclick={() => { type = 'advisor'; target = ''; error = ''; }}
						class="rounded-[6px] border px-3 py-[5px] text-[12px] transition-colors {type === 'advisor' ? 'border-[rgba(107,143,111,0.3)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] bg-transparent text-[var(--fg-3)] hover:text-[var(--fg)]'}"
					>Advisor</button>
				</div>

				<div>
					<label for="ext-target" class="mb-1 block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">
						{type === 'course' ? 'Course code' : 'Faculty slug'}
					</label>
					<input
						id="ext-target"
						type="text"
						placeholder={type === 'course' ? 'e.g. CS5.301' : 'e.g. prof-name'}
						bind:value={target}
						class="w-full rounded-[7px] border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-[13px] text-[var(--fg)] placeholder-[var(--fg-4)] outline-none focus:border-[var(--border-strong)]"
					/>
				</div>
			{/if}

			<div>
				<label for="ext-sourcenote" class="mb-1 block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Source note <span class="normal-case text-[var(--fg-4)]">(optional)</span></label>
				<input
					id="ext-sourcenote"
					type="text"
					placeholder="e.g. Discord thread, internal form"
					bind:value={sourcenote}
					class="w-full rounded-[7px] border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-[13px] text-[var(--fg)] placeholder-[var(--fg-4)] outline-none focus:border-[var(--border-strong)]"
				/>
			</div>

			<div>
				<label for="ext-body" class="mb-1 block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Review</label>
				<textarea
					id="ext-body"
					rows={6}
					placeholder="Paste the review text…"
					bind:value={body}
					class="w-full resize-none rounded-[7px] border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-[13px] leading-[1.65] text-[var(--fg)] placeholder-[var(--fg-4)] outline-none focus:border-[var(--border-strong)]"
				></textarea>
			</div>

			{#if error}
				<p class="text-[12px] text-[var(--danger)]">{error}</p>
			{/if}
		</div>

		<div class="flex items-center justify-end gap-2 border-t border-[var(--border)] p-[12px_22px]">
			<button
				type="button"
				onclick={onclose}
				class="rounded-[7px] border border-[var(--border)] bg-transparent px-4 py-[7px] text-[13px] text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-3)]"
			>Cancel</button>
			<button
				type="button"
				onclick={submit}
				disabled={submitting}
				class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[14px] py-[7px] text-[13px] font-medium text-[var(--accent-2)] transition-colors hover:bg-[rgba(107,143,111,0.14)] disabled:opacity-50"
			>{submitting ? 'Adding…' : 'Add review'}</button>
		</div>
	</div>
</div>
