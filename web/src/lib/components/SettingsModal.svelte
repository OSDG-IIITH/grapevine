<script lang="ts">
	import { untrack } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { IconLock, IconHelpCircle, IconKey, IconEye, IconEyeOff, IconLink } from '@tabler/icons-svelte';
	import { currentUser } from '$lib/stores';
	import { changePassword, newRecoveryCode, updateSecurityQuestion, casLinkUrl, unlinkCas, getMe } from '$lib/api';
	import type { AuthUser } from '$lib/types';

	interface Props { onclose: () => void; user: AuthUser; }
	let { onclose, user }: Props = $props();

	type Tab = 'password' | 'question' | 'code' | 'cas';
	let tab = $state<Tab>('password');

	// ── password tab ─────────────────────────────
	let curpw = $state('');
	let newpw = $state('');
	let confirmpw = $state('');
	let pwsaving = $state(false);
	let pwerror = $state('');
	let showcurpw = $state(false);
	let shownewpw = $state(false);
	let showconfirmpw = $state(false);

	async function savepw() {
		pwerror = '';
		if (newpw.length < 8) { pwerror = 'New password must be at least 8 characters.'; return; }
		if (newpw !== confirmpw) { pwerror = 'Passwords do not match.'; return; }
		pwsaving = true;
		const ok = await changePassword(curpw, newpw);
		pwsaving = false;
		if (!ok) { pwerror = 'Current password is incorrect.'; return; }
		toast.success('Password updated');
		curpw = ''; newpw = ''; confirmpw = '';
	}

	// ── security question tab ────────────────────
	const PRESET_QUESTIONS = [
		'What was the name of your first pet?',
		'What city were you born in?',
		"What is your mother's maiden name?",
		'What was the name of your first school?',
		'What is your favourite book?',
		'Write my own question…'
	];

	let secq = $state(untrack(() => user.security_question ?? PRESET_QUESTIONS[0]));
	let customq = $state(false);
	let qopen = $state(false);
	let qdropel: HTMLElement | null = $state(null);
	let secanswer = $state('');
	let showanswer = $state(false);
	let qsaving = $state(false);
	let qerror = $state('');
	let removing = $state(false);

	$effect(() => {
		if (!qopen) return;
		function out(e: MouseEvent) {
			if (qdropel && !qdropel.contains(e.target as Node)) qopen = false;
		}
		document.addEventListener('click', out, true);
		return () => document.removeEventListener('click', out, true);
	});

	function pickq(q: string) {
		if (q === 'Write my own question…') { customq = true; secq = ''; }
		else { customq = false; secq = q; }
		qopen = false;
	}

	const displayq = $derived(customq ? 'Write my own question…' : secq);

	async function saveq() {
		qerror = '';
		if (!secq.trim()) { qerror = 'Enter a question.'; return; }
		if (!secanswer.trim()) { qerror = 'Enter an answer.'; return; }
		qsaving = true;
		const ok = await updateSecurityQuestion(secq.trim(), secanswer.trim());
		qsaving = false;
		if (!ok) { qerror = 'Failed to update security question.'; return; }
		currentUser.update(u => u ? { ...u, security_question: secq.trim() } : u);
		toast.success('Security question updated');
		secanswer = '';
	}

	async function removeq() {
		removing = true;
		const ok = await updateSecurityQuestion(null, null);
		removing = false;
		if (!ok) { toast.error('Failed to remove security question.'); return; }
		currentUser.update(u => u ? { ...u, security_question: null } : u);
		secq = PRESET_QUESTIONS[0];
		secanswer = '';
		customq = false;
		toast.success('Security question removed');
	}

	// ── CAS tab ──────────────────────────────────
	let unlinkusername = $state('');
	let unlinkpw = $state('');
	let unlinkconfirm = $state('');
	let unlinksaving = $state(false);
	let unlinkerror = $state('');
	let showunlinkpw = $state(false);
	let showunlinkconfirm = $state(false);

	async function dounlink() {
		unlinkerror = '';
		if (unlinkpw.length < 8) { unlinkerror = 'Password must be at least 8 characters.'; return; }
		if (unlinkpw !== unlinkconfirm) { unlinkerror = 'Passwords do not match.'; return; }
		unlinksaving = true;
		const ok = await unlinkCas(unlinkusername, unlinkpw);
		unlinksaving = false;
		if (!ok) { unlinkerror = 'Failed to unlink. Check your username is valid.'; return; }
		const updated = await getMe();
		if (updated) currentUser.set(updated);
		toast.success('CAS unlinked — you are now a local account');
		unlinkusername = ''; unlinkpw = ''; unlinkconfirm = '';
	}

	// ── recovery code tab ────────────────────────
	let codeloading = $state(false);
	let codedone = $state(false);

	async function gencode() {
		codeloading = true;
		const res = await newRecoveryCode();
		codeloading = false;
		if (!res) { toast.error('Failed to generate recovery code.'); return; }
		const blob = new Blob([res.code], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url; a.download = 'grapevine-recovery-code.txt'; a.click();
		URL.revokeObjectURL(url);
		currentUser.update(u => u ? { ...u, has_recovery_code: true } : u);
		codedone = true;
	}

	// ── portal ───────────────────────────────────
	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return { destroy() { node.remove(); } };
	}

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	function keydown(e: KeyboardEvent) { if (e.key === 'Escape') onclose(); }

	const tabs = [
		{ id: 'password' as Tab, label: 'Password',         Icon: IconLock        },
		{ id: 'question' as Tab, label: 'Security question', Icon: IconHelpCircle  },
		{ id: 'code'     as Tab, label: 'Recovery code',     Icon: IconKey         },
		{ id: 'cas'      as Tab, label: 'CAS link',          Icon: IconLink        },
	];
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	use:portal
	role="presentation"
	class="fixed inset-0 z-[210] flex items-center justify-center p-4"
	style="background: rgba(10,14,12,0.7); backdrop-filter: blur(6px); animation: fadeIn 140ms ease-out;"
	onclick={onclose}
	onkeydown={keydown}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="flex w-[min(720px,100%)] flex-col rounded-xl border border-[var(--border-strong)] shadow-2xl"
		style="background: var(--bg-2); animation: fadeUp 180ms cubic-bezier(.2,.6,.2,1);"
		onclick={(e) => e.stopPropagation()}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<!-- header -->
		<div class="flex shrink-0 items-center justify-between border-b border-[var(--border)] px-5 py-[14px]">
			<span class="text-[22px] text-[var(--fg)]" style="font-family: var(--serif);">Settings</span>
			<button
				type="button"
				aria-label="Close"
				onclick={onclose}
				class="inline-flex h-7 w-7 items-center justify-center rounded-md text-[var(--fg-4)] transition-[color,background] duration-[100ms] hover:bg-[var(--bg-4)] hover:text-[var(--fg)]"
			>
				<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
					<path d="M18 6 6 18M6 6l12 12"/>
				</svg>
			</button>
		</div>

		<!-- body: sidebar + content, fixed height for consistent size -->
		<div class="flex" style="height: 440px;">
			<!-- sidebar tabs -->
			<div class="flex w-[185px] shrink-0 flex-col gap-[2px] border-r border-[var(--border)] p-2">
				{#each tabs as t}
					<button
						type="button"
						onclick={() => { tab = t.id; pwerror = ''; qerror = ''; }}
						class="flex items-center gap-[10px] rounded-[7px] px-[10px] py-[8px] text-left text-[13px] transition-[background,color] duration-[100ms]"
						style={tab === t.id
							? 'background: var(--bg-3); color: var(--fg);'
							: 'background: transparent; color: var(--fg-3);'}
					>
						<t.Icon size={15} stroke={1.6} class="shrink-0" />
						<span class="leading-snug">{t.label}</span>
					</button>
				{/each}
			</div>

			<!-- content — scrollable if content overflows fixed height -->
			<div class="flex-1 overflow-y-auto p-5">
				{#if tab === 'password'}
					{#if user.auth_method === 'cas'}
						<p class="text-[13px] leading-[1.6] text-[var(--fg-3)]">
							Your account uses CAS.
						</p>
					{:else}
						<div class="space-y-4">
							<div>
								<label class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="cur-pw">Current password</label>
								<div class="relative mt-1.5">
									<input id="cur-pw" type={showcurpw ? 'text' : 'password'} autocomplete="current-password" bind:value={curpw} placeholder="••••••••" class="w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[9px] pl-[13px] pr-[38px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]" />
									<button type="button" aria-label={showcurpw ? 'Hide' : 'Show'} onclick={() => showcurpw = !showcurpw} class="absolute right-[10px] top-1/2 -translate-y-1/2 text-[var(--fg-4)] transition-colors duration-[100ms] hover:text-[var(--fg-3)]">
										{#if showcurpw}<IconEyeOff size={15} stroke={1.6} />{:else}<IconEye size={15} stroke={1.6} />{/if}
									</button>
								</div>
							</div>
							<div>
								<label class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="new-pw">New password</label>
								<div class="relative mt-1.5">
									<input id="new-pw" type={shownewpw ? 'text' : 'password'} autocomplete="new-password" bind:value={newpw} placeholder="••••••••" class="w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[9px] pl-[13px] pr-[38px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]" />
									<button type="button" aria-label={shownewpw ? 'Hide' : 'Show'} onclick={() => shownewpw = !shownewpw} class="absolute right-[10px] top-1/2 -translate-y-1/2 text-[var(--fg-4)] transition-colors duration-[100ms] hover:text-[var(--fg-3)]">
										{#if shownewpw}<IconEyeOff size={15} stroke={1.6} />{:else}<IconEye size={15} stroke={1.6} />{/if}
									</button>
								</div>
							</div>
							<div>
								<label class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="confirm-pw">Confirm new password</label>
								<div class="relative mt-1.5">
									<input id="confirm-pw" type={showconfirmpw ? 'text' : 'password'} autocomplete="new-password" bind:value={confirmpw} placeholder="••••••••" class="w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[9px] pl-[13px] pr-[38px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]" />
									<button type="button" aria-label={showconfirmpw ? 'Hide' : 'Show'} onclick={() => showconfirmpw = !showconfirmpw} class="absolute right-[10px] top-1/2 -translate-y-1/2 text-[var(--fg-4)] transition-colors duration-[100ms] hover:text-[var(--fg-3)]">
										{#if showconfirmpw}<IconEyeOff size={15} stroke={1.6} />{:else}<IconEye size={15} stroke={1.6} />{/if}
									</button>
								</div>
							</div>

							{#if pwerror}
								<p class="rounded-[7px] border px-[12px] py-[8px] text-[13px]" style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);">{pwerror}</p>
							{/if}

							<button
								type="button"
								onclick={savepw}
								disabled={pwsaving || !curpw || !newpw || !confirmpw}
								class="inline-flex items-center rounded-[8px] border px-[14px] py-[9px] text-[13px] font-semibold transition-[background,border-color,opacity] duration-[120ms] disabled:cursor-not-allowed disabled:opacity-40"
								style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
							>
								{pwsaving ? 'Saving…' : 'Update password'}
							</button>
						</div>
					{/if}

				{:else if tab === 'question'}
					<div class="space-y-4">
						{#if user.security_question}
							<div class="rounded-[8px] border border-[var(--border-2)] px-[13px] py-[10px]" style="background: var(--bg-inset);">
								<p class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Current question</p>
								<p class="mt-1 text-[13px] text-[var(--fg-2)]">{user.security_question}</p>
							</div>
						{/if}

						<div>
							<p class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">{user.security_question ? 'New question' : 'Security question'}</p>
							<div class="relative mt-1.5" bind:this={qdropel}>
								<button
									type="button"
									onclick={() => (qopen = !qopen)}
									aria-label="Select security question"
									class="flex w-full items-center justify-between gap-2 rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[13px] py-[9px] text-left text-[13px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)]"
									style={qopen ? 'border-color: var(--accent);' : ''}
								>
									<span class="truncate {customq ? 'text-[var(--fg-3)]' : ''}">{displayq || '— choose —'}</span>
									<svg class="size-[14px] shrink-0 text-[var(--fg-4)] transition-transform duration-[120ms] {qopen ? 'rotate-180' : ''}" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
										<path d="M4 6l4 4 4-4" stroke-linecap="round" stroke-linejoin="round"/>
									</svg>
								</button>
								{#if qopen}
									<div class="absolute z-20 mt-1 w-full overflow-hidden rounded-[8px] border border-[var(--border-2)] shadow-lg" style="background: var(--bg-2);">
										{#each PRESET_QUESTIONS as q}
											<button
												type="button"
												onclick={() => pickq(q)}
												class="w-full px-[13px] py-[8px] text-left text-[13px] transition-colors duration-[80ms] hover:bg-[var(--bg-3)] {(!customq && secq === q) || (customq && q === 'Write my own question…') ? 'text-[var(--fg)] font-medium' : 'text-[var(--fg-2)]'}"
											>{q}</button>
										{/each}
									</div>
								{/if}
							</div>
						</div>

						{#if customq}
							<input
								type="text"
								bind:value={secq}
								placeholder="Type your question…"
								class="w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[13px] py-[9px] text-[13px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
							/>
						{/if}

						{#if secq.trim()}
							<div>
								<label class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="sec-ans">Answer</label>
								<div class="relative mt-1.5">
									<input
										id="sec-ans"
										type={showanswer ? 'text' : 'password'}
										bind:value={secanswer}
										placeholder="Case and punctuation ignored"
										class="w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[9px] pl-[13px] pr-[38px] text-[13px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
									/>
									<button
										type="button"
										aria-label={showanswer ? 'Hide answer' : 'Show answer'}
										onclick={() => showanswer = !showanswer}
										class="absolute right-[10px] top-1/2 -translate-y-1/2 text-[var(--fg-4)] transition-colors duration-[100ms] hover:text-[var(--fg-3)]"
									>
										{#if showanswer}
											<IconEyeOff size={15} stroke={1.6} />
										{:else}
											<IconEye size={15} stroke={1.6} />
										{/if}
									</button>
								</div>
							</div>
						{/if}

						{#if qerror}
							<p class="rounded-[7px] border px-[12px] py-[8px] text-[13px]" style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);">{qerror}</p>
						{/if}

						<div class="flex items-center gap-2">
							<button
								type="button"
								onclick={saveq}
								disabled={qsaving || !secq.trim() || !secanswer.trim()}
								class="inline-flex items-center rounded-[8px] border px-[14px] py-[9px] text-[13px] font-semibold transition-[background,border-color,opacity] duration-[120ms] disabled:cursor-not-allowed disabled:opacity-40"
								style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
							>
								{qsaving ? 'Saving…' : 'Save question'}
							</button>

							{#if user.security_question}
								<button
									type="button"
									onclick={removeq}
									disabled={removing}
									class="inline-flex items-center rounded-[8px] border border-[var(--border-2)] px-[14px] py-[9px] text-[13px] text-[var(--danger)] transition-[background,border-color,opacity] duration-[120ms] hover:border-[rgba(201,122,122,0.4)] hover:bg-[var(--danger-bg)] disabled:opacity-40"
								>
									{removing ? 'Removing…' : 'Remove'}
								</button>
							{/if}
						</div>
					</div>

				{:else if tab === 'code'}
					<div class="space-y-4">
						<p class="text-[13px] leading-[1.6] text-[var(--fg-3)]">
							Your recovery code lets you reset your password if you're locked out. Keep it somewhere safe.
						</p>

						{#if user.has_recovery_code}
							<div class="rounded-[8px] border border-[var(--border-2)] px-[13px] py-[10px] text-[13px] leading-[1.5] text-[var(--fg-3)]" style="background: var(--bg-inset);">
								A recovery code is currently active. Generating a new one will <span class="text-[var(--fg-2)]">invalidate the old file</span>.
							</div>
						{/if}

						{#if codedone}
							<div class="rounded-[8px] border px-[13px] py-[10px] text-[13px]" style="border-color: rgba(107,143,111,0.35); background: rgba(107,143,111,0.07); color: var(--accent-2);">
								New code downloaded. Save that file.
							</div>
						{/if}

						<button
							type="button"
							onclick={gencode}
							disabled={codeloading}
							class="inline-flex items-center gap-2 rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-3)] px-[14px] py-[9px] text-[13px] text-[var(--fg-2)] transition-[background,border-color,color,opacity] duration-[120ms] hover:border-[var(--border-strong)] hover:bg-[var(--bg-4)] hover:text-[var(--fg)] disabled:opacity-40"
						>
							<IconKey size={14} stroke={1.6} />
							{codeloading ? 'Generating…' : user.has_recovery_code ? 'Generate new code' : 'Generate code'}
						</button>
					</div>

				{:else if tab === 'cas'}
					{#if user.auth_method === 'local'}
						<div class="space-y-4">
							<p class="text-[13px] leading-[1.6] text-[var(--fg-3)]">
								Your account is currently local-only. Connect with CAS to switch to IIIT CAS authentication.
							</p>
							<div class="rounded-[8px] border border-[var(--border-2)] px-[13px] py-[10px] text-[13px] leading-[1.5] text-[var(--fg-3)]" style="background: var(--bg-inset);">
								This will <span class="text-[var(--fg-2)]">delete your username and password</span>. You will log in using CAS only.
							</div>
							<a
								href={casLinkUrl()}
								class="inline-flex items-center gap-2 rounded-[8px] border px-[14px] py-[9px] text-[13px] font-semibold transition-[background,border-color,opacity] duration-[120ms]"
								style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
							>
								<IconLink size={14} stroke={1.8} />
								Connect with CAS
							</a>
						</div>
					{:else}
						<div class="space-y-4">
							<div class="rounded-[8px] border border-[var(--border-2)] px-[13px] py-[10px]" style="background: var(--bg-inset);">
								<p class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Linked account</p>
								<p class="mt-1 text-[13px] text-[var(--fg-2)]">{user.cas_id}</p>
							</div>
							<div class="space-y-3">
								<div>
									<label class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="unlink-user">Choose username</label>
									<input id="unlink-user" type="text" autocomplete="username" bind:value={unlinkusername} placeholder="e.g. alice99" class="mt-1.5 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[13px] py-[9px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]" />
								</div>
								<div>
									<label class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="unlink-pw">Choose password</label>
									<div class="relative mt-1.5">
										<input id="unlink-pw" type={showunlinkpw ? 'text' : 'password'} autocomplete="new-password" bind:value={unlinkpw} placeholder="••••••••" class="w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[9px] pl-[13px] pr-[38px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]" />
										<button type="button" tabindex="-1" aria-label={showunlinkpw ? 'Hide' : 'Show'} onclick={() => showunlinkpw = !showunlinkpw} class="absolute right-[10px] top-1/2 -translate-y-1/2 text-[var(--fg-4)] transition-colors duration-[100ms] hover:text-[var(--fg-3)]">
											{#if showunlinkpw}<IconEyeOff size={15} stroke={1.6} />{:else}<IconEye size={15} stroke={1.6} />{/if}
										</button>
									</div>
								</div>
								<div>
									<label class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);" for="unlink-confirm">Confirm password</label>
									<div class="relative mt-1.5">
										<input id="unlink-confirm" type={showunlinkconfirm ? 'text' : 'password'} autocomplete="new-password" bind:value={unlinkconfirm} placeholder="••••••••" class="w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] py-[9px] pl-[13px] pr-[38px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]" />
										<button type="button" tabindex="-1" aria-label={showunlinkconfirm ? 'Hide' : 'Show'} onclick={() => showunlinkconfirm = !showunlinkconfirm} class="absolute right-[10px] top-1/2 -translate-y-1/2 text-[var(--fg-4)] transition-colors duration-[100ms] hover:text-[var(--fg-3)]">
											{#if showunlinkconfirm}<IconEyeOff size={15} stroke={1.6} />{:else}<IconEye size={15} stroke={1.6} />{/if}
										</button>
									</div>
								</div>
							</div>
							{#if unlinkerror}
								<p class="rounded-[7px] border px-[12px] py-[8px] text-[13px]" style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);">{unlinkerror}</p>
							{/if}
							<button
								type="button"
								onclick={dounlink}
								disabled={unlinksaving || !unlinkusername || !unlinkpw || !unlinkconfirm}
								class="inline-flex items-center rounded-[8px] border border-[rgba(201,122,122,0.4)] px-[14px] py-[9px] text-[13px] text-[var(--danger)] transition-[background,border-color,opacity] duration-[120ms] hover:bg-[var(--danger-bg)] disabled:cursor-not-allowed disabled:opacity-40"
							>
								{unlinksaving ? 'Converting…' : 'Convert to local account'}
							</button>
						</div>
					{/if}
				{/if}
			</div>
		</div>
	</div>
</div>
