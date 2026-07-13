<script lang="ts">
	interface Props {
		target: string;
		title?: string;
		onclose: () => void;
		onsubmit: (reason: string) => void;
		submitting?: boolean;
	}

	let { target, title = 'Report inaccurate information', onclose, onsubmit, submitting = false }: Props = $props();
	let reason = $state('');
	const reasonlength = $derived([...reason.trim()].length);
	const cansubmit = $derived(!submitting && reasonlength >= 3 && reasonlength <= 1000);

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	function submit() {
		if (cansubmit) onsubmit(reason.trim());
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	use:portal
	role="presentation"
	class="fixed inset-0 z-[210] flex items-center justify-center p-6"
	style="background: rgba(10,14,12,0.62); backdrop-filter: blur(4px); animation: fadeIn 160ms ease-out;"
	onclick={onclose}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="flex w-[min(520px,100%)] flex-col overflow-hidden rounded-xl border border-[var(--border-strong)]"
		style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.04), transparent 40%); animation: fadeUp 200ms cubic-bezier(.2,.6,.2,1);"
		onclick={(event) => event.stopPropagation()}
		role="dialog"
		aria-modal="true"
		aria-labelledby="report-title"
		tabindex="-1"
	>
		<div class="flex items-start justify-between gap-3 border-b border-[var(--border)] p-[18px_20px]">
			<div>
				<div id="report-title" class="text-[24px] text-[var(--fg)]" style="font-family: var(--serif);">{title}</div>
				<div class="mt-1 text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">{target}</div>
			</div>
			<button type="button" aria-label="Close" onclick={onclose} class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-[var(--fg-3)] transition-colors hover:bg-[var(--bg-4)] hover:text-[var(--fg)]">
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 6 6 18M6 6l12 12" /></svg>
			</button>
		</div>

		<div class="p-[18px_20px_20px]">
			<label for="report-reason" class="block text-[12px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">What is inaccurate?</label>
			<textarea
				id="report-reason"
				class="mt-2 w-full resize-none rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[12px] text-[14px] leading-[1.6] text-[var(--fg)] outline-none transition-colors placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
				rows="4"
				placeholder="Describe the incorrect information and, if possible, the correct information."
				bind:value={reason}
			></textarea>
		</div>

		<div class="flex items-center justify-end gap-2 border-t border-[var(--border)] p-[12px_20px]">
			<button type="button" onclick={onclose} class="rounded-[7px] px-[12px] py-[6px] text-[12px] text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-3)] hover:text-[var(--fg)]" style="font-family: var(--mono);">Cancel</button>
			<button
				type="button"
				onclick={submit}
				disabled={!cansubmit}
				class="rounded-[7px] border px-[12px] py-[6px] text-[12px] transition-[background,border-color] disabled:cursor-not-allowed disabled:opacity-50"
				style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612;"
			>{submitting ? 'Sending...' : 'Submit report'}</button>
		</div>
	</div>
</div>
