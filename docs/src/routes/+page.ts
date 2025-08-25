export const prerender = true;

export async function load() {
	// Fetch latest release from GitHub API
	const response = await fetch('https://api.github.com/repos/jowharshamshiri/boxmux/releases/latest');
	const release = await response.json();
	
	return {
		latestVersion: release.tag_name || release.name,
		releaseUrl: release.html_url
	};
}