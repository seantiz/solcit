<script lang="ts">
	import { createEventDispatcher } from 'svelte'
	import type { Job } from '$lib/schemas'

	export let jobs: Job[]

	const dispatch = createEventDispatcher()

	function handleClick() {
		dispatch('loadNewJobs', 'jooble')
	}

	function keyDown() {
		dispatch('loadNewJobs', 'jooble')
	}

	$: listings = jobs.filter((job) => !job.read && job.source === 'jooble')
</script>

<div
	class="cursor-pointer rounded-lg bg-neutral-300 p-4 shadow dark:bg-slate-800"
	on:click={handleClick}
	on:keydown={keyDown}
	role="button"
	tabindex="0"
>
	<h2 class="mb-4 text-xl font-semibold">Jooble</h2>
	{#if listings.length === 0}
		<p>No new Indeed listings</p>
	{:else}
		<p class="text-end text-2xl text-neutral-500">{listings.length}</p>
	{/if}
</div>
