export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set([]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.DfWJuEAq.js",app:"_app/immutable/entry/app.CMo8zyl8.js",imports:["_app/immutable/entry/start.DfWJuEAq.js","_app/immutable/chunks/CyGCQZqp.js","_app/immutable/chunks/DDBjOw2-.js","_app/immutable/chunks/BoqWdf-N.js","_app/immutable/entry/app.CMo8zyl8.js","_app/immutable/chunks/CyGCQZqp.js","_app/immutable/chunks/DRRgAQ6G.js","_app/immutable/chunks/DH0GcLFM.js","_app/immutable/chunks/BoqWdf-N.js","_app/immutable/chunks/BE2rU0OQ.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/reader/[chapterId]",
				pattern: /^\/reader\/([^/]+?)\/?$/,
				params: [{"name":"chapterId","optional":false,"rest":false,"chained":false}],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
