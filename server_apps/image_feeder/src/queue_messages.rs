use percent_encoding_rfc3986::{percent_decode, percent_decode_str};
use serde::{Deserialize, Serialize};
use std::convert::From;

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinioKakfaEvent {
    #[serde(rename = "EventName")]
    pub event_name: String,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Records")]
    pub records: Vec<Record>,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub event_version: String,
    pub event_source: String,
    pub aws_region: String,
    pub event_time: String,
    pub event_name: String,
    pub user_identity: UserIdentity,
    pub request_parameters: RequestParameters,
    pub response_elements: ResponseElements,
    pub s3: S3,
    pub source: Source,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIdentity {
    pub principal_id: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestParameters {
    pub principal_id: String,
    pub region: String,
    #[serde(rename = "sourceIPAddress")]
    pub source_ipaddress: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseElements {
    #[serde(rename = "x-amz-id-2")]
    pub x_amz_id_2: String,
    #[serde(rename = "x-amz-request-id")]
    pub x_amz_request_id: String,
    #[serde(rename = "x-minio-deployment-id")]
    pub x_minio_deployment_id: String,
    #[serde(rename = "x-minio-origin-endpoint")]
    pub x_minio_origin_endpoint: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3 {
    #[serde(rename = "s3SchemaVersion")]
    pub s3schema_version: String,
    pub configuration_id: String,
    pub bucket: Bucket,
    pub object: Object,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub name: String,
    pub owner_identity: OwnerIdentity,
    pub arn: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerIdentity {
    pub principal_id: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    pub key: String,
    pub size: i64,
    pub e_tag: String,
    pub content_type: String,
    pub user_metadata: UserMetadata,
    pub sequencer: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMetadata {
    #[serde(rename = "content-type")]
    pub content_type: String,
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub host: String,
    pub port: String,
    pub user_agent: String,
}

#[derive(Debug, Clone)]
pub struct ImageFeed {
    pub filename: String,
    pub content_type: String,
    pub bucket: String,
}

impl From<Record> for ImageFeed {
    fn from(item: Record) -> Self {
        ImageFeed {
            filename: item.s3.object.key,
            content_type: item.s3.object.content_type,
            bucket: item.s3.bucket.name,
        }
    }
}
impl From<&Record> for ImageFeed {
    fn from(item: &Record) -> Self {
        let orig_filename = item.s3.object.key.clone();
        let pct_filename: String = match percent_decode(&orig_filename.as_bytes()) {
            Ok(p) => p.decode_utf8_lossy().to_string().replace("+", " "),
            Err(_) => orig_filename,
        };
        ImageFeed {
            filename: pct_filename,
            content_type: item.s3.object.content_type.clone(),
            bucket: item.s3.bucket.name.clone(),
        }
    }
}
