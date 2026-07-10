<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { currentUser } from '$lib/stores';
	import { getMe, loginLocal, registerLocal, casLoginUrl, recoveryInfo, resetPassword } from '$lib/api';

	type View = 'auth' | 'secure' | 'forgot' | 'reset';

	let view = $state<View>('auth');
	let mode = $state<'login' | 'register'>('login');
	let username = $state('');
	let password = $state('');
	let error = $state('');
	let notice = $state('');
	let submitting = $state(false);
	let checking = $state(true);

	// security questions
	const PRESET_QUESTIONS = [
		'What was the name of your first pet?',
		'What city were you born in?',
		"What is your mother's maiden name?",
		'What was the name of your first school?',
		'What is your favourite book?',
		'Write my own question…'
	];
	let secQuestion = $state(PRESET_QUESTIONS[0]);
	let customQuestion = $state(false);
	let qOpen = $state(false);
	let qDropdownEl: HTMLElement | null = $state(null);
	let secAnswer = $state('');
	let codeDownloaded = $state(false);
	let pendingCode = $state('');

	$effect(() => {
		if (!qOpen) return;
		function handleOutside(e: MouseEvent) {
			if (qDropdownEl && !qDropdownEl.contains(e.target as Node)) qOpen = false;
		}
		document.addEventListener('click', handleOutside, true);
		return () => document.removeEventListener('click', handleOutside, true);
	});

	function selectQuestion(q: string) {
		if (q === 'Write my own question…') {
			customQuestion = true;
			secQuestion = '';
		} else {
			customQuestion = false;
			secQuestion = q;
		}
		qOpen = false;
	}

	// forgot-password flow
	let forgotUsername = $state('');
	let forgotInfo = $state<{ has_recovery_code: boolean; security_question: string | null } | null>(null);
	let resetTab = $state<'code' | 'question'>('code');
	let resetCode = $state('');
	let resetAnswer = $state('');
	let newPassword = $state('');
	let resetDone = $state(false);
	let newCodeForDownload = $state('');

	const USERNAME_RE = /^[a-z0-9_.-]+$/;

	const NOTICES: Record<string, string> = {
		domain: 'Only @students.iiit.ac.in and @research.iiit.ac.in emails can be used.',
		email_has_local: 'This email is already tied to a username/password account — sign in with that account instead.'
	};

	onMount(async () => {
		const code = new URLSearchParams(window.location.search).get('error');
		if (code && NOTICES[code]) {
			notice = NOTICES[code];
			history.replaceState(null, '', base + '/login');
		}
		let user = $currentUser;
		if (!user) user = await getMe();
		if (user) {
			currentUser.set(user);
			goto(base + (user.verified ? '/' : '/verify'));
			return;
		}
		checking = false;
	});

	function validate(): string | null {
		const u = username.trim().toLowerCase();
		if (u.length < 3 || u.length > 32) return 'Username must be 3–32 characters.';
		if (!USERNAME_RE.test(u)) return 'Username may only use a–z, 0–9, and _ . -';
		if (password.length < 8) return 'Password must be at least 8 characters.';
		return null;
	}

	const canSubmit = $derived(!submitting && username.trim().length > 0 && password.length > 0);

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		if (submitting) return;
		error = '';
		const problem = validate();
		if (problem) { error = problem; return; }

		if (mode === 'login') {
			submitting = true;
			const user = await loginLocal(username.trim().toLowerCase(), password);
			submitting = false;
			if (!user) { error = 'Incorrect username or password.'; return; }
			currentUser.set(user);
			goto(base + (user.verified ? '/' : '/verify'));
		} else {
			// transition to secure-your-account step
			view = 'secure';
		}
	}

	function generateCode(): string {
		const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
		const arr = new Uint8Array(30);
		crypto.getRandomValues(arr);
		const s = Array.from(arr).map(b => chars[b % 62]).join('');
		return `grapevine-${s.slice(0,6)}-${s.slice(6,12)}-${s.slice(12,18)}-${s.slice(18,24)}-${s.slice(24,30)}`;
	}

	function downloadCode(code: string, _name: string) {
		const blob = new Blob([code], { type: 'text/plain' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'grapevine-recovery-code.txt';
		a.click();
		URL.revokeObjectURL(url);
	}

	function handleDownload() {
		const code = generateCode();
		pendingCode = code;
		downloadCode(code, username.trim().toLowerCase());
		codeDownloaded = true;
	}

	const canComplete = $derived(
		codeDownloaded && !submitting
	);

	async function completeRegistration() {
		if (!canComplete) return;
		submitting = true;
		error = '';
		const q = secQuestion.trim();
		const user = await registerLocal(
			username.trim().toLowerCase(),
			password,
			pendingCode || undefined,
			q || undefined,
			secAnswer.trim() || undefined
		);
		submitting = false;
		pendingCode = '';
		if (!user) {
			error = 'Could not create account. The username may already be taken.';
			return;
		}
		currentUser.set(user);
		goto(base + (user.verified ? '/' : '/verify'));
	}

	function toggleMode() {
		mode = mode === 'login' ? 'register' : 'login';
		error = '';
		view = 'auth';
		codeDownloaded = false;
		pendingCode = '';
	}

	// forgot flow
	async function lookupForgot(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		const u = forgotUsername.trim().toLowerCase();
		if (!u) return;
		submitting = true;
		const info = await recoveryInfo(u);
		submitting = false;
		if (!info) { error = 'User not found.'; return; }
		forgotInfo = info;
		resetTab = info.security_question && !info.has_recovery_code ? 'question' : 'code';
		view = 'reset';
	}

	async function doReset(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (newPassword.length < 8) { error = 'Password must be at least 8 characters.'; return; }
		submitting = true;
		const opts = resetTab === 'code' ? { recovery_code: resetCode } : { security_answer: resetAnswer };
		const res = await resetPassword(forgotUsername.trim().toLowerCase(), newPassword, opts);
		submitting = false;
		if (!res) { error = 'Reset failed. Check your credentials and try again.'; return; }
		if (res.new_recovery_code) {
			newCodeForDownload = res.new_recovery_code;
			downloadCode(res.new_recovery_code, forgotUsername.trim().toLowerCase());
		}
		resetDone = true;
	}

	function backToAuth() {
		view = 'auth';
		error = '';
		forgotUsername = '';
		forgotInfo = null;
		resetCode = '';
		resetAnswer = '';
		newPassword = '';
		resetDone = false;
		newCodeForDownload = '';
	}
</script>

<svelte:head>
	<title>Sign In · grapevine</title>
</svelte:head>

{#if !checking}
	<div
		class="flex min-h-screen flex-col items-center justify-center px-6 py-16"
		style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;"
	>
		<div class="w-[min(420px,100%)]">
			<div class="mb-9 text-center">
				<h1
					class="m-0 font-normal leading-[0.95] tracking-[-0.02em] text-[var(--fg)]"
					style="font-family: var(--serif); font-size: clamp(56px, 12vw, 80px);"
				>
					grapevine
				</h1>
				<p class="mt-3 text-[15px] text-[var(--fg-3)]" style="text-wrap: pretty;">
					course &amp; advisor reviews for IIIT Hyderabad
				</p>
			</div>

			<div
				class="rounded-[10px] border border-[var(--border)] p-6 sm:p-7"
				style="background: var(--bg-2); background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);"
			>
				{#if view === 'auth'}
					{#if notice}
						<div
							class="mb-5 rounded-[7px] border px-[12px] py-[10px] text-[13px] leading-[1.5]"
							style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);"
						>
							{notice}
						</div>
					{/if}

					<div class="mb-5 flex items-baseline justify-between">
						<h2 class="text-[20px] text-[var(--fg)]" style="font-family: var(--serif);">
							{mode === 'login' ? 'Sign in' : 'Create account'}
						</h2>
						<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">
							anonymous
						</span>
					</div>

					<form onsubmit={submit} novalidate>
						<label
							class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]"
							style="font-family: var(--mono);"
							for="username"
						>
							Username
						</label>
						<input
							id="username"
							type="text"
							autocomplete="username"
							autocapitalize="none"
							spellcheck="false"
							bind:value={username}
							placeholder="your_handle"
							class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
						/>

						<label
							class="mt-4 block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]"
							style="font-family: var(--mono);"
							for="password"
						>
							Password
						</label>
						<input
							id="password"
							type="password"
							autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
							bind:value={password}
							placeholder="••••••••"
							class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
						/>

						{#if mode === 'register'}
							<p class="mt-2 text-[12px] leading-[1.5] text-[var(--fg-4)]">
								Username: 3–32 characters (a–z, 0–9, _ . -). Password: at least 8 characters.
							</p>
						{/if}

						{#if mode === 'login'}
							<button
								type="button"
								onclick={() => { view = 'forgot'; error = ''; }}
								class="mt-2 text-[12px] text-[var(--fg-4)] hover:text-[var(--accent-2)] transition-colors duration-[120ms]"
							>
								Forgot password?
							</button>
						{/if}

						{#if error}
							<div
								class="mt-4 rounded-[7px] border px-[12px] py-[9px] text-[13px]"
								style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);"
							>
								{error}
							</div>
						{/if}

						<button
							type="submit"
							disabled={!canSubmit}
							class="mt-5 inline-flex w-full items-center justify-center rounded-[8px] border px-[14px] py-[11px] text-[14px] font-semibold transition-[background,border-color,opacity] duration-[120ms] disabled:cursor-not-allowed disabled:opacity-50"
							style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
						>
							{#if submitting}
								{mode === 'login' ? 'Signing in…' : 'Next…'}
							{:else}
								{mode === 'login' ? 'Sign in' : 'Next'}
							{/if}
						</button>
					</form>

					<div class="my-5 flex items-center gap-3">
						<div class="h-px flex-1 bg-[var(--border)]"></div>
						<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">or</span>
						<div class="h-px flex-1 bg-[var(--border)]"></div>
					</div>

					<a
						href={casLoginUrl()}
						class="inline-flex w-full items-center justify-center rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-inset)] px-[14px] py-[11px] text-[14px] text-[var(--fg-2)] transition-[color,background,border-color] duration-[120ms] hover:border-[var(--border-strong)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
					>
						Continue with CAS
					</a>

				{:else if view === 'secure'}
					<div class="mb-5 flex items-baseline justify-between">
						<h2 class="text-[20px] text-[var(--fg)]" style="font-family: var(--serif);">Secure your account</h2>
						<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">optional</span>
					</div>

					<p class="mb-5 text-[13px] leading-[1.6] text-[var(--fg-3)]">
						Set up a way to recover your account if you forget your password. You can skip this, but you won't be able to recover your account later.
					</p>

					<p class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">
						Security Question
					</p>
					<div class="relative mt-2" bind:this={qDropdownEl}>
						<button
							type="button"
							onclick={() => (qOpen = !qOpen)}
							aria-label="Select security question"
							class="flex w-full items-center justify-between gap-2 rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-left text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)]"
							style={qOpen ? 'border-color: var(--accent);' : ''}
						>
							<span class="truncate {customQuestion ? 'text-[var(--fg-3)]' : ''}">
								{customQuestion ? 'Write my own question…' : secQuestion}
							</span>
							<svg
								class="size-4 shrink-0 text-[var(--fg-4)] transition-transform duration-[120ms] {qOpen ? 'rotate-180' : ''}"
								viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"
							>
								<path d="M4 6l4 4 4-4" stroke-linecap="round" stroke-linejoin="round"/>
							</svg>
						</button>

						{#if qOpen}
							<div class="absolute z-20 mt-1 w-full overflow-hidden rounded-[8px] border border-[var(--border-2)] shadow-lg" style="background: var(--bg-2);">
								{#each PRESET_QUESTIONS as q}
									<button
										type="button"
										onclick={() => selectQuestion(q)}
										class="w-full px-[14px] py-[9px] text-left text-[14px] transition-colors duration-[80ms] hover:bg-[var(--bg-3)] {(customQuestion && q === 'Write my own question…') || (!customQuestion && secQuestion === q) ? 'text-[var(--fg)] font-medium' : 'text-[var(--fg-2)]'}"
									>
										{q}
									</button>
								{/each}
							</div>
						{/if}
					</div>

					{#if customQuestion}
						<input
							type="text"
							bind:value={secQuestion}
							placeholder="Type your question…"
							class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
						/>
					{/if}

					{#if secQuestion.trim()}
						<label
							class="mt-4 block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]"
							style="font-family: var(--mono);"
							for="sec-answer"
						>
							Your Answer
						</label>
						<input
							id="sec-answer"
							type="text"
							bind:value={secAnswer}
							placeholder="Your Answer (case insensitive)"
							class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
						/>
					{/if}

					<div class="mt-5 rounded-[8px] border border-[var(--border-2)] p-4" style="background: var(--bg-inset);">
						<p class="mb-3 text-[13px] leading-[1.5] text-[var(--fg-2)]">
							<span class="font-semibold text-[var(--fg)]">Recovery code</span> <br> Keep this file safe as a fallback.
						</p>
						<button
							type="button"
							onclick={handleDownload}
							class="inline-flex w-full items-center justify-center rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg-2)] transition-[color,background,border-color] duration-[120ms] hover:border-[var(--border-strong)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
						>
							{codeDownloaded ? 'Downloaded' : 'Download Recovery Code'}
						</button>
					</div>

					{#if error}
						<div
							class="mt-4 rounded-[7px] border px-[12px] py-[9px] text-[13px]"
							style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);"
						>
							{error}
						</div>
					{/if}

					<button
						type="button"
						onclick={completeRegistration}
						disabled={!canComplete}
						class="mt-5 inline-flex w-full items-center justify-center rounded-[8px] border px-[14px] py-[11px] text-[14px] font-semibold transition-[background,border-color,opacity] duration-[120ms] disabled:cursor-not-allowed disabled:opacity-50"
						style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
					>
						{submitting ? 'Creating account…' : 'Complete Setup'}
					</button>

					<button
						type="button"
						onclick={async () => {
							submitting = true;
							error = '';
							const user = await registerLocal(username.trim().toLowerCase(), password);
							submitting = false;
							if (!user) { error = 'Could not create account. The username may already be taken.'; return; }
							currentUser.set(user);
							goto(base + (user.verified ? '/' : '/verify'));
						}}
						disabled={submitting}
						class="mt-3 w-full text-center text-[13px] text-[var(--fg-4)] hover:text-[var(--fg-3)] transition-colors duration-[120ms] disabled:opacity-50"
					>
						Skip Security Question
					</button>

				{:else if view === 'forgot'}
					<div class="mb-5">
						<button
							type="button"
							onclick={backToAuth}
							class="mb-4 text-[12px] text-[var(--fg-4)] hover:text-[var(--accent-2)] transition-colors duration-[120ms]"
						>
							Back to sign in
						</button>
						<h2 class="text-[20px] text-[var(--fg)]" style="font-family: var(--serif);">Forgot password</h2>
					</div>

					<form onsubmit={lookupForgot} novalidate>
						<label
							class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]"
							style="font-family: var(--mono);"
							for="forgot-username"
						>
							Username
						</label>
						<input
							id="forgot-username"
							type="text"
							autocomplete="username"
							autocapitalize="none"
							spellcheck="false"
							bind:value={forgotUsername}
							placeholder="your_handle"
							class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
						/>

						{#if error}
							<div
								class="mt-4 rounded-[7px] border px-[12px] py-[9px] text-[13px]"
								style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);"
							>
								{error}
							</div>
						{/if}

						<button
							type="submit"
							disabled={submitting || !forgotUsername.trim()}
							class="mt-5 inline-flex w-full items-center justify-center rounded-[8px] border px-[14px] py-[11px] text-[14px] font-semibold transition-[background,border-color,opacity] duration-[120ms] disabled:cursor-not-allowed disabled:opacity-50"
							style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
						>
							{submitting ? 'Looking up…' : 'Next'}
						</button>
					</form>

				{:else if view === 'reset'}
					<div class="mb-5">
						<button
							type="button"
							onclick={backToAuth}
							class="mb-4 text-[12px] text-[var(--fg-4)] hover:text-[var(--accent-2)] transition-colors duration-[120ms]"
						>
							Back to sign in
						</button>
						<h2 class="text-[20px] text-[var(--fg)]" style="font-family: var(--serif);">Reset password</h2>
					</div>

					{#if resetDone}
						<div
							class="mb-4 rounded-[7px] border px-[12px] py-[10px] text-[13px] leading-[1.5]"
							style="border-color: rgba(107,143,111,0.4); background: rgba(107,143,111,0.08); color: var(--accent);"
						>
							Password reset. {newCodeForDownload ? 'A new recovery code was downloaded to your device.' : ''} You can now sign in.
						</div>
						<button
							type="button"
							onclick={backToAuth}
							class="inline-flex w-full items-center justify-center rounded-[8px] border px-[14px] py-[11px] text-[14px] font-semibold transition-[background,border-color,opacity] duration-[120ms]"
							style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
						>
							Sign in
						</button>
					{:else}
						{#if forgotInfo?.security_question}
							<div class="mb-4 flex gap-2">
								<button
									type="button"
									onclick={() => (resetTab = 'code')}
									class="flex-1 rounded-[7px] border py-[8px] text-[13px] transition-[background,border-color] duration-[120ms]"
									style={resetTab === 'code'
										? 'border-color: var(--accent); background: rgba(107,143,111,0.08); color: var(--fg);'
										: 'border-color: var(--border-2); background: transparent; color: var(--fg-3);'}
								>
									Recovery code
								</button>
								<button
									type="button"
									onclick={() => (resetTab = 'question')}
									class="flex-1 rounded-[7px] border py-[8px] text-[13px] transition-[background,border-color] duration-[120ms]"
									style={resetTab === 'question'
										? 'border-color: var(--accent); background: rgba(107,143,111,0.08); color: var(--fg);'
										: 'border-color: var(--border-2); background: transparent; color: var(--fg-3);'}
								>
									Security question
								</button>
							</div>
						{/if}

						<form onsubmit={doReset} novalidate>
							{#if resetTab === 'code'}
								<p class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]" style="font-family: var(--mono);">Recovery Code</p>
								<label
									class="mt-2 flex w-full cursor-pointer items-center justify-between gap-3 rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)]"
									style={resetCode ? 'border-color: var(--accent);' : ''}
								>
									<span class="truncate {resetCode ? 'text-[var(--fg)]' : 'text-[var(--fg-4)]'}" style="font-family: var(--mono); font-size: 13px;">
										{resetCode || 'Upload recovery code file…'}
									</span>
									<svg class="size-4 shrink-0 text-[var(--fg-4)]" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
										<path d="M8 10V3m0 0L5.5 5.5M8 3l2.5 2.5" stroke-linecap="round" stroke-linejoin="round"/>
										<path d="M3 11v1.5A1.5 1.5 0 004.5 14h7A1.5 1.5 0 0013 12.5V11" stroke-linecap="round"/>
									</svg>
									<input
										type="file"
										accept=".txt,text/plain"
										class="sr-only"
										onchange={async (e) => {
											const file = (e.currentTarget as HTMLInputElement).files?.[0];
											if (!file) return;
											resetCode = (await file.text()).trim();
										}}
									/>
								</label>
							{:else if resetTab === 'question' && forgotInfo?.security_question}
								<p class="mb-3 text-[14px] text-[var(--fg-2)]">{forgotInfo.security_question}</p>
								<label
									class="block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]"
									style="font-family: var(--mono);"
									for="reset-answer"
								>
									Your Answer
								</label>
								<input
									id="reset-answer"
									type="text"
									bind:value={resetAnswer}
									placeholder="Answer"
									class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
								/>
							{/if}

							<label
								class="mt-4 block text-[11px] uppercase tracking-[0.08em] text-[var(--fg-3)]"
								style="font-family: var(--mono);"
								for="new-password"
							>
								New Password
							</label>
							<input
								id="new-password"
								type="password"
								autocomplete="new-password"
								bind:value={newPassword}
								placeholder="••••••••"
								class="mt-2 w-full rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-[10px] text-[14px] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]"
							/>

							{#if error}
								<div
									class="mt-4 rounded-[7px] border px-[12px] py-[9px] text-[13px]"
									style="border-color: rgba(201,122,122,0.32); background: var(--danger-bg); color: var(--danger);"
								>
									{error}
								</div>
							{/if}

							<button
								type="submit"
								disabled={submitting}
								class="mt-5 inline-flex w-full items-center justify-center rounded-[8px] border px-[14px] py-[11px] text-[14px] font-semibold transition-[background,border-color,opacity] duration-[120ms] disabled:cursor-not-allowed disabled:opacity-50"
								style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
							>
								{submitting ? 'Resetting…' : 'Reset Password'}
							</button>
						</form>
					{/if}
				{/if}
			</div>

			{#if view === 'auth'}
				<div class="mt-5 text-center text-[13px] text-[var(--fg-3)]">
					{mode === 'login' ? "Don't have an account?" : 'Already have an account?'}
					<button
						type="button"
						onclick={toggleMode}
						class="ml-1 text-[var(--accent-2)] transition-colors duration-[120ms] hover:text-[var(--accent)]"
					>
						{mode === 'login' ? 'Create one' : 'Sign in'}
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
