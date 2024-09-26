<script lang="ts">
    export const prerender = true
	import { appliedTotal, fetchedTotal } from '$lib/jobHistory'
</script>

<div class="h-screen w-screen">
	<div class="flex min-h-[70vh] flex-col">
		<main class="flex-grow">
			<h1 class="mb-20 text-center text-2xl font-bold">Your Chance of Landing A Job</h1>
			<div class="text-center">
				<p class="text-lg font-bold">
					Total jobs applied to: <span class="text-4xl font-bold text-blue-400">{$appliedTotal}</span>
				</p>
				<p class="text-lg font-bold">
					Total jobs found since we started: <span class="text-4xl font-bold text-blue-400"
						>{$fetchedTotal}</span
					>
				</p>
			</div>
		</main>

		<footer
			class="mt-auto w-full rounded-lg bg-neutral-100 p-2 shadow dark:bg-cyan-700 dark:text-neutral-100"
		>
			<div class="text-center text-sm">
				<p>( Booking Chance = Visibility x Profile Strength x Niche Demand / Total Active Teachers)</p>
			</div>
		</footer>
	</div>
</div>
