import { writable } from 'svelte/store';

export const latestVersion = writable<string>('Loading...');

let versionPromise: Promise<string> | null = null;

export async function fetchLatestVersion(): Promise<string> {
	// Return cached promise if already fetching
	if (versionPromise) {
		return versionPromise;
	}
	
	versionPromise = (async () => {
		try {
			const response = await fetch('https://api.github.com/repos/jowharshamshiri/boxmux/releases/latest');
			if (response.ok) {
				const release = await response.json();
				const version = release.tag_name || release.name || 'Unknown';
				latestVersion.set(version);
				return version;
			}
		} catch (error) {
			console.warn('Failed to fetch latest version:', error);
		}
		
		// Fallback version
		const fallback = '0.189.31871';
		latestVersion.set(fallback);
		return fallback;
	})();
	
	return versionPromise;
}