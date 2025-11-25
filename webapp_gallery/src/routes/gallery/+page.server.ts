import { getPresignedUrl, minioClient } from "$lib/server/minio"
import { fail } from "@sveltejs/kit";
import { prisma } from "../../lib/server/prisma/prisma"

import { createChannel, createClient } from "nice-grpc";
import { type GalleryViewClient, GalleryViewDefinition, type FilterGalleryRequest } from "../../proto/gallery_view";

interface Photo {
  src: string,
  caption: string,
  aspect: string,
  theme: string,
  alt: string,
}

export const load/*: PageServerLoad*/ = async () => {
  const channel = createChannel("localhost:50051");
  const client: GalleryViewClient = createClient(GalleryViewDefinition, channel);
  const filter: FilterGalleryRequest = {

  };
  const res = await client.listGallery(filter);
  console.log("grpgrpgrpgrpgrpcccccPres");
  console.log(res);
  channel.close();

  const mClient = minioClient();
  let imageData;
  try {
    imageData = await prisma.gallery.findMany({ include: { gallery_rag_embeddings: true }, take: 20, skip: 0 });
  } catch (e) {
    console.error(e);
    return fail(500, { message: "Failed to get gallery" });
  }
  const photos: Photo[] = [];
  const aspects: string[] = []
  const themes: string[] = []

  for (const i of imageData) {
    if (!i.thumbnail_path) { continue; }
    try {
      const link = await getPresignedUrl(mClient, i.thumbnail_path)
      const aspect = i.thumbnail_ratio ?? "landscape"
      const theme = i.gallery_rag_embeddings?.theme ?? "Unthemed";
      photos.push({
        src: link,
        caption: i.gallery_rag_embeddings?.img_aria ?? "Pending your input",
        aspect,
        theme,
        alt: i.gallery_rag_embeddings?.img_alt ?? "Image uploaded by user",
      })
      aspects.push(aspect);
      themes.push(theme);

    } catch (e) {
      console.error(e);
    }
  }

  return {
    imageData,
    themes,
    aspects: [...new Set(aspects)],
    photos
  }
}

