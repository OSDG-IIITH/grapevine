<script lang="ts">
	import type { CourseDetail, CourseReportSubmission, FacultyLean } from '$lib/types';
	import Combobox from '$lib/components/Combobox.svelte';

	interface Props {
		course: CourseDetail;
		faculty: FacultyLean[];
		initialofferingid?: string;
		submitting?: boolean;
		onclose: () => void;
		onsubmit: (submission: CourseReportSubmission) => void;
	}

	let { course, faculty, initialofferingid, submitting = false, onclose, onsubmit }: Props = $props();
	let mode = $state<'course' | 'offering' | null>(null);
	let reason = $state('');
	let offeringchoice = $state('');
	let correctedids = $state<string[]>([]);
	let initialized = $state(false);
	let facultypick = $state('');

	const selectedoffering = $derived(course.offerings.find((offering) => offering.id === offeringchoice));
	const reasonlength = $derived([...reason.trim()].length);
	const availablefaculty = $derived(faculty.filter((member) => !correctedids.includes(member.id)));
	const facultychanged = $derived(selectedoffering ? differentsets(selectedoffering.faculty.map((member) => member.id), correctedids) : false);
	const canSubmit = $derived.by(() => {
		if (submitting || !mode) return false;
		if (mode === 'course') return reasonlength >= 3 && reasonlength <= 1000;
		if (!selectedoffering) return false;
		const validreason = reasonlength === 0 || (reasonlength >= 3 && reasonlength <= 1000);
		return validreason && (facultychanged || reasonlength >= 3);
	});

	$effect(() => {
		if (!initialized) {
			selectoffering(initialofferingid ?? course.offerings[0]?.id ?? '');
			initialized = true;
		}
	});

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	function differentsets(left: string[], right: string[]): boolean {
		return left.length !== right.length || left.some((id) => !right.includes(id));
	}

	function selectmode(next: 'course' | 'offering') {
		mode = next;
		reason = '';
		if (next === 'offering') selectoffering(offeringchoice);
	}

	function selectoffering(id: string) {
		offeringchoice = id;
		reason = '';
		facultypick = '';
		const offering = course.offerings.find((item) => item.id === id);
		correctedids = offering ? offering.faculty.map((member) => member.id) : [];
	}

	function addfaculty(id: string) {
		if (id && !correctedids.includes(id)) correctedids = [...correctedids, id];
		facultypick = '';
	}

	function removefaculty(id: string) {
		correctedids = correctedids.filter((facultyid) => facultyid !== id);
	}

	function facultyname(id: string): string {
		return faculty.find((member) => member.id === id)?.name
			?? selectedoffering?.faculty.find((member) => member.id === id)?.name
			?? id;
	}

	function submit() {
		if (!canSubmit) return;
		if (mode === 'course') {
			onsubmit({ kind: 'course', reason: reason.trim() });
		} else if (selectedoffering) {
			onsubmit({ kind: 'offering', offering_id: offeringchoice, reason: reason.trim(), faculty_ids: facultychanged ? correctedids : undefined });
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div use:portal role="presentation" class="fixed inset-0 z-[210] flex items-center justify-center p-4 sm:p-6" style="background: rgba(10,14,12,0.62); backdrop-filter: blur(4px); animation: fadeIn 160ms ease-out;" onclick={onclose}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="flex max-h-[min(760px,calc(100vh-32px))] w-[min(680px,100%)] flex-col overflow-hidden rounded-xl border border-[var(--border-strong)]" style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.04), transparent 40%); animation: fadeUp 200ms cubic-bezier(.2,.6,.2,1);" onclick={(event) => event.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="course-report-title" tabindex="-1">
		<div class="flex items-start justify-between gap-3 border-b border-[var(--border)] p-[18px_20px]">
			<div>
				<div id="course-report-title" class="text-[24px] text-[var(--fg)]" style="font-family: var(--serif);">Report course information</div>
				<div class="mt-1 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">{course.code} · {course.name}</div>
			</div>
			<button type="button" aria-label="Close" onclick={onclose} class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-[var(--fg-3)] transition-colors hover:bg-[var(--bg-4)] hover:text-[var(--fg)]">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 6 6 18M6 6l12 12" /></svg>
			</button>
		</div>

		<div class="overflow-y-auto p-[18px_20px_22px]">
			<div class="grid gap-2 sm:grid-cols-2">
				<button type="button" onclick={() => selectmode('course')} class="rounded-[9px] border p-4 text-left transition-colors {mode === 'course' ? 'border-[var(--accent)] bg-[var(--accent-bg)]' : 'border-[var(--border-2)] hover:border-[var(--border-strong)] hover:bg-[var(--bg-3)]'}">
					<span class="block text-[14px] font-medium text-[var(--fg)]">Course details</span>
					<span class="mt-1 block text-[12px] leading-[1.5] text-[var(--fg-3)]">Code, name, description, aliases, or succession.</span>
				</button>
				<button type="button" onclick={() => selectmode('offering')} disabled={course.offerings.length === 0} class="rounded-[9px] border p-4 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 {mode === 'offering' ? 'border-[var(--accent)] bg-[var(--accent-bg)]' : 'border-[var(--border-2)] hover:border-[var(--border-strong)] hover:bg-[var(--bg-3)]'}">
					<span class="block text-[14px] font-medium text-[var(--fg)]">Offering details</span>
					<span class="mt-1 block text-[12px] leading-[1.5] text-[var(--fg-3)]">Correct instructors for an existing semester.</span>
				</button>
			</div>

			{#if mode === 'course'}
				<div class="mt-5">
					<label for="course-report-reason" class="block text-[12px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">What information is incorrect?</label>
					<textarea id="course-report-reason" rows="5" bind:value={reason} placeholder="Describe what is inaccurate and include the correct information if you know it." class="mt-2 w-full resize-none rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[12px] text-[14px] leading-[1.6] text-[var(--fg)] outline-none transition-colors placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"></textarea>
					<div class="mt-1 text-right text-[10px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reasonlength}/1000</div>
				</div>
			{:else if mode === 'offering'}
				<div class="mt-5">
					<label for="offering-report-select" class="block text-[12px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Offering</label>
					<select id="offering-report-select" value={offeringchoice} onchange={(event) => selectoffering(event.currentTarget.value)} class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[12px] py-[10px] text-[13px] text-[var(--fg)] outline-none focus:border-[var(--accent)]">
						{#each course.offerings as offering (offering.id)}
							<option value={offering.id}>{offering.code} · {offering.season === 'M' ? 'Monsoon' : 'Spring'} {offering.year}</option>
						{/each}
					</select>
				</div>

				{#if selectedoffering}
					<div class="mt-5 text-[12px] text-[var(--fg-3)]">
						Current instructors: <span class="text-[var(--fg-2)]">{selectedoffering.faculty.length ? selectedoffering.faculty.map((member) => member.name).join(', ') : 'None listed'}</span>
					</div>
				{/if}

				<div class="mt-5">
					<div class="flex items-center justify-between gap-3">
						<span class="text-[12px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Correct instructors</span>
					</div>
					<div class="mt-2 flex flex-wrap items-center gap-2">
						{#each correctedids as id (id)}
							<span class="inline-flex items-center gap-2 rounded-[6px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-2 py-[6px] text-[12px] text-[var(--fg-2)]">
								{facultyname(id)}
								<button type="button" onclick={() => removefaculty(id)} aria-label="Remove {facultyname(id)}" class="text-[var(--fg-4)] transition-colors hover:text-[var(--fg)]">×</button>
							</span>
						{/each}
						{#if availablefaculty.length > 0}
							<Combobox items={availablefaculty.map((member) => ({ value: member.id, label: member.name }))} bind:value={facultypick} onselect={addfaculty} placeholder="+ Add instructor" searchplaceholder="Search professors…" popoverwidth="280px" class="rounded-[6px] border border-dashed border-[var(--border-strong)] bg-transparent px-3 py-[7px] text-[12px] text-[var(--fg-3)] hover:text-[var(--fg)]" />
						{/if}
					</div>
				</div>

				<div class="mt-5">
					<label for="offering-report-reason" class="block text-[12px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Additional details <span class="normal-case tracking-normal text-[var(--fg-4)]">(optional)</span></label>
					<textarea id="offering-report-reason" rows="3" bind:value={reason} placeholder="Add context about what is inaccurate." class="mt-2 w-full resize-none rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[12px] text-[14px] leading-[1.6] text-[var(--fg)] outline-none transition-colors placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"></textarea>
				</div>
			{/if}
		</div>

		<div class="flex items-center justify-end gap-2 border-t border-[var(--border)] p-[12px_20px]">
			<button type="button" onclick={onclose} disabled={submitting} class="rounded-[7px] px-[12px] py-[6px] text-[12px] text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-3)] hover:text-[var(--fg)] disabled:opacity-50" style="font-family: var(--mono);">Cancel</button>
			<button type="button" onclick={submit} disabled={!canSubmit} class="rounded-[7px] border px-[12px] py-[6px] text-[12px] transition-[background,border-color] disabled:cursor-not-allowed disabled:opacity-50" style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612;">{submitting ? 'Sending...' : 'Submit report'}</button>
		</div>
	</div>
</div>
