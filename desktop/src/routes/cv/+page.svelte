<script lang="ts">
	import { currentCV, nextJobApplication, currentLetter } from '$lib/jobApplication'
	import type { ApplicantDetails } from '$lib'
	import { jobhunter } from '$lib/jobIO'
    import { resourceDir, appDataDir } from '@tauri-apps/api/path';

	import { PDF, BTA } from '$components'

	import * as pdfjs from 'pdfjs-dist'
	import type { PDFDocumentProxy } from 'pdfjs-dist'

	import { onMount } from 'svelte'
	import { get } from 'svelte/store'
	import { PUBLIC_FILES_PATH, PUBLIC_SAVED_CONFIG, PUBLIC_NODE_ENV } from '$env/static/public'

	let uploading = false
	let currentPDFContent: Blob | null = null
	let preloadedCVPath: string | null = null
	let extractedResult: ApplicantDetails | null = null
    const dev = PUBLIC_NODE_ENV === 'development'

    async function extractCVDetails() {
    if (!preloadedCVPath) {
        await jobhunter.showMessage('Please upload a CV first.', { title: 'No CV Found', type: 'error' });
        return;
    }

    const confirmed = await jobhunter.askConfirmation(
        'This is an experimental feature using Claude AI. Are you sure?',
        'Confirm Extraction'
    );

    if (!confirmed) {
        return;
    }

    uploading = true;
    try {
        let pdfContent: ArrayBuffer;

        if (currentPDFContent) {
            pdfContent = await currentPDFContent.arrayBuffer();
        } else {
            let path: string;
            if (dev) {
                path = await jobhunter.resolvePath(PUBLIC_FILES_PATH, preloadedCVPath);
            } else {
                const configpath = await appDataDir();
                path = await jobhunter.resolvePath(configpath, preloadedCVPath);
            }

            const fileContent = await jobhunter.readBinary(path);
            if (!fileContent) {
                throw new Error('Failed to read file content');
            }
            pdfContent = fileContent.buffer;
        }

        const extractedText = await extractText(pdfContent);

        if (!extractedText) {
            throw new Error('Cannot extract CV details');
        }

        const rawText = preprocessText(extractedText);
        const preprocessedText = rawText.join(' ');
        extractedResult = await jobhunter.tauriCommand('extract_cv', {
            preprocessedText: preprocessedText
        }) as ApplicantDetails;

        await jobhunter.showMessage('CV details extracted. Click save to accept.', 'Success');

    } catch (error) {
        await jobhunter.showMessage(`Failed to process CV details: ${error}`, {
            title: 'Failed to Extract From CV',
            type: 'error'
        });
    } finally {
        uploading = false;
    }
}

	async function extractText(pdfContent: ArrayBuffer): Promise<string | undefined> {
		let pdf: PDFDocumentProxy | null = null
		try {
			const loadingTask = pdfjs.getDocument(new Uint8Array(pdfContent))
			pdf = await loadingTask.promise

			if (pdf === null) {
				await jobhunter.showMessage('Failed to load PDF document', {
					title: 'PDF Not Loaded',
					type: 'error'
				})
				return undefined
			}

			const pageTexts = await Promise.all(
				Array.from({ length: pdf.numPages }, async (_, i) => {
					const page = await pdf!.getPage(i + 1)
					const textContent = await page.getTextContent()
					return textContent.items.map((item: any) => item.str).join(' ')
				})
			)

			return pageTexts.join('\n')
		} catch (error) {
			await jobhunter.showMessage(`Extracting text failed: ${error}`, { title: 'Error', type: 'error' })
			return undefined
		} finally {
			if (pdf) {
				pdf.destroy()
			}
		}
	}

	async function saveConfig(config: { cvFilename: string; coverLetter: string }) {
        let path:string
		try {
            if(dev) {
			path = await jobhunter.resolvePath(PUBLIC_FILES_PATH, PUBLIC_SAVED_CONFIG) }
            else {
                const resources = await resourceDir()
                path = await jobhunter.resolvePath(resources, 'config.json') }
			await jobhunter.write(path, JSON.stringify(config))
		} catch (error) {
			throw error
		}
	}

	async function uploadCV(event: CustomEvent<{ filename: string; file: Blob }>) {
		const { filename, file } = event.detail

		// Check using magic numbers
		const fileHeader = await file.slice(0, 4).arrayBuffer()
		const view = new Uint8Array(fileHeader)
		if (view[0] !== 0x25 || view[1] !== 0x50 || view[2] !== 0x44 || view[3] !== 0x46) {
			await jobhunter.showMessage('The uploaded file is not a valid PDF.', {
				title: 'File is not a valid PDF',
				type: 'error'
			})
			return
		}

		currentCV.set(filename)
		currentPDFContent = file

		try {
			await saveConfig({
				cvFilename: filename,
				coverLetter: get(currentLetter)
			})
			await jobhunter.showMessage('CV uploaded successfully!')
		} catch (error) {
			await jobhunter.showMessage(`Failed to upload CV: ${error}`, {
				title: 'CV Not Loaded',
				type: 'error'
			})
		}
	}

	function preprocessText(text: string): string[] {
		const cleanText = text.replace(/\s+/g, ' ').trim()
		const sentences = cleanText.match(/[^.!?]+[.!?]+/g) || []
		return sentences.map((sentence) => sentence.trim())
	}

	async function updateDetails() {
		try {
			if (extractedResult) {
				const confirmed = await jobhunter.askConfirmation(
					'Do you want to save the extracted details?',
					'Confirm Save'
				)
				if (confirmed) {
					nextJobApplication.update((current) => ({
						...current,
						...extractedResult,
						name: current.name
					}))
					extractedResult = null
					await jobhunter.showMessage('Saved details to your job application.')
				} else {
					extractedResult = null
					await jobhunter.showMessage('Cleared!')
				}
			} else {
				const confirmed = await jobhunter.askConfirmation(
					'Any previous data will be overwritten.',
					'Update Application Details?'
				)
				if (confirmed) {
					nextJobApplication.update((current) => ({
						...current,
						experience: current.experience,
						interests: current.interests,
						projects: current.projects,
						education: current.education,
						certificates: current.certificates
					}))
					await jobhunter.showMessage('Saved')
				} else {
					await jobhunter.showMessage('Details were not saved.', {
						title: 'Failed Save Attempt',
						type: 'error'
					})
				}
			}
			await saveConfig({
				cvFilename: get(currentCV),
				coverLetter: get(currentLetter)
			})
		} catch (e) {
			await jobhunter.showMessage('Save failed. Please try again.', {
				title: 'Failed Save Attempt',
				type: 'error'
			})
		}
	}

	onMount(() => {
		currentCV.subscribe((value) => {
			preloadedCVPath = value
		})
	})
