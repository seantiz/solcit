<script lang="ts">
    import { onMount } from 'svelte';
    import { browser } from '$app/environment';
    import { jobhunter } from '$lib/jobIO';
    import { PUBLIC_FILES_PATH, PUBLIC_NODE_ENV } from '$env/static/public';

    let apiKey = '';
    const dev = PUBLIC_NODE_ENV === 'development'

    onMount(async () => {
        if(browser){
        // Fetch the current API key if it exists
        apiKey = await jobhunter.tauriCommand('get_key') as string;}
    });

    async function saveApiKey() {
        if(dev){
            try {
                const path = await jobhunter.resolvePath(PUBLIC_FILES_PATH, 'credentials.json')
                await jobhunter.write(path, JSON.stringify({ anthropic_api_key: apiKey }) )
            } catch (error) {
                console.log('Error saving API key. Did you remember to set NODE_ENV to development?:', error)
            }
        } else {

        try {
            await jobhunter.tauriCommand('set_key', { key: apiKey });
            await jobhunter.showMessage('API key saved successfully', 'Success');
        } catch (error) {
            console.error('Error saving API key:', error);
            await jobhunter.showMessage('Failed to save API key', 'Error');
        }}
    }
</script>

<div class="container mx-auto mt-8 h-screen">
    <h1 class="text-2xl font-bold mb-4">Anthropic API Key</h1>
    <form on:submit|preventDefault={saveApiKey}>
        <div class="mb-4">
            <label for="apiKey" class="block text-sm font-medium text-gray-700">API Key</label>
            <input type="password" id="apiKey" bind:value={apiKey} class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50" required>
        </div>
        <button type="submit" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
            Save API Key
        </button>
    </form>
</div>
