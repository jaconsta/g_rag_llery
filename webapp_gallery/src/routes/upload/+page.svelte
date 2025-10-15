<script lang="ts">
	import { resolve } from '$app/paths';
	import Preview from '../../components/upload/preview.svelte';
	let autoUploadTimeout: NodeJS.Timeout | undefined = undefined;
	let autoUploadDelay = 5000; // 5 seconds
	let autoUploadCountdown = $state(autoUploadDelay / 1000);
	let autoUploadCountdownTimeout: NodeJS.Timeout | undefined = undefined;
	let scheduled = $state(false);
	let cancelled = $state(false);
	let uploading = $state(false);
	let stopRequested = false; // Fix
	let files: File[] = $state([]);

	let filesMetadata: {
		hash: string;
		status: 'pending' | 'finished' | 'duplicate' | 'uploading' | 'error' | 'stopped';
		preview: string;
	}[] = $state([]);
	let isFolder = $state(false);
	let fileInput: { value: string; click: () => void } = $state({ value: '', click: () => {} });

	let uploadMessage = $state('');
	let uploadError = $state('');

	function countdownTimeout() {
		clearTimeout(autoUploadCountdownTimeout);

		autoUploadTimeout = setTimeout(() => {
			if (!cancelled || autoUploadCountdown > 0) {
				autoUploadCountdown = autoUploadCountdown - 1;
				countdownTimeout();
			}
		}, 1000);
	}
	function scheduleAutoUpload() {
		clearTimeout(autoUploadTimeout);
		cancelled = false;
		stopRequested = false;
		scheduled = true;
		autoUploadCountdown = autoUploadDelay / 1000;
		if (files.length > 0 && filesMetadata.map((i) => i.status).some((s) => s === 'pending')) {
			countdownTimeout();
			autoUploadTimeout = setTimeout(() => {
				if (!cancelled) {
					uploadAll();
				}
				scheduled = false;
			}, autoUploadDelay);
		}
	}

	async function handleFileChange(event: Event) {
		const input = event.target as unknown as { files: File[] };

		if (!input?.files) return;
		const selectedFiles = Array.from(input.files).filter((file) => file.type.startsWith('image/'));
		files = selectedFiles;
		const previews = files.map((file) => URL.createObjectURL(file));
		// Detect duplicates by name/size and SHA-256
		const hashes = await Promise.all(files.map((file) => sha256File(file)));
		const statuses = files.map((file, i) => {
			// Name/size duplicate detection
			if (isDuplicateNameSize(file, files, i)) return 'duplicate';
			// SHA-256 duplicate detection
			if (hashes.indexOf(hashes[i]) !== i) return 'duplicate';
			return 'pending';
		});

		filesMetadata = hashes.map((hash, i) => ({ hash, status: statuses[i], preview: previews[i] }));
		scheduleAutoUpload();
	}

	function cancelScheduledUpload() {
		clearTimeout(autoUploadTimeout);
		cancelled = true;
		scheduled = false;
	}

	function triggerManualUpload() {
		clearTimeout(autoUploadTimeout);
		scheduled = false;
		uploadAll();
	}

	function stopUpload() {
		stopRequested = true;
		uploading = false;
		uploadMessage = '';
		uploadError = 'Upload stopped by user.';
		filesMetadata.map((s) => (s.status === 'uploading' || s.status === 'pending' ? 'stopped' : s));
		// Send cancel message to service worker
		if (navigator.serviceWorker.controller) {
			navigator.serviceWorker.controller.postMessage({ type: 'CANCEL_UPLOAD' });
		}
	}

	function isDuplicateNameSize(file: File, files: File[], idx: number) {
		return (
			files.findIndex((f, i) => i !== idx && f.name === file.name && f.size === file.size) !== -1
		);
	}

	// Helper: compute SHA-256 hash of file
	async function sha256File(file: File) {
		const arrayBuffer = await file.arrayBuffer();
		const hashBuffer = await crypto.subtle.digest('SHA-256', arrayBuffer);
		// Convert buffer to hex string
		return Array.from(new Uint8Array(hashBuffer))
			.map((b) => b.toString(16).padStart(2, '0'))
			.join('');
	}

	// // Register service worker and listen for upload status
	// if (typeof window !== 'undefined' && 'serviceWorker' in navigator) {
	// 	navigator.serviceWorker.register('/service-worker.js').then(() => {
	// 		if (!navigator.serviceWorker.controller) {
	// 			navigator.serviceWorker.addEventListener('controllerchange', () => {
	// 				swReady = true;
	// 			});
	// 			// Force reload so controller is set
	// 			window.location.reload();
	// 		} else {
	// 			swReady = true;
	// 		}
	// 	});
	// 	navigator.serviceWorker.addEventListener('message', (event) => {
	// 		if (event.data && event.data.type === 'UPLOAD_STATUS') {
	// 			const idx = files.findIndex(
	// 				(f) => f.name === event.data.name && f.size === event.data.size
	// 			);
	// 			if (idx !== -1) {
	// 				statuses[idx] = event.data.status;
	// 			}
	// 			if (statuses.every((s) => s === 'finished' || s === 'duplicate')) {
	// 				uploadMessage = 'Upload complete!';
	// 				files = [];
	// 				previews = [];
	// 				statuses = [];
	// 				if (fileInput) fileInput.value = '';
	// 				uploading = false;
	// 			}
	// 		}
	// 	});
	// }

	async function uploadAll() {
		if (stopRequested) return;
		uploading = true;
		uploadMessage = '';
		uploadError = '';

		for (const fi in files) {
			// Here duplicate finds for duplicates client level
			if (filesMetadata[fi].status === 'duplicate') continue;
			filesMetadata[fi].status = 'uploading';

			const formData = new FormData();
			formData.append('name', files[fi].name);
			formData.append('size', `${files[fi].size}`);
			formData.append('hash', filesMetadata[fi].hash);

			let signedUrl: string;
			try {
				const preResponse = await fetch('/api/uploads', {
					method: 'POST',
					headers: {},
					body: formData
				});

				const body = await preResponse.json();
				if (!body.success || typeof body.url !== 'string') {
					if (body.duplicate) {
						// The server found a duplicate file (for the user) in the database
						filesMetadata[fi].status = 'duplicate';
					} else {
						filesMetadata[fi].status = 'error';
					}
					continue;
				}

				signedUrl = body.url;
			} catch (e) {
				console.log(e);
				filesMetadata[fi].status = 'error';
				continue;
			}

			try {
				const bucketFormData = new FormData();
				bucketFormData.append('file', files[fi]); // The file must be the last element

				// Response is empty
				await fetch(signedUrl, {
					method: 'PUT',
					headers: {},
					body: bucketFormData
				});

				filesMetadata[fi].status = 'finished';
			} catch (e) {
				console.log(e);
				filesMetadata[fi].status = 'error';
				continue;
			}
		}

		uploadMessage = 'Complete';
		// // Convert files to transferable objects
		// const fileDatas = await Promise.all(
		// 	uploadFiles.map(async ({ name, size, file }) => {
		// 		if (stopRequested) return null;
		// 		const arrayBuffer = await file.arrayBuffer();
		// 		return { name, size, buffer: arrayBuffer };
		// 	})
		// );
		// if (stopRequested) {
		// 	uploading = false;
		// 	return;
		// }
		// // Send files to service worker
		// if (navigator.serviceWorker.controller) {
		// 	console.log('about to serviceWorker');
		// 	navigator.serviceWorker.controller.postMessage({
		// 		type: 'UPLOAD_IMAGES',
		// 		files: fileDatas.filter(Boolean)
		// 	});
		// } else {
		// 	uploadError = swReady
		// 		? 'Service worker not available.'
		// 		: 'Service worker not ready. Please reload.';
		// 	uploading = false;
		// }
	}

	function reset() {
		files = [];
		filesMetadata = [];
		uploading = false;
		uploadMessage = '';
		uploadError = '';
		scheduled = false;
		cancelled = false;
		stopRequested = false;
		clearTimeout(autoUploadTimeout);
		if (fileInput) fileInput.value = '';
	}
