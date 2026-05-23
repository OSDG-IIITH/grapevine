<script lang="ts">
	interface Item {
		label: string;
		href?: string;
	}
	let { items }: { items: Item[] } = $props();
</script>

<div
	class="mb-[18px] flex items-center gap-2 text-[12px] text-[var(--fg-3)]"
	style="font-family: var(--mono);"
>
	{#each items as item, i (i)}
		{#if i > 0}
			<span class="text-[var(--fg-4)]">/</span>
		{/if}
		{#if i === items.length - 1}
			<span class="text-[var(--fg-2)]">{item.label}</span>
		{:else}
			<a href={item.href} class="transition-colors duration-[120ms] hover:text-[var(--fg)]">
				{item.label}
			</a>
		{/if}
	{/each}
</div>
