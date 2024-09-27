import type { PageLoad } from './$types';
import {
	PUBLIC_FILES_PATH,
	PUBLIC_SAVED_CONFIG,
	PUBLIC_NODE_ENV,
	PUBLIC_SAVED_JOBDESC
} from '$env/static/public';
import { writable, type Writable } from 'svelte/store';

import type { Job, Stats } from '$lib';
import { allJobs, fetchedTotal, appliedTotal } from '$lib/jobHistory';
import { currentCV, currentLetter, nextJobApplication, nextJobDetails } from '$lib/jobApplication';

const dev = PUBLIC_NODE_ENV === 'development';
const jobhunterlib: Promise<any> = typeof window !== 'undefined'
	? import('$lib/jobIO').then((module) => module.jobhunter)
	: new Promise(() => ({}));

const getLoadApplicantConfig: Promise<() => Promise<any>> = typeof window !== 'undefined'
	? import('$lib/jobIO').then((module) => module.loadApplicantConfig)
	: new Promise(() => () => Promise.resolve({}));

let initialised: Writable<boolean> = writable(false);

export const load = (async () => {
	if (typeof window !== 'undefined') {
		initialiseApp();
	}
	return {
		initialised
	};
}) satisfies PageLoad;

async function initialiseApp() {
	const jobhunter = await jobhunterlib;
	jobhunter.startServer();

	async function readLetterParams(): Promise<string> {
		if (dev) {
			const config = await jobhunter.resolvePath(PUBLIC_FILES_PATH, PUBLIC_SAVED_CONFIG);
			const content = await jobhunter.read(config);
			return JSON.parse(content).coverLetter || '';
		} else {
			const content = (await jobhunter.tauriCommand('read_config')) as string;
			return JSON.parse(content).coverLetter || '';
		}
	}

	async function getLastSavedJob(): Promise<string> {
		if (dev) {
			const jobDescPath = await jobhunter.resolvePath(PUBLIC_FILES_PATH, PUBLIC_SAVED_JOBDESC);
			const content = await jobhunter.read(jobDescPath);
			return content || '';
		} else {
			return ((await jobhunter.tauriCommand('read_job_description')) as string) || '';
		}
	}

	async function readConfig(): Promise<string> {
        try {
            if (dev) {
                const configPath = await jobhunter.resolvePath(PUBLIC_FILES_PATH, PUBLIC_SAVED_CONFIG);
                return await jobhunter.read(configPath);
            } else {
                return (await jobhunter.tauriCommand('read_config')) as string;
            }
        } catch (error) {
            console.error('Error reading config:', error);
            await jobhunter.showMessage(`Failed to read config: ${error}`, { title: 'Error', type: 'error' });
            return '{}'; // Return empty object as string
        }
    }


	async function loadAppData() {
        try {
            const loadApplicantConfig = await getLoadApplicantConfig;
            const applicantDetails = await loadApplicantConfig();
            const lastSavedJob = await getLastSavedJob();
            const coverLetter = await readLetterParams();

            // Load job description
            let jobDescription;
            try {
                jobDescription = JSON.parse(lastSavedJob);
            } catch (error) {
                console.error('Error parsing job description:', error);
                jobDescription = {};
            }
            nextJobDetails.set({...nextJobDetails, ...jobDescription});

            // Load applicant details
            let parsedApplicantDetails;
            try {
                if (dev) {
                    parsedApplicantDetails = applicantDetails;
                } else {
                    const applicantDetailsStr = await jobhunter.tauriCommand('read_applicant_details') as string;
                    parsedApplicantDetails = JSON.parse(applicantDetailsStr);
                }
            } catch (error) {
                console.error('Error loading applicant details:', error);
                parsedApplicantDetails = {};
            }
            nextJobApplication.set({...nextJobApplication, ...parsedApplicantDetails});

            currentLetter.set(coverLetter);
            const config = await readConfig();
            currentCV.set(JSON.parse(config).cvFilename);

            const [jobs, stats] = await Promise.all([
                jobhunter.tauriCommand('get_unread_jobs') as Promise<Job[]>,
                jobhunter.tauriCommand('get_stats') as Promise<Stats>
            ]);
            allJobs.set(jobs);

            if (stats) {
                fetchedTotal.set(stats.uniquejobs);
                appliedTotal.set(stats.appliedjobs);
            } else {
                await jobhunter.showMessage('No stats data received', { title: 'Warning', type: 'error' });
            }
        } catch (error) {
            console.error('Error in loadAppData:', error);
            fetchedTotal.set(0);
            appliedTotal.set(0);
            await jobhunter.showMessage('Failed to initialize app.', { title: 'Error', type: 'error' });
        }
    }


	try {
		await loadAppData();
		initialised.set(true);
	} catch (error) {
		await jobhunter.showMessage('Failed to initialize app.', { title: 'Error', type: 'error' });
		initialised.set(true);
	}
}

export const ssr = false;
