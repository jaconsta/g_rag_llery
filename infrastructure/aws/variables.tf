variable "environment" {
  description = "Environment"
  default     = "Dev"
  # option -> production
}

variable "aws_access_key" {
  description = "AWS access key"
  type        = string
}
variable "aws_secret_key" {
  description = "AWS secret key"
  type        = string
}
variable "aws_region" {
  description = "AWS region for service deployment"
  default     = "eu-central-1"
}

variable "bucket_upload" {
  type        = string
  description = "Bucket for users uploded content"
  default     = "g-rag-upload"
}
variable "bucket_processed" {
  type        = string
  description = "Bucket for content processed by the rag"
  default     = "g-rag-processed"
}

variable "localstack_s3_endpoint" {
  type        = string
  description = "Localstack endpoint for S3 services"
  default     = "http://s3.localhost.localstack.cloud:4566"
}
variable "localstack_url_endpoint" {
  type        = string
  description = "Localstack endpoint for AWS services"
  default     = "http://localhost:4566"
}

