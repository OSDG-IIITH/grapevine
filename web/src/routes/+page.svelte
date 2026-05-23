<script lang="ts">
	const hints = [
		{ label: 'CS3.301 Operating Systems', href: '/courses/cs3301' },
		{ label: 'Manish Shrivastava', href: '/faculty/f2' },
		{ label: 'CVIT', href: '/labs/l1' },
		{ label: 'CS7.401 NLP', href: '/courses/cs7401' }
	];
</script>

<div class="flex min-h-[calc(100vh-64px)] flex-col items-center justify-center px-8 text-center">
	<h1
		class="m-0 mb-4 text-[92px] font-normal leading-[0.95] tracking-[-0.02em] text-[var(--fg)]"
		style="font-family: var(--serif);"
	>
		grapevine
	</h1>

	<p
		class="mb-14 max-w-[540px] text-[16px] leading-[1.6] text-[var(--fg-3)]"
		style="text-wrap: pretty;"
	>
		for when you need help choosing courses, labs, and advisors
	</p>

	<div class="relative w-[min(640px,90vw)]">
		<div
			class="pointer-events-none absolute -inset-[60px] -z-10 rounded-[50%]"
			style="background: radial-gradient(closest-side, rgba(143,207,163,0.22), transparent 70%); filter: blur(20px);"
		></div>

		<div
			class="relative z-[1] flex w-full items-center gap-[14px] rounded-xl border border-[var(--border-strong)] bg-[var(--bg-2)] px-[22px] py-[18px] text-[16px] text-[var(--fg)]"
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
			>
				<circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" />
			</svg>
			<input
				class="flex-1 border-0 bg-transparent text-[16px] outline-none placeholder:text-[var(--fg-4)]"
				placeholder="Search a course, faculty, or lab…"
				autofocus
			/>
			<span
				class="shrink-0 whitespace-nowrap rounded border border-[var(--border-2)] bg-[var(--bg-3)] px-[7px] py-[3px] text-[12px] text-[var(--fg-3)]"
				style="font-family: var(--mono);"
			>
				Ctrl K
			</span>
		</div>

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
	</div>
</div>
