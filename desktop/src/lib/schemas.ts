export interface Job {
    id: number,
    title: string,
    company: string,
    location: string,
    salary: string,
    jobkey: string,
    fetched_date: string,
    read: boolean,
    appliedto: boolean,
    source: string
}

export interface ApplicantDetails {
    name: string,
    experience: string,
    interests: string,
    projects: string,
    education: string,
    certificates: string
}

export interface JobDescription extends ApplicantDetails {
    jobTitle: string,
    company: string,
    jobDescription: string,
    keyRequirements: string
}

export interface SavedJobDescription {
    jobTitle: '',
	company: '',
	jobDescription: '',
	keyRequirements: ''
}

export interface Stats {
    uniquejobs: number,
    appliedjobs: number
}