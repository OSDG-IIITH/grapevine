<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { currentUser } from '$lib/stores';
	import { getMe, loginLocal, registerLocal, casLoginUrl } from '$lib/api';

	let mode = $state<'login' | 'register'>('login');
	let username = $state('');
	let password = $state('');
	let error = $state('');
	let notice = $state('');
	let submitting = $state(false);
	let checking = $state(true);

	const USERNAME_RE = /^[a-z0-9_.-]+$/;

	// Friendly copy for the ?error= codes the CAS login callback redirects back with.
	const NOTICES: Record<string, string> = {
		domain: 'Only @students.iiit.ac.in and @research.iiit.ac.in emails can be used.',
		email_has_local:
			'This email is already tied to a username/password account — sign in with that account instead.'
	};

	onMount(async () => {
		// Surface a CAS-callback rejection, then strip the param so a refresh is clean.
		const code = new URLSearchParams(window.location.search).get('error');
		if (code && NOTICES[code]) {
			notice = NOTICES[code];
			history.replaceState(null, '', base + '/login');
		}

		// Bounce already-authenticated users out of the front door.
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

	const canSubmit = $derived(
		!submitting && username.trim().length > 0 && password.length > 0
	);

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		if (submitting) return;
		error = '';

		const problem = validate();
		if (problem) {
			error = problem;
			return;
		}

		submitting = true;
		const u = username.trim().toLowerCase();
		const user =
			mode === 'login' ? await loginLocal(u, password) : await registerLocal(u, password);
		submitting = false;

		if (!user) {
			error =
				mode === 'login'
					? 'Incorrect username or password.'
					: 'Could not create account. The username may already be taken.';
			return;
		}

		currentUser.set(user);
		goto(base + (user.verified ? '/' : '/verify'));
	}

	function toggleMode() {
		mode = mode === 'login' ? 'register' : 'login';
		error = '';
	}
</script>

<svelte:head>
	<title>grapevine — sign in</title>
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
							{mode === 'login' ? 'Signing in…' : 'Creating account…'}
						{:else}
							{mode === 'login' ? 'Sign in' : 'Create account'}
						{/if}
					</button>
				</form>

				<div class="my-5 flex items-center gap-3">
					<div class="h-px flex-1 bg-[var(--border)]"></div>
					<span class="text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">
						or
					</span>
					<div class="h-px flex-1 bg-[var(--border)]"></div>
				</div>

				<a
					href={casLoginUrl()}
					class="inline-flex w-full items-center justify-center rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-inset)] px-[14px] py-[11px] text-[14px] text-[var(--fg-2)] transition-[color,background,border-color] duration-[120ms] hover:border-[var(--border-strong)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]"
				>
					Continue with CAS
				</a>
			</div>

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
		</div>
	</div>
{/if}
