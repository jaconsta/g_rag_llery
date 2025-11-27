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


  return new Response(JSON.stringify({ success: true, key: publicKey.publicKey }), { status: 200, headers: { "Content-type": "application/json" } });
}

export const POST: RequestHandler = async ({ request }) => {
  const body: EncryptedMsg = await request.json();


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
    const noData: UserPublicAuth = {
      ephemeralPublicKey: body.publicKey,
      nonce: body.nonce,
      message: body.ciphertext
    };
    publicKey = await client.exchangeAuth(noData);
  } catch (e) {
    console.error("Provided an unsuitable", e);
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  } finally {
    channel.close();
  }

  console.log(publicKey.status)
  return json({ success: true }, { status: 200 });
}

