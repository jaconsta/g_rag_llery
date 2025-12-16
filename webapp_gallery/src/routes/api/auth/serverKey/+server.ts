import { type RequestHandler, json } from "@sveltejs/kit";

import { createChannel, createClient } from "nice-grpc";
import { type AuthGreeterClient, AuthGreeterDefinition, EmptyRequest } from "../../../../proto/user_auth";

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

