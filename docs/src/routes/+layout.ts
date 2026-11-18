import { newestRelease } from '$lib/release';

// Every page is built once, with the newest release read from the manifest then.
export const prerender = true;

export async function load({ fetch }) {
	return { release: await newestRelease(fetch) };
}
