<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();

	if (typeof window !== 'undefined' && 'serviceWorker' in navigator) {
    window.addEventListener('load', () => {
      navigator.serviceWorker.register('/service-worker.js');
    });
  }
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	 <link rel="manifest" href="/manifest.json" />
</svelte:head>

{@render children?.()}
