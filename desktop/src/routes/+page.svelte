<script lang="ts">
	import { allJobs, jobSite, unreadJobs, fetching, fetchedTotal, appliedTotal } from '$lib/jobHistory'
	import { generatedLetter, nextJobApplication, nextJobDetails } from '$lib/jobApplication'
	import { jobhunter, updateJobRecord} from '$lib/jobIO'
	import type { Job, Stats } from '$lib'

	import { View, Indeed, Jooble, BTA, StreamingAnimation } from '$components'

	import { fade, fly } from 'svelte/transition'
	import { spring } from 'svelte/motion'
	import { writable, get } from 'svelte/store'

	export let data

	const { initialised } = data
	const isGenerating = writable(false)

    let jobKeywords = ''

	let indeedSpring = spring(
		{ scale: 1, rotate: 0 },
		{
			stiffness: 0.1,
			damping: 0.15
		}
	)

	let joobleSpring = spring(
		{ scale: 1, rotate: 0 },
		{
			stiffness: 0.1,
			damping: 0.15
		}
	)

	function handleIndeedClick() {
		indeedSpring.set({ scale: 1.05, rotate: 5 })
		setTimeout(() => {
			indeedSpring.set({ scale: 1, rotate: 0 })
		}, 300)
	}

	function handleJoobleClick() {
		joobleSpring.set({ scale: 1.05, rotate: 5 })
		setTimeout(() => {
			joobleSpring.set({ scale: 1, rotate: 0 })
		}, 300)
	}

	function handleIndeedKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault()
			handleIndeedClick()
		}
	}

	function handleJoobleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault()
			handleJoobleClick()
		}
	}

	// View helper function
	function fromHere(event: CustomEvent<string>) {
		jobSite.set(event.detail)
	}

    async function searchJooble() {
    if (!jobKeywords) {
        await jobhunter.showMessage('Please enter job keywords', 'Error');
        return;
    }
    fetching.set(true);
    try {
        const result = await jobhunter.tauriCommand('run_jooble_search', {
            keywords: jobKeywords,
            location: 'Provincia di Monza Brianza' // TODO: make dynamic user input
        });
        console.log(result); // This will log the success message
        await refreshJobListings();
    } catch (error) {
        console.error('Error searching Jooble:', error);
        await jobhunter.showMessage(`Couldn't fetch Jooble listings: ${error}`, 'Error');
    } finally {
        fetching.set(false);
    }
}


    async function searchIndeed() {
        if (!jobKeywords) {
            await jobhunter.showMessage('Please enter job keywords', 'Error');
            return;
        }
        fetching.set(true);
        try {
            await jobhunter.tauriCommand('run_search_engine', {
                engine: 'indeed',
                keywords: jobKeywords,
                location: 'Italy' // TODO: Make dynamic user input
            });
            await refreshJobListings();
        } catch (error) {
            console.error('Error searching Indeed:', error);
            await jobhunter.showMessage(`Couldn't fetch Indeed listings: ${error}`, 'Error');
        } finally {
            fetching.set(false);
        }
    }

    async function refreshJobListings() {
        try {
            const newJobs = await jobhunter.tauriCommand('get_unread_jobs') as Job[];
            allJobs.update(jobs => {
                const existingJobIds = new Set(jobs.map(job => job.id));
                const uniqueNewJobs = newJobs.filter(job => !existingJobIds.has(job.id));
                return [...jobs, ...uniqueNewJobs];
            });

            const newStats = await jobhunter.tauriCommand('get_stats') as Stats;
            if (newStats) {
                fetchedTotal.set(newStats.uniquejobs);
                appliedTotal.set(newStats.appliedjobs);
            }
            const currentJobSite = get(jobSite);
            jobSite.set(null);
            jobSite.set(currentJobSite);

            console.log('New job records loaded from database');
        } catch (error) {
            console.error('Error refreshing job listings:', error);
            await jobhunter.showMessage(`Couldn't refresh job listings: ${error}`, 'Error');
        }
    }

