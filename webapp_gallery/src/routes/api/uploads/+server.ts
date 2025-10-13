import { getPresignedPostUrl, minioClient } from '$lib/server/minio';
import type { RequestHandler } from '@sveltejs/kit';

export const POST: RequestHandler = async ({ request }) => {
  let formData;
  try {
    formData = await request.formData(); // this can throw
  } catch (e) {
    console.error(e);
    return new Response(JSON.stringify({ success: false }), { status: 500, headers: { "Content-type": "application/json" } });
  }

  const name = formData.get("name");
  if (typeof name != "string") {
    console.error("Provided a non string value", typeof name);
    return new Response(JSON.stringify({ success: false }), { status: 501, headers: { "Content-type": "application/json" } });
  }

  const m = minioClient()
  const presignedUrl = await getPresignedPostUrl(m, name as string);
  return new Response(JSON.stringify({ success: true, url: presignedUrl }), { status: 201, headers: { "Content-type": "application/json" } });
}
