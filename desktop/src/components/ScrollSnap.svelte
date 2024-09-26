<script lang="ts">
	import { createEventDispatcher } from 'svelte'
	import type { Job } from '../lib/schemas'
	import { fade } from 'svelte/transition'
	import { elasticOut } from 'svelte/easing'
	import { dialog } from '@tauri-apps/api'

	export let job: Job

	let readbox = job.read
	let appliedtobox = job.appliedto
	let observer: IntersectionObserver
	let buttonVisible = false

	const dispatch = createEventDispatcher()

	function formatDate(dateString: string) {
		const date = new Date(dateString)
		return date.toLocaleDateString()
	}

	function getCVLink(job: Job): string {
		if (job.source === 'indeed') {
			return `https://it.indeed.com/jobs?q=insegnante+madrelingua+inglese&vjk=${job.jobkey}`
		} else if (job.source === 'jooble') {
			return `https://it.jooble.org/desc/${job.jobkey}?ckey=insegnante+madrelingua+inglese`
		} else {
			return '#'
		}
	}

	async function confirm(jobId: number, field: 'read' | 'appliedto', value: boolean) {
		if (!value) {
			job = { ...job, [field]: value }
			dispatch('refresh', { jobId: job.id, field: field as keyof Job, value })
			return
		}

		const confirmed = await dialog.confirm('Do you want to mark job listing as read?', 'Confirmation')
		if (confirmed) {
			job = { ...job, [field]: value }
			dispatch('refresh', { jobId: job.id, field: field as keyof Job, value })
		} else {
			if (field === 'read') {
				readbox = !value
			} else {
				appliedtobox = !value
			}
		}
	}

	function handleIntersection(entries: IntersectionObserverEntry[]): void {
		const [entry] = entries
		buttonVisible = entry.isIntersecting
	}

	function setupObserver(node: Element): { destroy: () => void } {
		observer = new IntersectionObserver(handleIntersection, {
			root: null,
			rootMargin: '0px',
			threshold: 0.1
		})

		observer.observe(node)

		return {
			destroy() {
				observer.disconnect()
			}
		}
	}
</script>

<li
	class="flex min-h-full w-full snap-start flex-col rounded-2xl bg-neutral-100 dark:bg-neutral-800 dark:text-white"
>
	<div class="flex-grow p-10">
		<h2 class="text-xl font-bold">{job.company}</h2>
		<p>Job Title: {job.title}</p>
		<p>Location: {job.location}</p>
		<p>Salary: {job.salary}</p>
		<p>Fetched Date: {formatDate(job.fetched_date)}</p>
		<div class="flex flex-col items-end justify-end" use:setupObserver>
			{#if buttonVisible}
				<a
					href={getCVLink(job)}
					target="_blank"
					rel="noopener noreferrer"
					class="inline-block h-14 w-32 transform rounded-2xl bg-slate-600 text-center
                leading-[3.5rem] text-white shadow-xl transition-all
                duration-150 hover:-translate-y-0.5 hover:bg-neutral-800 hover:shadow-2xl
                focus:outline-none"
					transition:fade={{ duration: 12000, delay: 3000, easing: elasticOut }}
				>
					Find Out More
				</a>
			{/if}
		</div>
	</div>
	<div class="ml-4 flex px-5 pb-10 text-lg font-bold">
		<label>
			<input type="checkbox" bind:checked={readbox} on:change={() => confirm(job.id, 'read', readbox)} />
			Read
		</label>
		<label class="pl-4">
			<input
				type="checkbox"
				bind:checked={appliedtobox}
				on:change={() => confirm(job.id, 'appliedto', appliedtobox)}
			/>
			Applied To
		</label>
	</div>
</li>
