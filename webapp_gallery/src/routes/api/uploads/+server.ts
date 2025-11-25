import type { RequestHandler } from '@sveltejs/kit';

import { createChannel, createClient } from "nice-grpc";
import { type GalleryViewClient, GalleryViewDefinition, type UploadImageRequest } from "../../../proto/gallery_view";


export const POST: RequestHandler = async ({ request }) => {
  let formData;
  try {
    formData = await request.formData();
  } catch (e) {
    console.error(e);
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  }

  const name = formData.get("name");
  const hash = formData.get("hash")?.toString();
  const size = parseInt(formData.get("size")?.toString() ?? "");
  // I need zod here
  if (isNaN(size)) {
    console.error("Received wrong size for file", size);
    return new Response(JSON.stringify({ success: false }), { status: 400, headers: { "Content-type": "application/json" } });
  } else if (typeof name != "string") {
    console.error("Provided a non string value", typeof name);
    return new Response(JSON.stringify({ success: false }), { status: 400, headers: { "Content-type": "application/json" } });
  } else if (!hash) {
    console.error("Provided an empty hash");
    return new Response(JSON.stringify({ success: false }), { status: 400, headers: { "Content-type": "application/json" } });
  }

  let channel
  try {
    channel = createChannel(process.env["SERVER_GRPC_URL"]!);
  } catch (e) {
    console.error(e);
    return new Response(JSON.stringify({ success: false }), { status: 400, headers: { "Content-type": "application/json" } });
  }

  let signedUrl;
  try {
    const client: GalleryViewClient = createClient(GalleryViewDefinition, channel);
    const uploadData: UploadImageRequest = { filehash: hash, filename: name, filesize: size };
    signedUrl = await client.uploadImage(uploadData);
  } catch (e) {
    console.error("Provided an unsuitable", e);
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  }

  return new Response(JSON.stringify({ success: true, url: signedUrl.bucketLink }), { status: 201, headers: { "Content-type": "application/json" } });
}
