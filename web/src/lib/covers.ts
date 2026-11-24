// Cover object URLs via fetch-to-blob (decision 26 applies to cover bytes exactly as to
// page bytes: <img> cannot carry the bearer token, so covers ride the authed fetch).
// One fetch per manga per session, cached by id; null means placeholder.

import { API_BASE, TOKEN } from './koma';

const cache = new Map<string, string>();

export async function coverObjectUrl(mangaId: bigint): Promise<string | null> {
	const key = String(mangaId);
	const hit = cache.get(key);
	if (hit) return hit;
	try {
		const res = await fetch(`${API_BASE()}/covers/${mangaId}`, {
			headers: { Authorization: `Bearer ${TOKEN()}` }
		});
		if (!res.ok) return null;
		const url = URL.createObjectURL(await res.blob());
		cache.set(key, url);
		return url;
	} catch {
		return null;
	}
}
