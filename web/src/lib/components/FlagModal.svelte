<script lang="ts">
	interface Props {
		onclose: () => void;
		onsubmit: (reason: string) => void;
		submitting?: boolean;
	}

	let { onclose, onsubmit, submitting = false }: Props = $props();

	let reason = $state('');
	const cansubmit = $derived(!submitting && reason.trim().length > 2);

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	function submit() {
		if (!cansubmit) return;
		onsubmit(reason.trim());
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
		onclick={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div class="flex items-center justify-between gap-3 border-b border-[var(--border)] p-[18px_20px]">
			<div>
				<div class="text-[24px] text-[var(--fg)]" style="font-family: var(--serif);">Flag review</div>
			</div>
			<button
				type="button"
				aria-label="Close"
				onclick={onclose}
				class="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md text-[var(--fg-3)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-4)] hover:text-[var(--fg)]"
			>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
					<path d="M18 6 6 18M6 6l12 12" />
				</svg>
			</button>
		</div>

		<div class="p-[18px_20px_20px]">
			<label class="block text-[12px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Reason</label>
			<textarea
				class="mt-2 w-full resize-none rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[12px] text-[14px] leading-[1.6] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
				rows="4"
				placeholder="Why are you flagging this review? "
				bind:value={reason}
				autofocus
			></textarea>
		</div>

		<div class="flex items-center justify-end gap-2 border-t border-[var(--border)] p-[12px_20px]">
			<button
				type="button"
				onclick={onclose}
				class="inline-flex items-center gap-2 rounded-[7px] border border-transparent px-[12px] py-[6px] text-[12px] text-[var(--fg-2)] transition-[color,background] duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
				style="font-family: var(--mono);"
			>Cancel</button>
			<button
				type="button"
				onclick={submit}
				disabled={!cansubmit}
				class="inline-flex items-center gap-2 rounded-[7px] border px-[12px] py-[6px] text-[12px] transition-[background,border-color] duration-[120ms] {cansubmit ? 'font-semibold' : ''}"
				style="{cansubmit ? 'background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);' : 'border-color: var(--border-2); background: var(--bg-2); color: var(--fg); opacity: 0.5; cursor: not-allowed;'}"
			>
				{submitting ? 'Sending...' : 'Submit flag'}
			</button>
		</div>
	</div>
</div>
