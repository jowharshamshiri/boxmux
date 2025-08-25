<script>
	import Features from '$lib/components/home/features.svelte';
	import Footer from '$lib/components/home/footer.svelte';
	import Hero from '$lib/components/home/hero.svelte';
	import Nav from '$lib/components/home/nav.svelte';
	import { latestVersion, fetchLatestVersion } from '$lib/stores/version';
	import { onMount } from 'svelte';
	
	export let data;
	
	// Initialize the version store with data from the load function
	onMount(() => {
		if (data.latestVersion && data.latestVersion !== 'Loading...') {
			latestVersion.set(data.latestVersion);
		} else {
			fetchLatestVersion();
		}
	});
</script>

<Nav />
<Hero {data} />
<Features />
<Footer />
