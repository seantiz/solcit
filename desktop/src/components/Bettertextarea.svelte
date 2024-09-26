<script lang="ts">
	import { onMount } from 'svelte'

	export let value = ''
	export let placeholder = ''
	export let className = ''

	let textarea: HTMLTextAreaElement

	function autoResize() {
		textarea.style.height = 'auto'
		textarea.style.height = textarea.scrollHeight + 'px'
	}

	onMount(() => {
		autoResize()
	})
</script>

<textarea
	bind:this={textarea}
	bind:value
	{placeholder}
	on:input={autoResize}
	class="min-h-12 min-w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-yellow-600 dark:bg-transparent dark:text-neutral-100 {className}"
></textarea>
