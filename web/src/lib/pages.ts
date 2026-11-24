// Page bytes via fetch-to-blob (decision 26): <img> cannot carry the bearer token, so
// chapter pages ride the authed fetch, exactly like cover bytes do in covers.ts.
import { API_BASE, TOKEN } from './koma';

export async function fetchPageBlob(
	chapterId: bigint,
	pageN: number
): Promise<string | undefined> {
	const res = await fetch(`${API_BASE()}/pages/${chapterId}/${pageN}`, {
		headers: { Authorization: `Bearer ${TOKEN()}` }
	});
	if (!res.ok) return undefined;
	return URL.createObjectURL(await res.blob());
}