async function makeJobApplication() {
    try {
        const confirmed = await jobhunter.askConfirmation(
            'Save job application details?',
            'Please Confirm'
        )

        if (!confirmed) {
            return
        }

        // Save job description
        const jobDescriptionData = {
            jobTitle: $nextJobDetails.jobTitle,
            company: $nextJobDetails.company,
            jobDescription: $nextJobDetails.jobDescription,
            keyRequirements: $nextJobDetails.keyRequirements
        }
        await jobhunter.tauriCommand('write_job_description', { content: JSON.stringify(jobDescriptionData) })

        // Save applicant details
        const applicantDetailsData = {
            name: $nextJobApplication.name,
            experience: $nextJobApplication.experience,
            interests: $nextJobApplication.interests,
            projects: $nextJobApplication.projects,
            education: $nextJobApplication.education,
            certificates: $nextJobApplication.certificates
        }
        await jobhunter.tauriCommand('write_applicant_details', { content: JSON.stringify(applicantDetailsData) })

        await jobhunter.showMessage('Job application saved.', 'Success')
    } catch (error) {
        console.error('Error saving job application:', error)
        await jobhunter.showMessage('Failed to save. Try again?', {
            title: 'Could Not Save',
            type: 'error'
        })
    }
}


	async function submitQuery(): Promise<void> {
		try {
			const confirmed = await jobhunter.askConfirmation(
				'Are you happy with your job application details? They will be sent to Claude AI',
				{ title: 'Please Confirm', type: 'info' }
			)

			if (!confirmed) {
				return
			}

			const queryDetails = {
				job_title: get(nextJobDetails).jobTitle,
				company_name: get(nextJobDetails).company,
				job_description: get(nextJobDetails).jobDescription,
				key_requirements: get(nextJobDetails).keyRequirements,
				applicant_name: get(nextJobApplication).name,
				applicant_experience: get(nextJobApplication).experience,
				applicant_skills: get(nextJobApplication).interests,
				applicant_projects: get(nextJobApplication).projects,
				applicant_education: get(nextJobApplication).education,
				applicant_certificates: get(nextJobApplication).certificates
			}

			isGenerating.set(true)
			generatedLetter.set('')

			const result = await jobhunter.tauriCommand('suggestions', { queryDetails })

			if (typeof result === 'string') {
				generatedLetter.set(result)
			} else {
				throw new Error('Unexpected response from claude')
			}
		} catch (error) {
			await jobhunter.showMessage(`Failed to generate cover letter: ${error}`, {
				title: 'Error Generating Letter',
				type: 'error'
			})
		} finally {
			isGenerating.set(false)
		}
	}
</script>

