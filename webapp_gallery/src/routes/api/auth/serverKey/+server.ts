import { type RequestHandler, json } from "@sveltejs/kit";

import { createChannel, createClient } from "nice-grpc";
import { type AuthGreeterClient, AuthGreeterDefinition, EmptyRequest, UserPublicAuth } from "../../../../proto/user_auth";

interface EncryptedMsg {
  ciphertext: string;
  nonce: string;
  publicKey: string;
}
export const GET: RequestHandler = async () => {
  let channel;
  try {
    channel = createChannel(process.env["SERVER_GRPC_URL"]!);
  } catch (e) {
    console.error(e);
    return new Response(JSON.stringify({ success: false }), { status: 400, headers: { "Content-type": "application/json" } });
  }

  let publicKey;
  try {
    const client: AuthGreeterClient = createClient(AuthGreeterDefinition, channel);
    const noData: EmptyRequest = {};
    publicKey = await client.greetAuth(noData);
  } catch (e) {
    console.error("Provided an unsuitable", e);
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  } finally {
    channel.close();
  }

  return json({ success: true, key: publicKey.publicKey }, { status: 200 });
}

export const POST: RequestHandler = async ({ cookies, request }) => {
  const body: EncryptedMsg = await request.json();

  let channel;
  try {
    channel = createChannel(process.env["SERVER_GRPC_URL"]!);
  } catch (e) {
    console.error(e);
    return json({ success: false }, { status: 400 });
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
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  } finally {
    channel.close();
  }

  // Make the rpc bearer attribute non-optional
  if (publicKey.status === "Ok" && publicKey.bearer) {
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
    // redirect(302, '/')
  }
  return json({ success: true }, { status: 200 });
}

