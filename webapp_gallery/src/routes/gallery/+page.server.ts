import { fail } from "@sveltejs/kit";

import { createChannel, createClient } from "nice-grpc";
import { type GalleryViewClient, GalleryViewDefinition, type FilterGalleryRequest } from "../../proto/gallery_view";

interface Photo {
  src: string,
  caption: string,
  aspect: string,
  theme: string,
  alt: string,
}

export const load = async () => {
  let channel
  try {
    channel = createChannel(process.env["SERVER_GRPC_URL"]!);
  } catch (e) {
    console.error(e);
    return fail(500, { message: "Failed to get gallery" });
  }

  let images;
  let filterItems;
  try {
    const client: GalleryViewClient = createClient(GalleryViewDefinition, channel);
    const filter: FilterGalleryRequest = {};
    images = await client.listGallery(filter);
    filterItems = await client.filterOptions(filter);
  } catch (e) {
    console.error(e);
    return fail(500, { message: "Failed to get gallery" });
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