{#if $initialised && !$fetching}
	<!----------- View All Unread Jobs -------------->
	<div class="flex flex-row text-sm">
		<!----------- 2/3rd left -------------->
		<div class="w-2/3 pl-2">
			<section
				class="h-[calc(58vh-200px)] snap-y snap-mandatory overflow-y-auto"
				style="scrollbar-width: none; -ms-overflow-style: none;"
			>
				{#if $jobSite}
					<div class="" transition:fly={{ x: 500, duration: 700 }}>
						{#if $unreadJobs.length === 0}
							<p>No jobs available for the selected source.</p>
						{:else}
							<ul class="">
								{#each $unreadJobs as job (job.id)}
									<div transition:fly={{ x: 500, duration: 700 }}>
										<View {job} on:refresh={updateJobRecord} />
									</div>
								{/each}
							</ul>
						{/if}
					</div>
				{:else}
					<div class="flex h-full flex-col items-center justify-center">
						<p class="font-semibold text-yellow-600">Click a job source to get started.</p>
					</div>
				{/if}
			</section>
		</div>

		<!-- 1/3rd right -->
		<div class="mr-5 w-1/3 pl-2">
			<div
				class="pb-2"
				on:click|stopPropagation={handleIndeedClick}
				on:keydown={handleIndeedKeydown}
				role="button"
				tabindex="0"
				style="transform: scale({$indeedSpring.scale}) rotate({$indeedSpring.rotate}deg);"
			>
				<Indeed jobs={$allJobs} on:loadNewJobs={fromHere} />
			</div>
			<div
				class=""
				on:click|stopPropagation={handleJoobleClick}
				on:keydown={handleJoobleKeydown}
				role="button"
				tabindex="0"
				style="transform: scale({$joobleSpring.scale}) rotate({$joobleSpring.rotate}deg);"
			>
				<Jooble jobs={$allJobs} on:loadNewJobs={fromHere} />
			</div>
			<!-- Add the job keywords input box -->
            <input
            type="text"
            bind:value={jobKeywords}
            placeholder="Enter job keywords"
            class="mt-4 w-full px-4 py-2 border rounded"
        />

        <!-- Add the Search Jooble button -->
        <button
            class="mt-2 w-full bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
            on:click={searchJooble}
            disabled={$fetching}
        >
            {$fetching ? 'Searching...' : 'Search Jooble'}
        </button>

        <!-- Add the Search Indeed button -->
        <button
            class="mt-2 w-full bg-yellow-500 hover:bg-yellow-700 text-white font-bold py-2 px-4 rounded"
            on:click={searchIndeed}
            disabled={$fetching}
        >
            {$fetching ? 'Searching...' : 'Search Indeed'}
        </button>
		</div>
	</div>

	<!-- Job Application and Cover Letter Draft Section -->
	<div class="mt-4 flex flex-row space-x-8 text-sm">
		<!-- Job Application Section -->
		<div
			class="w-1/3 space-y-8 rounded-lg bg-gradient-to-b from-yellow-50 to-yellow-400 p-6 shadow-lg dark:bg-gradient-to-b dark:from-slate-800 dark:to-slate-900"
		>
			<form on:submit|preventDefault={makeJobApplication} class="space-y-6">
				<!-- Job Details Section -->
				<div
					class="input-animation space-y-2 rounded-lg bg-white p-2 shadow-sm transition-all duration-300 focus-within:shadow-lg dark:bg-transparent dark:text-white"
				>
					<h3 class="text-lg font-semibold text-yellow-600">1. Job Details</h3>
					<BTA
						bind:value={$nextJobDetails.jobTitle}
						placeholder="Job Title"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobDetails.company}
						placeholder="Company Name"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobDetails.jobDescription}
						placeholder="Job Description"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobDetails.keyRequirements}
						placeholder="Key Requirements"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
				</div>

				<!-- Applicant Details Section -->
				<div
					class="input-animation space-y-2 rounded-lg bg-white p-2 shadow-sm transition-all duration-300 focus-within:shadow-lg dark:bg-transparent dark:text-white"
				>
					<h3 class="text-lg font-semibold text-yellow-600">2. My Details</h3>
					<BTA
						bind:value={$nextJobApplication.name}
						placeholder="Your Name"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobApplication.experience}
						placeholder="Your Experience"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobApplication.interests}
						placeholder="Your Skills"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobApplication.projects}
						placeholder="Your Projects"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobApplication.education}
						placeholder="Your Education"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
					<BTA
						bind:value={$nextJobApplication.certificates}
						placeholder="Your Certificates"
						className="transition-all duration-300 focus:ring-2 focus:ring-yellow-300"
					/>
				</div>

				<button
					type="submit"
					class="w-full rounded-md bg-yellow-600 px-2 py-2 text-white transition-all duration-300 hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-yellow-500 focus:ring-offset-2"
				>
					Save Job Application
				</button>
			</form>
		</div>

		<!-- Cover Letter Drafting Section -->
		<div
			class="w-2/3 space-y-8 rounded-lg bg-gradient-to-b from-yellow-50 to-yellow-400 p-6 shadow-lg dark:bg-gradient-to-b dark:from-slate-800 dark:to-slate-900"
		>
			<h3 class="text-lg font-semibold text-yellow-600">3. Cover Letter</h3>
			<div class="flex flex-col items-center justify-center text-sm">
				<button
					on:click={submitQuery}
					class="mb-2 rounded-2xl bg-yellow-600 px-4 py-4 text-neutral-100 transition-all duration-300 hover:bg-yellow-800"
					disabled={$isGenerating}
				>
					{$isGenerating ? 'Generating...' : 'Draft New Cover Letter'}
				</button>
			</div>

			<div class="min-h-[60vh] rounded-2xl bg-white p-6 shadow-sm dark:bg-transparent dark:text-neutral-100">
				{#if $isGenerating}
					<div
						in:fade={{ duration: 300 }}
						out:fade={{ duration: 300 }}
						class="flex flex-col items-center justify-center"
					>
						<p class="mb-2 text-center text-sm">
							Drafting your next cover letter... this might take a moment!
						</p>
						<StreamingAnimation class="h-48 w-64" />
					</div>
				{:else if $generatedLetter}
					<div
						in:fly={{ y: 20, duration: 300 }}
						out:fade={{ duration: 300 }}
						class="cover-letter-content text-sm"
					>
						{@html $generatedLetter}
					</div>
				{:else}
					<p in:fade={{ duration: 300 }} out:fade={{ duration: 300 }} class="text-center text-sm">
						Use the button above to tailor your cover letter to the job!
					</p>
				{/if}
			</div>
		</div>
	</div>
{:else if $fetching}
<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900 bg-opacity-75"
		transition:fade={{ duration: 300 }}
	>
		<div class="text-center">
			<div class="mb-4">
				<div
					class="inline-block h-16 w-16 animate-spin rounded-full border-4 border-solid border-yellow-400 border-r-transparent align-[-0.125em]"
					role="status"
				>
					<span
						class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]"
						>Loading...</span
					>
				</div>
			</div>
			<p class="text-lg font-semibold text-yellow-400">
				Updating job listings...
			</p>
			<p class="text-sm font-semibold text-yellow-400">
				This could take a minute. Please do not quit the app while this task is running!
			</p>
		</div>
	</div>
{:else}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-slate-900 bg-opacity-75"
		transition:fade={{ duration: 300 }}
	>
		<div class="text-center">
			<div class="mb-4">
				<div
					class="inline-block h-16 w-16 animate-spin rounded-full border-4 border-solid border-yellow-400 border-r-transparent align-[-0.125em]"
					role="status"
				>
					<span
						class="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]"
						>Loading...</span
					>
				</div>
			</div>
			<p class="text-lg font-semibold text-yellow-400">
				Starting up. Please wait...
			</p>
		</div>
	</div>
{/if}

<style>
	.input-animation {
		transition: transform 0.3s ease-out;
	}
	.input-animation:focus-within {
		transform: scale(1.02);
	}

	.cover-letter-content :global(p) {
		margin-bottom: 1.5em;
	}

	.cover-letter-content :global(p:last-child) {
		margin-bottom: 0;
	}
</style>
