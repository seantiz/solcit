<script lang="ts">
	import { createEventDispatcher } from 'svelte'

	export let visible = false

	const dispatch = createEventDispatcher()

	function menuClick(action: string) {
		dispatch('itemClick', { action })
	}
</script>

{#if visible}
	<div
		class="absolute right-0 z-50 mt-2 w-36 rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5 dark:bg-slate-700"
	>
		<div class="py-1" role="menu" aria-orientation="vertical" aria-labelledby="options-menu">
			<button
				on:click={() => menuClick('home')}
				class="block w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-white dark:hover:bg-slate-400"
				role="menuitem"
			>
				Home
			</button>
			<button
				on:click={() => menuClick('stats')}
				class="block w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-white dark:hover:bg-slate-400"
				role="menuitem"
			>
				Stats
			</button>
			<button
				on:click={() => menuClick('cv')}
				class="block w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-white dark:hover:bg-slate-400"
				role="menuitem"
			>
				My CV
			</button>
			<button
				on:click={() => menuClick('letter')}
				class="block w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-white dark:hover:bg-slate-400"
				role="menuitem"
			>
				My Cover Letter
			</button>
            <button
				on:click={() => menuClick('credentials')}
				class="block w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-white dark:hover:bg-slate-400"
				role="menuitem"
			>
				My API Key
			</button>
		</div>
	</div>
{/if}
