resource "aws_s3_bucket" "rag_upload" {
  bucket = var.bucket_upload

  tags = {
    Name        = "Rag upload"
    Environment = var.environment
  }
}

# Bucket notification
data "aws_iam_policy_document" "user_upload_queue" {
  statement {
    effect = "Allow"

    principals {
      type        = "*"
      identifiers = ["*"]
    }

    actions   = ["sqs:SendMessage"]
    resources = ["arn:aws:sqs:*:*:rag-user-upload-queue"]

    condition {
      test     = "ArnEquals"
      variable = "aws:SourceArn"
      values   = [aws_s3_bucket.rag_upload.arn]
    }
  }
}

resource "aws_s3_bucket" "rag_processed" {
  bucket = var.bucket_processed

  tags = {
    Name        = "Rag upload"
    Environment = var.environment
  }
}


resource "aws_s3_bucket_ownership_controls" "rag_processed" {
  bucket = aws_s3_bucket.rag_processed.id
  rule {
    object_ownership = "BucketOwnerPreferred"
  }
}

resource "aws_s3_bucket_acl" "rag_processed" {
  depends_on = [aws_s3_bucket_ownership_controls.rag_processed]

  bucket = aws_s3_bucket.rag_processed.id
  acl    = "private"
}
