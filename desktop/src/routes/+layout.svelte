<script lang="ts">
	import '../app.pcss'
	import { page } from '$app/stores'
	import { fade } from 'svelte/transition'
	import { cubicIn, cubicOut } from 'svelte/easing'
	import { derived } from 'svelte/store'
	import { onMount } from 'svelte'
	import { goto } from '$app/navigation'
	import { Menu } from '$components'
	import { jobhunter } from '$lib/jobIO'
	import { allJobs, fetching } from '$lib/jobHistory'

	let dropdownVisible = false

	const unreadJobsCount = derived(allJobs, ($allJobs) => $allJobs.filter((job) => !job.read).length)

	function toggleDropdown(event: MouseEvent) {
		event.stopPropagation()
		dropdownVisible = !dropdownVisible
	}

	function menuAction(event: CustomEvent) {
		const { action } = event.detail
		switch (action) {
			case 'home':
				goto('/')
				break
			case 'cv':
				goto('/cv')
				break
			case 'letter':
				goto('/coverletter')
				break
			case 'stats':
				goto('/stats')
				break
		}
		dropdownVisible = false
	}

	onMount(() => {
		const handleDocumentClick = (event: MouseEvent) => {
			if (
				dropdownVisible &&
				event.target instanceof Element &&
				!event.target.closest('.dropdown-container')
			) {
				dropdownVisible = false
			}
		}

		document.addEventListener('click', handleDocumentClick)

		return () => {
			document.removeEventListener('click', handleDocumentClick)
		}
	})
</script>

<button
	class="titlebar fixed left-0 right-0 top-0 flex h-8 select-none justify-between bg-transparent"
	aria-label="Window controls"
	on:mousedown={jobhunter.dragWindow}
>
</button>

<div
	class="h-full w-full bg-gradient-to-bl from-yellow-50 to-yellow-300 px-4 pb-20 pt-8 dark:bg-slate-900 dark:from-slate-900 dark:to-slate-900 dark:text-neutral-400"
>
	<div class="mb-10 w-full rounded-lg bg-neutral-300 p-4 shadow dark:bg-slate-800">
		<div class="relative flex items-center justify-center">
			<p class="text-center text-lg font-semibold text-neutral-600 dark:text-neutral-300">
				Today has <span class="text-6xl font-bold text-yellow-600">{$unreadJobsCount}</span> unread job{$unreadJobsCount ===
				1
					? ''
					: 's'}
			</p>
			<div class="absolute right-0 flex flex-col items-center">
				<div class="relative">
					<button
						on:click={toggleDropdown}
						class="mb-2 overflow-hidden rounded-full
								 border-2 shadow-lg
								focus:outline-none"
						class:border-yellow-500={!$fetching}
						class:border-red-600={$fetching}
					>
						<img src="/headshot2.jpeg" alt="Sean Headshot" class="h-10 w-10 object-cover" />
						<div
							class="-transform-x-1 absolute right-0 top-0 h-3.5 w-3.5 -translate-y-1 transform rounded-full"
							class:bg-yellow-500={!$fetching}
							class:bg-red-600={$fetching}
						></div>
					</button>
					{#if dropdownVisible}
						<Menu visible={dropdownVisible} on:itemClick={menuAction} />
					{/if}
				</div>
			</div>
		</div>
	</div>
	{#key $page.url.pathname}
		<div in:fade={{ duration: 300, easing: cubicIn }} out:fade={{ duration: 50, easing: cubicOut }}>
			<slot></slot>
		</div>
	{/key}
</div>

<style>
	.titlebar {
		-webkit-user-select: none;
		user-select: none;
	}
</style>
