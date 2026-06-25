<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { PUBLIC_API_URL } from '$env/static/public';
	import { currentUser } from '$lib/stores';
	import { getMe, casVerifyUrl } from '$lib/api';

	let checking = $state(true);
	let notice = $state('');

	// Friendly copy for the ?error= codes the CAS verify callback redirects back with.
	const NOTICES: Record<string, string> = {
		domain: 'Only @students.iiit.ac.in and @research.iiit.ac.in emails can be used.',
		email_has_cas: 'This email already has a CAS account — log in with CAS instead.',
		email_used: 'This email has already been used to verify an account.'
	};

	onMount(async () => {
		// Surface a CAS-callback rejection, then strip the param so a refresh is clean.
		const code = new URLSearchParams(window.location.search).get('error');
		if (code && NOTICES[code]) {
			notice = NOTICES[code];
			history.replaceState(null, '', base + '/verify');
		}

		let user = $currentUser;
		if (!user) user = await getMe();
		if (user) currentUser.set(user);

		if (!user) {
			// Not logged in at all — front door.
			goto(base + '/login');
			return;
		}
		if (user.verified) {
			// Already verified — into the app.
			goto(base + '/');
			return;
		}
		checking = false;
	});

	async function logout() {
		try {
			const res = await fetch(`${PUBLIC_API_URL || '/grapevine/api'}/auth/logout`, {
				method: 'POST',
				credentials: 'include'
			});
			if (res.ok) {
				const data = await res.json();
				currentUser.set(null);
				window.location.href = data.redirect_url;
				return;
			}
		} catch (e) {
			console.error('Logout failed:', e);
		}
		currentUser.set(null);
		goto(base + '/login');
	}
</script>

<svelte:head>
	<title>grapevine — verify your account</title>
</svelte:head>

{#if !checking}
	<div
		class="flex min-h-screen flex-col items-center justify-center px-6 py-16"
		style="animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;"
	>
		<div class="w-[min(460px,100%)]">
			<div class="mb-8 text-center">
				<h1
					class="m-0 font-normal leading-[0.95] tracking-[-0.02em] text-[var(--fg)]"
					style="font-family: var(--serif); font-size: clamp(40px, 9vw, 56px);"
				>
					one last step
				</h1>
				<p class="mt-3 text-[15px] text-[var(--fg-3)]" style="text-wrap: pretty;">
					verify you're a genuine IIIT student
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

				<p class="text-[14px] leading-[1.7] text-[var(--fg-2)]">
					To keep grapevine trustworthy, every account verifies once through IIIT CAS. You'll sign in
					with your institute email just this once.
				</p>
				<p class="mt-3 text-[14px] leading-[1.7] text-[var(--fg-3)]">
					We never store your email against your account — only a one-way hash that proves the
					address was used. Your username stays anonymous.
				</p>

				<a
					href={casVerifyUrl()}
					class="mt-6 inline-flex w-full items-center justify-center rounded-[8px] border px-[14px] py-[11px] text-[14px] font-semibold transition-[background,border-color,opacity] duration-[120ms]"
					style="background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border-color: #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 1px 0 rgba(0,0,0,0.25);"
				>
					Verify with CAS
				</a>
			</div>

			<div class="mt-5 text-center text-[13px] text-[var(--fg-3)]">
				Wrong account?
				<button
					type="button"
					onclick={logout}
					class="ml-1 text-[var(--accent-2)] transition-colors duration-[120ms] hover:text-[var(--accent)]"
				>
					Log out
				</button>
			</div>
		</div>
	</div>
{/if}
