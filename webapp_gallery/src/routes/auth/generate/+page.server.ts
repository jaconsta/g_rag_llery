import type { PageServerLoad } from "../$types";
import { redirect } from "@sveltejs/kit";

export const load: PageServerLoad = async ({ cookies }) => {
  // Make it validate that the token in cookies is also still valid
  if (cookies.get('session')) {
    return redirect(302, '/')
  }
}
