<script lang="ts">
	import { resolve } from '$app/paths';
	let isCodeCopied = $state(false);
	let buttonClicked = $state(false);
	const buttonColor = $derived(
		buttonClicked
			? 'bg-teal-500 hover:bg-teal-400 focus-visible:outline-teal-500'
			: 'bg-indigo-500 hover:bg-indigo-400 focus-visible:outline-indigo-500'
	);
	const lenghtyKey = makeid(24);

	function makeid(length: number) {
		var result = '';
		var characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
		var charactersLength = characters.length;
		for (var i = 0; i < length; i++) {
			result += characters.charAt(Math.floor(Math.random() * charactersLength));
		}
		return result;
	}

	async function copyToClipboard(textToCopy: string) {
		try {
			await navigator.clipboard.writeText(textToCopy);
			buttonClicked = true;
		} catch (error) {
			console.log('Failed to copy to clipboard:', error);
		}
	}
</script>

<div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
	<div class="sm:mx-auto sm:w-full sm:max-w-sm">
		<img
			src="https://tailwindcss.com/plus-assets/img/logos/mark.svg?color=indigo&shade=500"
			alt="gallerag"
			class="mx-auto h-10 w-auto"
		/>
		<h2 class="mt-10 text-center text-2xl/9 font-bold tracking-tight text-white">New account</h2>
	</div>

	<div class="mt-10 text-gray-700 sm:mx-auto sm:w-full sm:max-w-sm dark:text-gray-400">
		<p>This code serves as your username and password</p>
		<p>
			We don't store it, if you loose it, you must create a new code. Which means, a new account.
		</p>
		<p>
			We recommend you don't use any words or secret messages. This helps with the security of your
			account.
		</p>
	</div>

	<div class="mt-10 text-gray-700 sm:mx-auto sm:w-full sm:max-w-sm dark:text-gray-400">
		<h3>Key</h3>
		<p class="size-8 font-semibold tracking-wide font-stretch-ultra-expanded">{lenghtyKey}</p>
		<div class="flex w-full flex-row-reverse">
			<button
				type="button"
				onclick={() => copyToClipboard(lenghtyKey)}
				class={buttonColor +
					'rounded-md px-3 py-1.5 text-sm/6 font-semibold text-white focus-visible:outline-2 focus-visible:outline-offset-2 '}
				>{buttonClicked ? '✅ ' : ''}Copy</button
			>
		</div>

		<label>
			<input type="checkbox" bind:checked={isCodeCopied} />
			I backed up the code.
		</label>
		<div>
			{#if isCodeCopied}
				<a
					href={resolve('/auth')}
					class="flex w-full justify-center rounded-md px-3 py-1.5 text-sm/6 font-semibold text-white {isCodeCopied
						? 'bg-indigo-500  hover:bg-indigo-400 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500'
						: 'disabled bg-gray-500'}">Sign in</a
				>
			{:else}
				<span
					class="disabled flex w-full justify-center rounded-md bg-gray-500 px-3 py-1.5 text-sm/6 font-semibold text-white"
					>Sign in</span
				>
			{/if}
		</div>
	</div>
</div>
