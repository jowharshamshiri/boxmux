<script>
	import Features from '$lib/components/home/features.svelte';
	import Footer from '$lib/components/home/footer.svelte';
	import Hero from '$lib/components/home/hero.svelte';
	import Nav from '$lib/components/home/nav.svelte';
	
	export let data;
</script>

<Nav />
<Hero {data} />
<Features />
<Footer />
