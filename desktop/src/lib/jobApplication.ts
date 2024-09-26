import { writable } from 'svelte/store'
import type { ApplicantDetails, SavedJobDescription } from '.';

export const currentCV = writable('')
export const currentLetter = writable('')
export const generatedLetter = writable('')

export const nextJobApplication = writable<ApplicantDetails>({
	name: '',
	experience: '',
	interests: '',
	projects: '',
	education: '',
	certificates: ''
});

export const nextJobDetails = writable<SavedJobDescription>({
	jobTitle: '',
		company: '',
		jobDescription: '',
		keyRequirements: ''
})
