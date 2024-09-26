<script lang="ts">
	import { readBinaryFile, writeBinaryFile, BaseDirectory } from '@tauri-apps/api/fs'
	import { onMount } from 'svelte'
	import * as pdfjs from 'pdfjs-dist'
	import pdfjsWorker from 'pdfjs-dist/build/pdf.worker.mjs?worker'
	import type { PDFDocumentProxy } from 'pdfjs-dist/types/src/display/api'
	import { createEventDispatcher } from 'svelte'
	import { jobhunter } from '$lib/jobIO'

	pdfjs.GlobalWorkerOptions.workerPort = new pdfjsWorker()

	export let title: string
	export let filename: string

	let pdfBlob: Blob | null = null
	let fileInfo: string = ''
	let pdfDoc: PDFDocumentProxy | null = null
	let canvas: HTMLCanvasElement
	let containerDiv: HTMLDivElement
	let fileInput: HTMLInputElement

	function triggerFileInput() {
		fileInput.click()
	}

	function removeFile() {
		fileInfo = ''
		pdfBlob = null
		filename = ''
		if (pdfDoc) {
			pdfDoc.destroy()
			pdfDoc = null
		}
		if (canvas) {
			const ctx = canvas.getContext('2d')
			if (ctx) ctx.clearRect(0, 0, canvas.width, canvas.height)
		}
		dispatch('fileUpdate', { filename: '', file: null })
	}

	const dispatch = createEventDispatcher()

	onMount(async () => {
		if (filename) {
			try {
				const content = await readBinaryFile(filename, { dir: BaseDirectory.Document })
				pdfBlob = new Blob([content], { type: 'application/pdf' })
				updateFileInfo()
				await renderPDF()
			} catch (error) {
				fileInfo = 'No PDF file uploaded yet.'
			}
		}
	})

	async function handleFileUpload(event: Event) {
		const input = event.target as HTMLInputElement
		const file = input.files?.[0]
		if (file && file.type === 'application/pdf') {
			removeFile()
			pdfBlob = file
			filename = file.name
			dispatch('fileUpdate', { filename, file })
			await saveContent()
			updateFileInfo()
			await renderPDF()
		} else {
			jobhunter.showMessage('Please select a PDF file')
		}
	}

	async function saveContent() {
		if (pdfBlob) {
			try {
				const arrayBuffer = await pdfBlob.arrayBuffer()
				await writeBinaryFile(filename, arrayBuffer, { dir: BaseDirectory.Document })
				jobhunter.showMessage(`Your CV: ${title} is saved!`)
			} catch (error) {
				jobhunter.showMessage(`Couldn't save ${title}. Please try again.`)
			}
		}
	}

	function updateFileInfo() {
		if (pdfBlob) {
			fileInfo = `PDF file uploaded. Size: ${pdfBlob.size} bytes`
		} else {
			fileInfo = 'No PDF file uploaded yet.'
		}
	}

	async function renderPDF() {
		if (pdfBlob && canvas) {
			try {
				const arrayBuffer = await pdfBlob.arrayBuffer()
				pdfDoc = await pdfjs.getDocument(new Uint8Array(arrayBuffer)).promise
				const page = await pdfDoc.getPage(1)

				const containerWidth = containerDiv ? containerDiv.clientWidth : window.innerWidth
				const viewport = page.getViewport({ scale: 1 })
				const scale = containerWidth / viewport.width
				const reducedScale = scale * 1 // Reduced to 90% to ensure it fits
				const scaledViewport = page.getViewport({ scale: reducedScale })

				canvas.height = scaledViewport.height
				canvas.width = scaledViewport.width

				const canvasContext = canvas.getContext('2d')
				if (canvasContext) {
					const renderContext = {
						canvasContext: canvasContext,
						viewport: scaledViewport,
						background: 'transparent'
					}

					canvasContext.globalCompositeOperation = 'difference'
					canvasContext.fillStyle = 'white'
					canvasContext.fillRect(0, 0, canvas.width, canvas.height)
					canvasContext.globalCompositeOperation = 'source-over'

					await page.render(renderContext)
				} else {
					console.error('Canvas context is null or undefined.')
				}
			} catch (error) {
				jobhunter.showMessage(`Error loading PDF: ${error}`)
			}
		}
	}
</script>

<div class="mx-auto w-full max-w-md">
	<h2 class="mb-4 text-center text-xl font-bold">{title}</h2>
	<div class="mb-4">
		<input type="file" accept=".pdf" on:change={handleFileUpload} class="hidden" bind:this={fileInput} />
		{#if fileInfo}
			<div class="flex items-center justify-between rounded-lg bg-gray-100 p-3 dark:bg-slate-950">
				<div class="flex items-center">
					<svg class="mr-2 h-8 w-8 text-red-500" fill="currentColor" viewBox="0 0 20 20">
						<path
							d="M9 2a2 2 0 00-2 2v8a2 2 0 002 2h6a2 2 0 002-2V6.414A2 2 0 0016.414 5L14 2.586A2 2 0 0012.586 2H9z"
						/>
						<path d="M3 8a2 2 0 012-2v10h2a2 2 0 01-2 2H5a2 2 0 01-2-2V8z" />
					</svg>
					<span class="max-w-xs truncate text-sm font-medium text-gray-900 dark:text-slate-300">
						{fileInfo}
					</span>
				</div>
				<button on:click={removeFile} class="ml-2 text-sm font-medium text-red-600 hover:text-red-500">
					Remove
				</button>
			</div>
		{:else}
			<button
				on:click={triggerFileInput}
				class="w-full rounded-md border border-transparent bg-yellow-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-yellow-700 focus:outline-none focus:ring-2 focus:ring-yellow-500 focus:ring-offset-2"
			>
				Upload PDF
			</button>
		{/if}
	</div>
	<div class="mt-4" bind:this={containerDiv}>
		<canvas bind:this={canvas} class="dark:invert"></canvas>
	</div>
</div>
