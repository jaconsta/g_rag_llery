import * as Minio from "minio";

export function minioClient(): Minio.Client {

  const endPoint = process.env["MINIO_BUCKET_URL"];
  if (!endPoint) {
    console.error("MINIO_BUCKET_URL is missing");
    throw new Error("Failed initialization");
  }
  const accessKey = process.env["MINIO_ACCESS_KEY"];
  const secretKey = process.env["MINIO_SECRET_KEY"];
  const useSSL = process.env["MINIO_CHECK_SSL"] == "true";
  return new Minio.Client({
    endPoint: "localhost",
    port: 9000,
    accessKey,
    secretKey,
    useSSL
  });
}

// The link expires in 7 days
export async function getPresignedUrl(mClient: Minio.Client, docPath: string): Promise<string> {
  const bucket = process.env["BUCKET_THUMBNAIL_NAME"];
  if (!bucket) {
    console.error("BUCKET_THUMBNAIL_NAME is missing");
    throw new Error("Missing env var");
  }

  return await mClient.presignedGetObject(bucket, docPath, 1000);
}

// The link expires in 7 days
export async function getPresignedPostUrl(mClient: Minio.Client, name: string): Promise<string> {
  const bucket = process.env["BUCKET_FEEDER_NAME"];
  if (!bucket) {
    console.error("BUCKET_FEEDER_NAME is missing");
    throw new Error("Missing env var");
  }

  return await mClient.presignedPutObject(bucket, `feeder/${name}`, 1000);
}
