import { getPresignedPostUrl, minioClient } from '$lib/server/minio';
import type { RequestHandler } from '@sveltejs/kit';
import { prisma } from "../../../lib/server/prisma/prisma"

import { v4 as uuidv4 } from 'uuid';


export const POST: RequestHandler = async ({ request }) => {
  let formData;
  try {
    formData = await request.formData(); // this can throw
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

  try {
    const isDuplicate = await prisma.user_upload.count({ where: { filename: name, filehash: hash } });
    if (isDuplicate) {
      return new Response(JSON.stringify({ success: false, duplicate: true }), { status: 406, headers: { "Content-type": "application/json" } });
    }
  } catch (e) {
    console.error("Failed to fetch duplicate for ", { filename: name, filehash: hash }, e);
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  }
  try {
    // Then insert
    await prisma.user_upload.create({ data: { filename: `feeder/${name}`, filehash: hash, filesize: size, user_id: uuidv4() } });
  } catch (e) {
    console.error("Provided an unsuitable", e);
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  }


  const m = minioClient()
  const presignedUrl = await getPresignedPostUrl(m, name as string);
  return new Response(JSON.stringify({ success: true, url: presignedUrl }), { status: 201, headers: { "Content-type": "application/json" } });
}
