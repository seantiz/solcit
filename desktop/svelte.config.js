import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
            ssr: false,
			pages: 'build',
			assets: 'build',
			fallback: 'index.html',
			precompress: false,
			strict: true
		}),
        prerender: {
            entries: ['*']
          },
		appDir: 'app',
		alias: {
			"$components" : path.resolve("src/components")
		},
	},
};

export default config;
