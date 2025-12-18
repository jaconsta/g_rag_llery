import { error } from "@sveltejs/kit";

import { createChannel, createClient, Metadata } from "nice-grpc";
import { type GalleryViewClient, GalleryViewDefinition, type FilterGalleryRequest } from "../../proto/gallery_view";
import type { PageServerLoad } from "./$types";

interface Photo {
  src: string,
  caption: string,
  aspect: string,
  theme: string,
  alt: string,
}

export const load: PageServerLoad = async ({ cookies }) => {
  const userSession = cookies.get('session');
  if (!userSession) {
    console.warn("Should redirect to login or show some demo images");

    return {
      themes: [],
      aspects: [],
      photos: [],
      total: 0,
    }
  }

  let channel
  try {
    channel = createChannel(process.env["SERVER_GRPC_URL"]!);
  } catch (e) {
    console.error(e);
    return error(500, { message: "Failed to get gallery" });
  }

  let images;
  let filterItems;
  try {
    const client: GalleryViewClient = createClient(GalleryViewDefinition, channel);
    const filter: FilterGalleryRequest = {};
    // Note: Pending to add `Bearer` to the auth token.
    const options = { metadata: Metadata({ "x-authorization": `${userSession}` }) };
    images = await client.listGallery(filter, options);
    filterItems = await client.filterOptions(filter, options);
  } catch (e) {
    console.error(e);
    return error(500, { message: "Failed to get gallery from the server" });
  } finally {
    channel?.close();
  }

  const photos: Photo[] = images.images.map(p => ({
    src: p.imgUrl,
    caption: p.ariaText,
    aspect: p.aspect,
    theme: p.theme,
    alt: p.altText,
  }));

  return {
    themes: filterItems.themes,
    aspects: filterItems.aspects,
    photos,
    total: images.count,
  }
}

