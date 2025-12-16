<script lang="ts">
	import sodium from 'libsodium-wrappers';
	import type { ActionResult } from '@sveltejs/kit';
	import { resolve } from '$app/paths';
	import { applyAction, deserialize } from '$app/forms';
	import { invalidateAll, goto } from '$app/navigation';

	let shouldHidePassword = $state(true);

	// Cast to force the linter
	interface Target {
		value: string;
	}
	interface EncryptedMsg {
		ciphertext: string;
		nonce: string;
		publicKey: string;
	}

	function getSecretCode(e: SubmitEvent) {
		if (!e.target) {
			return;
		}
		const target = e.target as unknown as Target[];
		const secretCode = target[0].value;

		return secretCode;
	}

	function encryptMessage(recipientPublicKey: string, message: string): EncryptedMsg {
		const messageBytes = sodium.from_string(message);
		const recipientPublicKeyBytes = sodium.from_hex(recipientPublicKey);

		// Take the role of Alice
		// Generate ephmeral keypair.
		const ephimeralKeypair = sodium.crypto_box_keypair();
		const { privateKey, publicKey } = ephimeralKeypair;
		// Generate unique Nonce
		const nonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);

		// Encrypt the message
		const ciphertext = sodium.crypto_box_easy(
			messageBytes,
			nonce,
			recipientPublicKeyBytes,
			privateKey
		);

		return {
			ciphertext: sodium.to_hex(ciphertext),
			nonce: sodium.to_hex(nonce),
			publicKey: sodium.to_hex(publicKey)
		};
	}

	interface ServerPublicKey {
		success: boolean;
		key: string;
	}

	// Queries the server for the public key
	async function getAuthPublicKey(): Promise<string> {
		const response = await fetch('/api/auth/serverKey');
		const body: ServerPublicKey = await response.json();

		return body.key;
	}

	// Sends the user auth cipher to get the session jwt
	async function sendUserSecret(data: EncryptedMsg, url: string): Promise<ActionResult> {
		const formData = new FormData();
		formData.append('ciphertext', data.ciphertext);
		formData.append('nonce', data.nonce);
		formData.append('publicKey', data.publicKey);
		const response = await fetch(url, { method: 'POST', body: formData });

		const result: ActionResult = deserialize(await response.text());

		return result;
	}

	async function handleSubmit(e: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }) {
		e.preventDefault();
		const actionUrl = e.currentTarget.action;
		const userCode = getSecretCode(e);
		if (!userCode) return;

		let serverPublicKey: string;
		try {
			serverPublicKey = await getAuthPublicKey();
		} catch (e) {
			console.error('serverPublicKey failed to get.');
			console.error(e);
			return;
		}

		const cipherBody = encryptMessage(serverPublicKey, userCode);

		try {
			const tokenResult = await sendUserSecret(cipherBody, actionUrl);
			if (tokenResult.type === 'success') {
				invalidateAll();
			} else if (tokenResult.type === 'redirect') {
				// "Hardcoding" the redirect path to make "resolve" happy
				goto(resolve('/'));
			}

			applyAction(tokenResult);
		} catch (e) {
			console.error(e);
			return;
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
		<h2 class="mt-10 text-center text-2xl/9 font-bold tracking-tight text-white">
			Sign in to your account
		</h2>
	</div>

	<div class="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
		<form onsubmit={handleSubmit} method="POST" class="space-y-6">
			<div>
				<label for="keyphrase" class="block text-sm/6 font-medium text-gray-100">Secret Code</label>
				<div class="mt-2 flex flex-row">
					<input
						id="keyphrase"
						type={shouldHidePassword ? 'password' : 'text'}
						name="keyphrase"
						required
						minlength="24"
						maxlength="100"
						class="w-full rounded-md bg-white/5 px-3 py-1.5 text-base text-white outline-1 -outline-offset-1 outline-white/10 placeholder:text-gray-500 focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-500 sm:text-sm/6"
						spellcheck="false"
					/>
					<button
						type="button"
						onclick={() => {
							shouldHidePassword = !shouldHidePassword;
						}}
						aria-label="toggle password visibility"
						class="justify-items-center rounded-md bg-white/5 ps-2 pe-2 outline-1 -outline-offset-1 outline-white/10"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							fill="#bbb"
							stroke="#bbb"
							class=" h-[18px] cursor-pointer"
							viewBox="0 0 128 128"
						>
							<path
								d="M64 104C22.127 104 1.367 67.496.504 65.943a4 4 0 0 1 0-3.887C1.367 60.504 22.127 24 64 24s62.633 36.504 63.496 38.057a4 4 0 0 1 0 3.887C126.633 67.496 105.873 104 64 104zM8.707 63.994C13.465 71.205 32.146 96 64 96c31.955 0 50.553-24.775 55.293-31.994C114.535 56.795 95.854 32 64 32 32.045 32 13.447 56.775 8.707 63.994zM64 88c-13.234 0-24-10.766-24-24s10.766-24 24-24 24 10.766 24 24-10.766 24-24 24zm0-40c-8.822 0-16 7.178-16 16s7.178 16 16 16 16-7.178 16-16-7.178-16-16-16z"
								data-original="#000000"
							></path>
						</svg>
					</button>
				</div>
			</div>

			<div class="flex columns-2">
				<span class="w-full content-center font-medium text-gray-700 dark:text-gray-400"
					>New account?</span
				>
				<a
					href={resolve('/auth/generate')}
					class="w-full text-right font-semibold text-indigo-400 hover:text-indigo-300"
					>Generate new</a
				>
			</div>
			<div>
				<button
					type="submit"
					class="flex w-full justify-center rounded-md bg-indigo-500 px-3 py-1.5 text-sm/6 font-semibold text-white hover:bg-indigo-400 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500"
					>Sign in</button
				>
			</div>
		</form>
	</div>
</div>
