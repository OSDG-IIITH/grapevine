<script lang="ts">
	import SearchBox from '$lib/components/SearchBox.svelte';
</script>

<div
	class="flex min-h-0 flex-1 flex-col items-center justify-center overflow-hidden px-8 text-center"
	style="transform: translateY(-54px);"
>
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

		<SearchBox />
	</div>
</div>
