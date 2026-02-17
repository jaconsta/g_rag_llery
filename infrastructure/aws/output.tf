output "user_upload_queue_arn" {
  value = aws_sqs_queue.user_upload_queue.arn
}

output "bucket_rag_upload_id" {
  value = aws_s3_bucket.rag_upload.id
}

output "bucket_rag_processed_id" {
  value = aws_s3_bucket.rag_processed.id
}
