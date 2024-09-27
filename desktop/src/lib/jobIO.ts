import { readTextFile, writeTextFile, readBinaryFile, type FsOptions } from '@tauri-apps/api/fs'
import { join } from '@tauri-apps/api/path'
import { dialog } from '@tauri-apps/api'
import { invoke, type InvokeArgs } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { appWindow } from '@tauri-apps/api/window'

import { PUBLIC_FILES_PATH, PUBLIC_SAVED_CV_DETAILS, PUBLIC_NODE_ENV } from '$env/static/public'
import { allJobs, appliedTotal, fetchedTotal } from './jobHistory'
import type { ApplicantDetails, Job, Stats } from '.'

const dev = PUBLIC_NODE_ENV === 'development';

const initApplicant: ApplicantDetails = {
	name: '',
	experience: '',
	interests: '',
	projects: '',
	education: '',
	certificates: ''
}

class Jobhunter {
	constructor() {}

	async startServer() {
		try {
			return await invoke('start_python_server')
		} catch (error) {
			throw error
		}
	}

	async tauriCommand(command: string, args?: InvokeArgs): Promise<unknown> {
		try {
			return await invoke(command, args)
		} catch (error) {
			throw error
		}
	}

	async listen(event: string, callback: (...args: any[]) => void) {
		return await listen(event, callback)
	}

	async read(filepath: string, options?: FsOptions): Promise<string> {
		try {
			return await readTextFile(filepath, options)
		} catch (error) {
			throw error
		}
	}

	async readBinary(filePath: string, options?: FsOptions): Promise<Uint8Array | null> {
		try {
			return await readBinaryFile(filePath, options)
		} catch (error) {
			throw error
		}
	}

	async write(filepath: string, contents: string, options?: FsOptions): Promise<void> {
		try {
			await writeTextFile(filepath, contents, options)
		} catch (error) {
			throw error
		}
	}

	async resolvePath(...paths: string[]): Promise<string> {
		try {
			return await join(...paths)
		} catch (error) {
			throw error
		}
	}

	async showMessage(message: string, options?: string | dialog.MessageDialogOptions): Promise<void> {
		return await dialog.message(message, options)
	}

	async askConfirmation(
		message: string,
		options?: string | dialog.ConfirmDialogOptions
	): Promise<boolean | null> {
		return await dialog.confirm(message, options)
	}

	async dragWindow() {
		try {
			await appWindow.startDragging()
		} catch (error) {
			dialog.message('Failed to start dragging')
		}
	}
}

export const jobhunter = new Jobhunter()

export async function updateJobRecord(
	event: CustomEvent<{ jobId: number; field: 'read' | 'appliedto'; value: boolean }>
) {
	if (typeof window !== 'undefined') {
		const { jobId, field, value } = event.detail
		try {
			const jobUpdate = { [field]: value }
			const updatedJob = (await invoke('update_job', { jobId, jobUpdate })) as Job
			allJobs.update(($allJobs) => {
				return $allJobs.map((job) => {
					if (job.id === jobId) {
						return { ...job, ...updatedJob }
					}
					return job
				})
			})

			const stats = (await invoke('get_stats')) as Stats
			fetchedTotal.set(stats.uniquejobs)
			appliedTotal.set(stats.appliedjobs)
		} catch (error) {
			dialog.message(`Couldn't mark the job. Unexpected error: ${error}`)
		}
	}
}

export async function loadApplicantConfig() {
    try {
        if (dev) {
            const path = await jobhunter.resolvePath(PUBLIC_FILES_PATH, PUBLIC_SAVED_CV_DETAILS);
            const fileContents = await jobhunter.read(path);
            return JSON.parse(fileContents);
        } else {
            const fileContents = await jobhunter.tauriCommand('read_config') as string;
            return JSON.parse(fileContents);
        }
    } catch (error) {
        dialog.message(`Error loading applicant details: ${error}`);
        return initApplicant;
    }
}
