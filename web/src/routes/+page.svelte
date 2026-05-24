<script lang="ts">
	import SearchBox from '$lib/components/SearchBox.svelte';

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

		<SearchBox {hints} />
	</div>
</div>
