import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		adapter: adapter({ fallback: 'index.html' }), // SPA fallback: gRPC clients render client-side
		prerender: { concurrency: 1 }
	}
};

export default config;