</script>

<div class="flex h-screen w-full">
	<div class="flex w-1/2 flex-grow justify-center">
		<PDF title="Current CV" filename={$currentCV} on:fileUpdate={uploadCV} />
	</div>

	<div class="flex w-1/2 flex-grow justify-center pr-10">
		{#if uploading}
			<p>Processing CV...</p>
		{:else}
			<div class="w-full">
				<div class="mb-4 flex justify-center">
					<button
						on:click={extractCVDetails}
						class="mt-4 rounded-md bg-neutral-700 px-4 py-2 text-white hover:bg-neutral-800 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
					>
						Try Automatically Extracting My Details ⚠
					</button>
				</div>
				<form on:submit|preventDefault={updateDetails} class="space-y-2">
					<input
						bind:value={$nextJobApplication.name}
						placeholder="Full Name"
						class="w-full rounded-md border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-yellow-600 dark:bg-transparent dark:text-neutral-100"
					/>
					{#if extractedResult}
						<BTA bind:value={extractedResult.experience} placeholder="My Experience" />
						<BTA bind:value={extractedResult.interests} placeholder="Interests" />
						<BTA bind:value={extractedResult.projects} placeholder="My Projects" />
						<BTA bind:value={extractedResult.education} placeholder="My Education" />
						<BTA bind:value={extractedResult.certificates} placeholder="My Certified Skills" />
					{:else}
						<BTA bind:value={$nextJobApplication.experience} placeholder="My Experience" />
						<BTA bind:value={$nextJobApplication.interests} placeholder="My Interests" />
						<BTA bind:value={$nextJobApplication.projects} placeholder="My Projects" />
						<BTA bind:value={$nextJobApplication.education} placeholder="My Education" />
						<BTA bind:value={$nextJobApplication.certificates} placeholder="My Certified Skills" />
					{/if}
					<button
						type="submit"
						class="w-full rounded-md bg-yellow-600 px-4 py-2 text-white hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-yellow-500 focus:ring-offset-2"
					>
						{#if extractedResult}
							Accept and Save Extracted Details
						{:else}
							Save My Details
						{/if}
					</button>
				</form>
			</div>
		{/if}
	</div>
</div>
