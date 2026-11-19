// Connect transport + typed clients for the Koma server (fetch-to-blob pages + gRPC RPCs).
import { createClient } from '@connectrpc/connect';
import { createGrpcWebTransport } from '@connectrpc/connect-web';
import { KomaService, Library } from '../gen/koma/v1/koma_pb';

export const API_BASE = () => localStorage.getItem('koma_api') ?? 'http://127.0.0.1:18080';
export const TOKEN = () => localStorage.getItem('koma_token') ?? '';

// All requests carry the bearer (decision 24): transport-level fetch wrapper.
function authedFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
	const headers = new Headers(init?.headers);
	headers.set('Authorization', `Bearer ${TOKEN()}`);
	return fetch(input, { ...(init ?? {}), headers });
}

let libraryCache: ReturnType<typeof createClient<typeof Library>> | null = null;
let komaCache: ReturnType<typeof createClient<typeof KomaService>> | null = null;

export function library() {
	libraryCache ??= createClient(Library, createGrpcWebTransport({ baseUrl: API_BASE(), fetch: authedFetch, useBinaryFormat: true }));
	return libraryCache;
}

export function transportClient() {
	komaCache ??= createClient(KomaService, createGrpcWebTransport({ baseUrl: API_BASE(), fetch: authedFetch, useBinaryFormat: true }));
	return komaCache;
}

// fetch-to-blob (decision 26): page images ride the authed fetch, wrapped into object URLs.
export async function fetchPageBlob(chapterId: number, pageN: number): Promise<string | undefined> {
	const res = await fetch(`${API_BASE()}/pages/${chapterId}/${pageN}`, {
		headers: { Authorization: `Bearer ${TOKEN()}` }
	});
	if (!res.ok) return undefined;
	return URL.createObjectURL(await res.blob());
}
