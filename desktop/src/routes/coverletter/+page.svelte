<script lang="ts">
	import { PUBLIC_FILES_PATH, PUBLIC_SAVED_CONFIG } from '$env/static/public'
	import { CoverLetter } from '$components'
	import { currentLetter } from '$lib/jobApplication'
	import { jobhunter } from '$lib/jobIO'

	let coverLetterComponent: CoverLetter

	async function saveLetter(event: CustomEvent<string>) {
		try {
			const confirmed = await jobhunter.askConfirmation('Any previous content will be overwritten.', {
				title: 'Save New Cover Letter?',
				type: 'info'
			})
			if (confirmed) {
				coverLetterComponent.confirmSave()
				currentLetter.set(event.detail)
				await saveLetterParam(event.detail)
				if (event.type === 'save') {
					await jobhunter.showMessage('Saved cover letter.')
				}
			}
		} catch (error) {
			await jobhunter.showMessage('Error saving letter!', {
				title: 'Failed to Save Letter',
				type: 'error'
			})
		}
	}

	async function saveLetterParam(content: string) {
    await jobhunter.tauriCommand('write_config_file', { content: JSON.stringify({ coverLetter: content }) })
}

async function readLetterParam(): Promise<string> {
    const config = await jobhunter.tauriCommand('read_config_file') as string
    return JSON.parse(config).coverLetter || ''
}

</script>

<div class="h-screen w-full">
	<CoverLetter
		bind:this={coverLetterComponent}
		title="Cover Letter"
		content={$currentLetter}
		on:update={saveLetter}
		on:save={saveLetter}
	></CoverLetter>
</div>