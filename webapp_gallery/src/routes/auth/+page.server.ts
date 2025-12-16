import type { Actions } from "@sveltejs/kit";
import { fail } from "@sveltejs/kit";
import { json, redirect } from "@sveltejs/kit";

import { createChannel, createClient } from "nice-grpc";
import { type AuthGreeterClient, AuthGreeterDefinition, UserAuthResponse, UserPublicAuth } from "../../proto/user_auth";
import type { PageServerLoad } from "./$types";

interface EncryptedMsg {
  ciphertext: string;
  nonce: string;
  publicKey: string;
}

async function authExchange(body: EncryptedMsg): Promise<UserAuthResponse> {
  let channel;
  try {
    channel = createChannel(process.env["SERVER_GRPC_URL"]!);
  } catch (e) {
    console.error(e);
    throw new Error("Create channel failed");
  }

  let publicKey;
  try {
    const client: AuthGreeterClient = createClient(AuthGreeterDefinition, channel);
    const authData: UserPublicAuth = {
      ephemeralPublicKey: body.publicKey,
      nonce: body.nonce,
      message: body.ciphertext
    };
    publicKey = await client.exchangeAuth(authData);
  } catch (e) {
    console.error("Provided an unsuitable", e);
    throw new Error("Create channel failed");
  } finally {
    channel.close();
  }

  return publicKey
}

export const load: PageServerLoad = async ({ cookies }) => {
  // Make it validate that the token in cookies is also still valid
  if (cookies.get('session')) {
    return redirect(302, '/')
  }
}


export const actions = {
  default: async ({ cookies, request }) => {
    const data = await request.formData();
    if (!data.get('ciphertext') || !data.get('nonce') || !data.get('publicKey')) {
      return fail(400, { success: false })
    }
    const body: EncryptedMsg = { ciphertext: data.get('ciphertext')! as string, nonce: data.get('nonce')! as string, publicKey: data.get('publicKey')! as string };

    let publicKey
    try {
      publicKey = await authExchange(body);
    } catch {
      return json({ success: false }, { status: 400 });
    }

    // Make the rpc bearer attribute non-optional
    if (publicKey.status === "OK" && publicKey.bearer) {
      cookies.set('session', publicKey.bearer, {
        // send cookie for every page
        path: '/',
        // server side only cookie so you can't use `document.cookie`
        httpOnly: true,
        // only requests from same site can send cookies
        // https://developer.mozilla.org/en-US/docs/Glossary/CSRF
        sameSite: 'strict',
        // only sent over HTTPS in production
        secure: process.env.NODE_ENV === 'production',
        // set cookie to expire with the bearer token.
        // Residual calculation for a month: 60 * 60 * 24 * 30
        maxAge: publicKey.expires || 0,
      });

      // It should redirect the user but right now, this is an api endpoint.
      return redirect(302, '/')
    }
    return json({ success: false }, { status: 200 });
  },
} satisfies Actions;
