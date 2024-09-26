<script lang="ts">
	import { createEventDispatcher } from 'svelte'
	import { BTA } from './'

	export let title: string
	export let content: string = ''

	let isEditing: boolean = false
	let editableContent: string = content

	const dispatch = createEventDispatcher()

	export function confirmSave() {
		isEditing = false
		content = editableContent
		dispatch('update', content)
	}

	function startEditing() {
		isEditing = true
		editableContent = content
	}

	function requestSave() {
		dispatch('save', editableContent)
		// Note: We're not changing isEditing or content here
	}

	function cancelEditing() {
		isEditing = false
		editableContent = content
	}
</script>

<div class="p-4 text-sm">
	<h2 class="mb-2 text-xl font-bold">{title}</h2>
	<div class="mt-4">
		{#if isEditing}
			<button class="rounded bg-green-600 px-4 py-2 text-white" on:click={requestSave}> Save </button>
			<button class="ml-2 rounded bg-red-600 px-4 py-2 text-white" on:click={cancelEditing}> Cancel </button>
		{:else}
			<button class="rounded bg-neutral-800 px-4 py-2 text-white" on:click={startEditing}> Edit </button>
		{/if}
	</div>
	<div class="mt-4">
		{#if isEditing}
			<BTA bind:value={editableContent} />
		{:else}
			<div class="rounded border px-3 py-2" style="white-space: pre-wrap;">
				{content}
			</div>
		{/if}
	</div>
</div>
