import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	build: {
		ssr: false,
	  },
	plugins: [sveltekit()],
	optimizeDeps: {
		include: ['pdfjs-dist']
	  }
});
