<script lang="ts">
	import { page } from '$app/state';
</script>

<div class="text-white">
	<div>It was not possible to load the images this time.</div>
	<div>
		Reason: {page?.error?.message ?? 'No reason given.'}
	</div>
</div>