</script>

<div class="mx-auto mt-10 max-w-2xl rounded-lg bg-white p-6 shadow-lg">
	{#if uploading}
		<h2 class="mb-4 text-2xl font-bold text-gray-800">Uploading your images...</h2>
		<div class="mb-4 rounded bg-blue-50 p-3 text-base font-medium text-blue-700">
			Your images are being uploaded in the background.<br />
			Please wait until the process completes.<br />
			You can cancel at any time.
		</div>
	{:else}
		<h2 class="mb-4 text-2xl font-bold text-gray-800">Upload Images</h2>
		<label class="mb-4 block">
			<span class="mb-2 block text-lg font-semibold text-gray-700">Select Images</span>
			<input
				bind:this={fileInput}
				type="file"
				multiple
				webkitdirectory={isFolder}
				style="display:none"
				onchange={handleFileChange}
				accept="image/*"
			/>
			<button
				class="mb-2 block w-full cursor-pointer rounded-lg border-2 border-blue-500 bg-blue-50 px-4 py-3 text-base text-gray-800 focus:ring-2 focus:ring-blue-400 focus:outline-none"
				type="button"
				onclick={() => fileInput.click()}>Select Images or Folder</button
			>
			<span class="mt-1 block text-sm text-gray-500">
				<input type="checkbox" bind:checked={isFolder} />
				Select complete folder.</span
			>
		</label>
	{/if}
	{#if files.length > 0}
		<div class="mb-4 grid grid-cols-2 gap-4 sm:grid-cols-3">
			{#each filesMetadata as metadata, i (i)}
				<Preview status={metadata.status} preview={metadata.preview} />
			{/each}
		</div>
		{#if scheduled && !uploading}
			<div class="mb-2 flex items-center gap-2 rounded bg-yellow-50 p-2 text-yellow-800">
				<span>Upload will start automatically in {autoUploadCountdown} seconds.</span>
				<button
					class="rounded bg-yellow-200 px-2 py-1 text-yellow-900"
					onclick={cancelScheduledUpload}>Cancel</button
				>
				<button class="rounded bg-blue-200 px-2 py-1 text-blue-900" onclick={triggerManualUpload}
					>Upload Now</button
				>
			</div>
		{/if}
		<div class="flex gap-2">
			{#if !uploadMessage}
				<button
					class="rounded bg-blue-600 px-4 py-2 text-white shadow disabled:opacity-50"
					onclick={uploadAll}
					disabled={uploading || filesMetadata.every((s) => s.status === 'finished') || scheduled}
					>{uploading ? 'Uploading...' : 'Upload All'}</button
				>
			{/if}
			<button class="rounded bg-gray-300 px-4 py-2 text-gray-800 shadow" onclick={reset}
				>Reset</button
			>
			{#if uploading && !uploadMessage}
				<button class="rounded bg-red-500 px-4 py-2 text-white shadow" onclick={stopUpload}
					>Stop Upload</button
				>
			{/if}
		</div>
	{/if}

	{#if uploadMessage}
		<div class="mt-6 flex items-center justify-between rounded bg-green-100 p-4 text-green-800">
			<span>{uploadMessage}</span>
			<a href={resolve('/gallery')} class="ml-4 font-semibold text-blue-700 underline"
				>Go to Gallery</a
			>
		</div>
	{/if}
	{#if uploadError}
		<div class="mt-6 rounded bg-red-100 p-4 text-red-800">{uploadError}</div>
	{/if}
</div>
