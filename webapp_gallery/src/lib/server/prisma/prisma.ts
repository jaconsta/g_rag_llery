import { PrismaClient } from "@prisma/client";

const prisma = global.prisma || new PrismaClient();

if (process.env.NODE_END === "development") {
  global.prisma = prisma
}

export { prisma };
