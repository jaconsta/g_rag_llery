<script lang="ts">
	import { onMount } from 'svelte';
	import type { PageProps } from './$types';

	interface Photo {
		src: string;
		caption: string;
		aspect: string;
		theme: string;
		alt?: string;
	}

	let { data }: PageProps = $props();
	let { photos } = data as {
		photos: Photo[];
		// aspects: string[];
		// themes: string[];
	};

	let innerWidth = $state(0);
	let innerHeight = $state(0);
	// Show the filter only when reaching mobile size
	let showFilterButton = $derived(innerWidth <= 800);
	let showFilters = $state(false);

	let searchText = $state('');
	let debouncedSearch = '';
	let aspect = $state('');
	let theme = $state('');
	let modalOpen = $state(false);
	let modalImg = $state('');
	let modalCaption = $state('');
	let modalIndex = $state(-1);
	// let searchTimeout;

	// // Debounce search input
	// $: if (typeof window !== 'undefined') {
	// 	if (searchTimeout) clearTimeout(searchTimeout);
	// 	searchTimeout = setTimeout(() => {
	// 		debouncedSearch = searchText;
	// 	}, 350);
	// }

	const aspectRatioStyles = {
		landscape: '',
		square: 'aspect-ratio:1/1',
		portrait: 'aspect-ratio:2/3',
		wide: 'aspect-ratio:16/7',
		tall: 'aspect-ratio:9/16'
	} as Record<string, string>;
	function aspectRatioStyle(aspect: string): string {
		return aspectRatioStyles[aspect] ?? '';
	}

	let filteredPhotos = $derived(
		photos.filter((photo) => {
			let show = true;
			if (debouncedSearch && !photo.caption.toLowerCase().includes(debouncedSearch.toLowerCase()))
				show = false;
			if (aspect && photo.aspect !== aspect) show = false;
			if (theme && photo.theme !== theme) show = false;
			return show;
		})
	);

	// This works here not, but should be server rendered
	let [visibleThemes, visibleAspects] = $derived.by(() => {
		const visibleThemes: string[] = [];
		const visibleAspects: string[] = [];
		filteredPhotos.forEach((photo) => {
			visibleThemes.push(photo.theme);
			visibleAspects.push(photo.aspect);
		});
		return [[...new Set(visibleThemes)], [...new Set(visibleAspects)]];
	});
	// $: filteredPhotos = photos.filter((photo) => {
	// 	let show = true;
	// 	if (debouncedSearch && !photo.caption.toLowerCase().includes(debouncedSearch.toLowerCase()))
	// 		show = false;
	// 	if (aspect && photo.aspect !== aspect) show = false;
	// 	if (theme && photo.theme !== theme) show = false;
	// 	return show;
	// });

	// Infinite scroll state
	let visibleCount = $state(12);
	let loadingMore = false;
	function handleScroll() {
		if (loadingMore) return;
		if (typeof window === 'undefined') return;
		const scrollY = window.scrollY;
		const viewport = window.innerHeight;
		const fullHeight = document.body.offsetHeight;
		if (scrollY + viewport > fullHeight - 200) {
			loadingMore = true;
			setTimeout(() => {
				visibleCount += 8;
				loadingMore = false;
			}, 300);
		}
	}
	onMount(() => {
		window.addEventListener('scroll', handleScroll);
		function handleKeydown(e: { key: string }) {
			if (modalOpen && e.key === 'Escape') closeModal();
		}
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('scroll', handleScroll);
			window.removeEventListener('keydown', handleKeydown);
		};
	});

	function openModal(photo: Photo) {
		modalIndex = filteredPhotos.findIndex((p) => p === photo);
		modalImg = photo.src.replace(/w=\d+/, 'w=1200');
		modalCaption = photo.caption;
		modalOpen = true;
	}
	function closeModal() {
		modalOpen = false;
		modalImg = '';
	}
	function showPrev() {
		if (modalIndex > 0) {
			modalIndex -= 1;
			const prevPhoto = filteredPhotos[modalIndex];
			modalImg = prevPhoto.src.replace(/w=\d+/, 'w=1200');
			modalCaption = prevPhoto.caption;
		}
	}
	function showNext() {
		if (modalIndex < filteredPhotos.length - 1) {
			modalIndex += 1;
			const nextPhoto = filteredPhotos[modalIndex];
			modalImg = nextPhoto.src.replace(/w=\d+/, 'w=1200');
			modalCaption = nextPhoto.caption;
		}
	}
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="mx-auto max-w-5xl rounded-lg bg-white p-5 shadow-lg">
	<h1 class="mb-8 text-center text-3xl font-bold text-gray-800">Photo Gallery</h1>
	<div class="mb-8 flex flex-wrap items-center justify-center gap-4">
		<div class="flex w-full items-center gap-2 sm:w-auto">
			<input
				type="text"
				bind:value={searchText}
				placeholder="Search photos..."
				class="w-full rounded-md border border-gray-300 px-4 py-2 text-base focus:ring-2 focus:ring-blue-400 focus:outline-none sm:w-64"
			/>

			{#if showFilterButton}
				<button
					class="flex items-center justify-center rounded-full bg-gray-800 p-2 text-white"
					onclick={() => (showFilters = !showFilters)}
					aria-label="Toggle filters"
				>
					<svg
						width="24"
						height="24"
						viewBox="0 0 24 24"
						fill="none"
						xmlns="http://www.w3.org/2000/svg"
					>
						<path
							d="M3 5h18M6 12h12M10 19h4"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
						/>
					</svg>
				</button>
			{/if}
		</div>
		{#if (showFilters && showFilterButton) || !showFilterButton}
			<div class="flex w-full flex-col gap-2 sm:w-auto sm:flex-row">
				<select
					bind:value={aspect}
					class="rounded-md border border-gray-300 px-4 py-2 text-base focus:ring-2 focus:ring-blue-400 focus:outline-none"
				>
					<option value="">All Aspects</option>
					{#each visibleAspects as aspectOpt (aspectOpt)}
						<option value={aspectOpt}
							>{aspectOpt.charAt(0).toUpperCase()}{aspectOpt.slice(1)}</option
						>
					{/each}
				</select>
				<select
					bind:value={theme}
					class="rounded-md border border-gray-300 px-4 py-2 text-base focus:ring-2 focus:ring-blue-400 focus:outline-none"
				>
					<option value="">All Themes</option>
					{#each visibleThemes as themeOpt (themeOpt)}
						<option value={themeOpt}>{themeOpt.charAt(0).toUpperCase()}{themeOpt.slice(1)}</option>
					{/each}
				</select>
			</div>
		{/if}
	</div>
	<div class="columns-1 gap-6 sm:columns-2 lg:columns-3">
		<!--
		{#each filteredPhotos.slice(0, visibleCount) as photo, i (i)}
    -->
		{#each filteredPhotos as photo, i (i)}
			<div
				class="gallery-item group relative mb-6 cursor-pointer overflow-hidden rounded-md bg-gray-50 shadow transition-shadow duration-200"
				style={aspectRatioStyle(photo.aspect)}
				role="button"
				tabindex="0"
				aria-label={`View photo: ${photo.caption}`}
				onclick={() => openModal(photo)}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') openModal(photo);
				}}
			>
				<img src={photo.src} alt={photo.alt} class="gallery-photo block w-full object-cover" />
				<div
					class="photo-caption absolute bottom-0 left-0 z-10 w-full bg-gradient-to-t from-black/70 via-black/20 to-transparent px-3 py-2 text-left text-base text-white opacity-0 transition-opacity duration-300 group-hover:opacity-100"
				>
					{photo.caption}
				</div>
			</div>
		{/each}
		{#if visibleCount < filteredPhotos.length}
			<div
				class="w-full animate-pulse py-6 text-center text-gray-500"
				role="status"
				aria-live="polite"
			>
				Loading more photos...
			</div>
		{/if}
	</div>

	{#if modalOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="modal fixed inset-0 z-50 flex items-center justify-center bg-black/70"
			role="dialog"
			aria-modal="true"
			aria-label={modalCaption ? `Photo modal: ${modalCaption}` : 'Photo modal'}
			tabindex="0"
			onclick={closeModal}
		>
			<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<div
				class="modal-content relative flex max-h-[90vh] w-full max-w-3xl flex-col items-center rounded-lg bg-white p-0 shadow-xl"
				role="document"
				tabindex="0"
				onclick={(e) => e.stopPropagation()}
			>
				<button
					class="modal-close absolute top-3 right-4 z-10 cursor-pointer border-none bg-transparent text-3xl text-gray-700"
					onclick={closeModal}
					aria-label="Close"
					tabindex="0">&times;</button
				>
				{#if modalIndex > 0}
					<button
						class="modal-arrow absolute top-1/2 left-2 z-10 -translate-y-1/2 cursor-pointer rounded-full border-none bg-white/80 px-2 py-1 text-3xl text-gray-700 shadow"
						onclick={showPrev}
						aria-label="Previous photo"
						tabindex="0"
					>
						<span class="icon">&#8592;</span>
					</button>
				{/if}
				{#if modalIndex < filteredPhotos.length - 1}
					<button
						class="modal-arrow absolute top-1/2 right-2 z-10 -translate-y-1/2 cursor-pointer rounded-full border-none bg-white/80 px-2 py-1 text-3xl text-gray-700 shadow"
						onclick={showNext}
						aria-label="Next photo"
						tabindex="0"
					>
						<span class="icon">&#8594;</span>
					</button>
				{/if}
				<img
					src={modalImg}
					alt="Large view"
					class="modal-img max-h-[70vh] max-w-full rounded-t-lg object-contain"
				/>
				<div class="modal-caption p-4 text-center text-lg text-gray-800">{modalCaption}</div>
			</div>
		</div>
	{/if}
</div>
