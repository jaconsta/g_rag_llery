<script lang="ts">
	let {
		status,
		preview
	}: {
		status: 'finished' | 'stopped' | 'pending' | 'uploading' | 'error' | 'duplicate';
		preview: string;
	} = $props();
</script>

<div class="flex flex-col items-center rounded-lg border bg-gray-50 p-3 shadow">
	<img src={preview} alt="Preview" class="mb-2 h-32 w-full rounded object-cover" />
	<div class="flex w-full justify-center">
		{#if status === 'pending'}
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
		{:else if status === 'uploading'}
			<span title="Uploading" class="flex animate-spin items-center gap-1 text-sm text-blue-500">
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
		{:else if status === 'error'}
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
		{:else if status === 'duplicate'}
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
		{:else if status === 'stopped'}
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
