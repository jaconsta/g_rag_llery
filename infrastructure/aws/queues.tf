resource "aws_sqs_queue" "user_upload_queue" {
  name = "rag-user-upload-queue"

  tags = {
    Environment = var.environment
  }

  policy = data.aws_iam_policy_document.user_upload_queue.json
}

resource "aws_s3_bucket_notification" "user_upload_queue" {
  bucket = aws_s3_bucket.rag_upload.id

  queue {
    queue_arn = aws_sqs_queue.user_upload_queue.arn
    events    = ["s3:ObjectCreated:*"]
  }
}
