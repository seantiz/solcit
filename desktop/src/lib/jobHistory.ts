import type { Job } from '.'
import { writable, derived } from 'svelte/store'

export const fetching = writable(false)
export const allJobs = writable<Job[]>([])
export const jobSite = writable<string | null>(null)
export const appliedTotal = writable(0)
export const fetchedTotal = writable(0)
export const unreadJobs = derived([allJobs, jobSite], ([$allJobs, $jobSite]) =>
	$jobSite
		? $allJobs.filter((job) => job.source === $jobSite && !job.read && !job.appliedto)
		: $allJobs.filter((job) => !job.read && !job.appliedto)
)
