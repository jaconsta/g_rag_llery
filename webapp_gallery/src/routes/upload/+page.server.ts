import type { Actions, } from "@sveltejs/kit";
import { fail } from "@sveltejs/kit";
import { prisma } from "../../lib/server/prisma/prisma"

export const actions: Actions = {
  createImage: async ({ request }) => {
    // Taken from sample. modify
    const { title, content } = Object.fromEntries(await request.formData()) as { title: string, content: string };

    try {
      await prisma.gallery.create({
        data: {
          path: title,
          thumbnail_path: content,
        }
      });
    } catch (e: unknown) {
      console.error(e);
      return fail(500, { message: "Failed to insert gallery" });
    }

    return {
      status: 200
    };
  },
};

