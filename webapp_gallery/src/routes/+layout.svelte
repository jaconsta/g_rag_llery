<script lang="ts">
	import '../app.css';
	import { resolve } from '$app/paths';
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

<header class="absolute inset-x-0 top-0 z-50">
	<nav aria-label="Global" class="flex items-center justify-between p-6 lg:px-8">
		<div class="flex lg:flex-1">
			<a href={resolve('/')} class="-m-1.5 p-1.5">
				<span class="sr-only">Clord</span>
				<img
					src="https://tailwindcss.com/plus-assets/img/logos/mark.svg?color=indigo&shade=500"
					alt=""
					class="h-8 w-auto"
				/>
			</a>
		</div>
		<div class="flex lg:hidden">
			<button
				type="button"
				command="show-modal"
				commandfor="mobile-menu"
				class="-m-2.5 inline-flex items-center justify-center rounded-md p-2.5 text-gray-200"
			>
				<span class="sr-only">Open main menu</span>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					data-slot="icon"
					aria-hidden="true"
					class="size-6"
				>
					<path
						d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			</button>
		</div>
		<div class="hidden lg:flex lg:gap-x-12">
			<a href={resolve('/gallery')} class="text-sm/6 font-semibold text-white">Gallery</a>
			<a href={resolve('/upload')} class="text-sm/6 font-semibold text-white">Upload</a>
			<a href="#" class="text-sm/6 font-semibold text-white">Marketplace</a>
			<a href="#" class="text-sm/6 font-semibold text-white">Company</a>
		</div>
		<div class="hidden lg:flex lg:flex-1 lg:justify-end">
			<a href={resolve('/auth')} class="text-sm/6 font-semibold text-white"
				>Log in <span aria-hidden="true">&rarr;</span></a
			>
			<a href={resolve('/auth/logout')} class="text-sm/6 font-semibold text-white"
				>Log out <span aria-hidden="true">&rarr;</span></a
			>
		</div>
	</nav>
	<el-dialog>
		<dialog id="mobile-menu" class="backdrop:bg-transparent lg:hidden">
			<div tabindex="0" class="fixed inset-0 focus:outline-none">
				<el-dialog-panel
					class="fixed inset-y-0 right-0 z-50 w-full overflow-y-auto bg-gray-900 p-6 sm:max-w-sm sm:ring-1 sm:ring-gray-100/10"
				>
					<div class="flex items-center justify-between">
						<a href={resolve('/')} class="-m-1.5 p-1.5">
							<span class="sr-only">Clord</span>
							<img
								src="https://tailwindcss.com/plus-assets/img/logos/mark.svg?color=indigo&shade=500"
								alt=""
								class="h-8 w-auto"
							/>
						</a>
						<button
							type="button"
							command="close"
							commandfor="mobile-menu"
							class="-m-2.5 rounded-md p-2.5 text-gray-200"
						>
							<span class="sr-only">Close menu</span>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.5"
								data-slot="icon"
								aria-hidden="true"
								class="size-6"
							>
								<path d="M6 18 18 6M6 6l12 12" stroke-linecap="round" stroke-linejoin="round" />
							</svg>
						</button>
					</div>
					<div class="mt-6 flow-root">
						<div class="-my-6 divide-y divide-white/10">
							<div class="space-y-2 py-6">
								<a
									href={resolve('/gallery')}
									class="-mx-3 block rounded-lg px-3 py-2 text-base/7 font-semibold text-white hover:bg-white/5"
									>Gallery</a
								>
								<a
									href={resolve('/upload')}
									class="-mx-3 block rounded-lg px-3 py-2 text-base/7 font-semibold text-white hover:bg-white/5"
									>Upload</a
								>
								<a
									href="#"
									class="-mx-3 block rounded-lg px-3 py-2 text-base/7 font-semibold text-white hover:bg-white/5"
									>Marketplace</a
								>
								<a
									href="#"
									class="-mx-3 block rounded-lg px-3 py-2 text-base/7 font-semibold text-white hover:bg-white/5"
									>Company</a
								>
							</div>
							<div class="py-6">
								<a
									href={resolve('/auth')}
									class="-mx-3 block rounded-lg px-3 py-2.5 text-base/7 font-semibold text-white hover:bg-white/5"
									>Log in</a
								>
								<a
									href={resolve('/auth/logout')}
									class="-mx-3 block rounded-lg px-3 py-2.5 text-base/7 font-semibold text-white hover:bg-white/5"
									>Log out</a
								>
							</div>
						</div>
					</div>
				</el-dialog-panel>
			</div>
		</dialog>
	</el-dialog>
</header>
<div class="mt-20">
	{@render children?.()}
</div>
