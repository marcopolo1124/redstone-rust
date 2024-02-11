import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from "vite-plugin-wasm"
import tla from "vite-plugin-top-level-await"

export default defineConfig({
	server: {
		fs: {
		  // Allow serving files from one level up to the project root
		  allow: ['..'],
		},
	},
	plugins: [sveltekit(), wasm("../"), tla()],

});
