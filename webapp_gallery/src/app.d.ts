// See https://svelte.dev/docs/kit/types#app.d.ts

import type { PrismaClient } from "@prisma/client";

// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }
  var prisma: PrismaClient
}

export { };
