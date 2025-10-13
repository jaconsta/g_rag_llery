<script lang="ts">
	import { resolve } from '$app/paths';
	let swReady = false;
	let autoUploadTimeout: NodeJS.Timeout | undefined = undefined;
	let autoUploadDelay = 5000; // 5 seconds
	let scheduled = false;
	let cancelled = false;
	let stopRequested = false;
	let files: File[] = [];
	let previews: string[] = [];
	let statuses: ('pending' | 'finished' | 'duplicate' | 'uploading' | 'error' | 'stopped')[] = [];
	let uploading = false;
	let singleFileInput: { value: string; click: () => void };
	let fileInput: { value: string; click: () => void };

	let uploadMessage = '';
	let uploadError = '';

	function scheduleAutoUpload() {
		clearTimeout(autoUploadTimeout);
		cancelled = false;
		stopRequested = false;
		scheduled = true;
		if (files.length > 0 && statuses.some((s) => s === 'pending')) {
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
		previews = files.map((file) => URL.createObjectURL(file));
		// Detect duplicates by name/size and SHA-256
		const hashes = await Promise.all(files.map((file) => sha256File(file)));
		statuses = files.map((file, i) => {
			// Name/size duplicate detection
			if (isDuplicateNameSize(file, files, i)) return 'duplicate';
			// SHA-256 duplicate detection
			if (hashes.indexOf(hashes[i]) !== i) return 'duplicate';
			return 'pending';
		});
		scheduleAutoUpload();
	}

	// Helper: detect duplicate by name/size
	async function handleSingleFileChange(event: Event) {
		const input = event.target as unknown as { files: File[] };
		if (!input.files) return;
		const selectedFiles = Array.from(input.files).filter((file) => file.type.startsWith('image/'));
		// Add to existing files
		const newFiles: File[] = [...files, ...selectedFiles];
		files = newFiles;
		previews = newFiles.map((file) => URL.createObjectURL(file));
		const hashes = await Promise.all(newFiles.map((file) => sha256File(file)));
		statuses = newFiles.map((file, i) => {
			if (isDuplicateNameSize(file, newFiles, i)) return 'duplicate';
			if (hashes.indexOf(hashes[i]) !== i) return 'duplicate';
			return 'pending';
		});

		if (singleFileInput) singleFileInput.value = '';
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
		statuses = statuses.map((s) => (s === 'uploading' || s === 'pending' ? 'stopped' : s));
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

	// Register service worker and listen for upload status
	if (typeof window !== 'undefined' && 'serviceWorker' in navigator) {
		navigator.serviceWorker.register('/service-worker.js').then(() => {
			if (!navigator.serviceWorker.controller) {
				navigator.serviceWorker.addEventListener('controllerchange', () => {
					swReady = true;
				});
				// Force reload so controller is set
				window.location.reload();
			} else {
				swReady = true;
			}
		});
		navigator.serviceWorker.addEventListener('message', (event) => {
			if (event.data && event.data.type === 'UPLOAD_STATUS') {
				const idx = files.findIndex(
					(f) => f.name === event.data.name && f.size === event.data.size
				);
				if (idx !== -1) {
					statuses[idx] = event.data.status;
				}
				if (statuses.every((s) => s === 'finished' || s === 'duplicate')) {
					uploadMessage = 'Upload complete!';
					files = [];
					previews = [];
					statuses = [];
					if (fileInput) fileInput.value = '';
					uploading = false;
				}
			}
		});
	}

	async function uploadAll() {
		if (stopRequested) return;
		uploading = true;
		uploadMessage = '';
		uploadError = '';
		// Prepare files for service worker (only non-duplicates)
		const uploadFiles = files
			.map((file) => ({ name: file.name, size: file.size, file }))
			.filter((_, i) => statuses[i] !== 'duplicate');

		for (const fi in uploadFiles) {
			console.log(fi);

			const formData = new FormData();
			formData.append('name', uploadFiles[fi].name);

			let signedUrl: string;
			try {
				const preResponse = await fetch('/api/uploads', {
					method: 'POST',
					headers: {},
					body: formData
				});

				const body = await preResponse.json();
				if (!body.success || typeof body.url !== 'string') {
					statuses[fi] = 'error';
					continue;
				}

				signedUrl = body.url;
			} catch (e) {
				console.log(e);
				statuses[fi] = 'error';
				continue;
			}

			try {
				const bucketFormData = new FormData();
				bucketFormData.append('file', uploadFiles[fi].file); // The file has be the last element

				const response = await fetch(signedUrl, {
					method: 'PUT',
					headers: {},
					body: bucketFormData
				});

				console.log(await response.text());
			} catch (e) {
				console.log(e);
				statuses[fi] = 'error';
				continue;
			}
		}
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
		previews = [];
		statuses = [];
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
				webkitdirectory
				style="display:none"
				on:change={handleFileChange}
				accept="image/*"
			/>
			<button
				class="mb-2 block w-full cursor-pointer rounded-lg border-2 border-blue-500 bg-blue-50 px-4 py-3 text-base text-gray-800 focus:ring-2 focus:ring-blue-400 focus:outline-none"
				type="button"
				on:click={() => fileInput.click()}>Select Images or Folder</button
			>
			<span class="mt-1 block text-sm text-gray-500"
				>You can select multiple images or a folder.</span
			>
		</label>
	{/if}
	{#if !uploading}
		<button
			class="mb-4 rounded bg-blue-100 px-3 py-2 text-blue-700 shadow"
			on:click={() => singleFileInput.click()}
			type="button">Add Individual Image</button
		>
		<input
			bind:this={singleFileInput}
			type="file"
			multiple
			accept="image/*"
			style="display:none"
			on:change={handleSingleFileChange}
		/>
	{/if}
	{#if files.length > 0}
		<div class="mb-4 grid grid-cols-2 gap-4 sm:grid-cols-3">
			{#each previews as preview, i (i)}
				<div class="flex flex-col items-center rounded-lg border bg-gray-50 p-3 shadow">
					<img src={preview} alt="Preview" class="mb-2 h-32 w-full rounded object-cover" />
					<div class="flex w-full justify-center">
						{#if statuses[i] === 'pending'}
							<span title="Pending" class="flex items-center gap-1 text-sm text-yellow-500">
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
									><circle cx="12" cy="12" r="10" stroke-width="2" /><path
										stroke-linecap="round"
										stroke-width="2"
										d="M12 8v4l2 2"
									/></svg
								> Pending
							</span>
						{:else if statuses[i] === 'uploading'}
							<span
								title="Uploading"
								class="flex animate-spin items-center gap-1 text-sm text-blue-500"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
									><circle cx="12" cy="12" r="10" stroke-width="2" /><path
										stroke-linecap="round"
										stroke-width="2"
										d="M12 8v4l2 2"
									/></svg
								> Uploading
							</span>
						{:else if statuses[i] === 'error'}
							<span title="Error" class="flex items-center gap-1 text-sm text-red-500">
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
									><circle cx="12" cy="12" r="10" stroke-width="2" /><path
										stroke-linecap="round"
										stroke-width="2"
										d="M12 8v8"
									/><path stroke-linecap="round" stroke-width="2" d="M12 16h.01" /></svg
								> Error
							</span>
						{:else if statuses[i] === 'duplicate'}
							<span title="Duplicate" class="flex items-center gap-1 text-sm text-yellow-400">
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
									><circle cx="12" cy="12" r="10" stroke-width="2" /><path
										stroke-linecap="round"
										stroke-width="2"
										d="M12 8v8"
									/><path stroke-linecap="round" stroke-width="2" d="M12 16h.01" /></svg
								> Duplicate
							</span>
						{:else if statuses[i] === 'stopped'}
							<span title="Stopped" class="flex items-center gap-1 text-sm text-gray-500">
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
									><circle cx="12" cy="12" r="10" stroke-width="2" /><path
										stroke-linecap="round"
										stroke-width="2"
										d="M12 8v8"
									/><path stroke-linecap="round" stroke-width="2" d="M12 16h.01" /></svg
								> Stopped
							</span>
						{:else}
							<span title="Finished" class="flex items-center gap-1 text-sm text-green-500">
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-4 w-4"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
									><circle cx="12" cy="12" r="10" stroke-width="2" /><path
										stroke-linecap="round"
										stroke-width="2"
										d="M12 16l4-4-4-4"
									/></svg
								> Finished
							</span>
						{/if}
					</div>
				</div>
			{/each}
		</div>
		{#if scheduled && !uploading}
			<div class="mb-2 flex items-center gap-2 rounded bg-yellow-50 p-2 text-yellow-800">
				<span>Upload will start automatically in {autoUploadDelay / 1000} seconds.</span>
				<button
					class="rounded bg-yellow-200 px-2 py-1 text-yellow-900"
					on:click={cancelScheduledUpload}>Cancel</button
				>
				<button class="rounded bg-blue-200 px-2 py-1 text-blue-900" on:click={triggerManualUpload}
					>Upload Now</button
				>
			</div>
		{/if}
		<div class="flex gap-2">
			<button
				class="rounded bg-blue-600 px-4 py-2 text-white shadow disabled:opacity-50"
				on:click={uploadAll}
				disabled={uploading || statuses.every((s) => s === 'finished') || scheduled}
				>{uploading ? 'Uploading...' : 'Upload All'}</button
			>
			<button class="rounded bg-gray-300 px-4 py-2 text-gray-800 shadow" on:click={reset}
				>Reset</button
			>
			{#if uploading}
				<button class="rounded bg-red-500 px-4 py-2 text-white shadow" on:click={stopUpload}
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
