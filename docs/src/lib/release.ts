// BoxMux's newest release, read from its release manifest on bahramj.com when the site is built.
//
// The manifest lists, for each channel, every version BoxMux published and the newest one. The
// site shows the `release` channel's. A manifest that is not there means nothing is published
// yet, which the site says; any other answer, or a manifest of another shape, fails the build
// rather than show a version that is not the newest.

export const MANIFEST_URL = 'https://releases.bahramj.com/boxmux/manifest';

export type Release = {
	/** The newest version on the `release` channel, or null when none is published yet. */
	version: string | null;
	/** Where that version's release notes are. */
	notesUrl: string | null;
};

type Fetch = (input: string, init?: RequestInit) => Promise<Response>;

export async function newestRelease(fetch: Fetch): Promise<Release> {
	const response = await fetch(MANIFEST_URL, { headers: { 'Cache-Control': 'no-cache' } });
	if (response.status === 404) {
		return { version: null, notesUrl: null };
	}
	if (!response.ok) {
		throw new Error(`${MANIFEST_URL} answered ${response.status} ${response.statusText}`);
	}
	const manifest = await response.json();
	if (manifest?.appId !== 'boxmux') {
		throw new Error(`${MANIFEST_URL} is not BoxMux's release manifest (appId ${manifest?.appId})`);
	}
	const channel = manifest.channels?.release;
	if (!channel) {
		throw new Error(`${MANIFEST_URL} has no release channel`);
	}
	const version: string | null = channel.latestVersion ?? null;
	if (version === null) {
		return { version: null, notesUrl: null };
	}
	const published = channel.versions?.[version];
	if (!published || typeof published.notesUrl !== 'string') {
		throw new Error(`${MANIFEST_URL} names ${version} as the newest release and lists no notes for it`);
	}
	return { version, notesUrl: published.notesUrl };
}
