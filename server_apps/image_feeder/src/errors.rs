use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageProcessError {
    // #[error("Get image from S3.")]
    // S3ImageFetch,
    // #[error("Convert S3 image into bytes.")]
    // S3ImageIntoBytes,
    #[error("Failed to load image")]
    ImageLoad,
    #[error("Failed to detect format.")]
    ImageGuessFormat,
}

#[derive(Error, Debug)]
pub enum BucketOperationsError {
    #[error("Failed to upload blob.")]
    BlobUpload,
    #[error("Failed to download blob.")]
    BlobDownload,
    #[error("Failed to read/parse Stream.")]
    BlobRead,
}

#[derive(Error, Debug)]
pub enum KafkaConnectionError {
    #[error("Failed to Subscribe to topics.")]
    Subscribe,
    #[error("Failed to Read next message.")]
    RecvMessage,
    #[error("Failed to Send payload to mpsc.")]
    MpscSendMessage,
}

#[derive(Error, Debug)]
pub enum LlmRetrievalError {
    #[error("Failed to fetch from OpenAI.")]
    OpenAi,
    #[error("Failed to fetch from Ollama.")]
    Ollama,
    #[error("Query returned no results.")]
    NoContent,
    #[error("Failed to setup custom retrieval model.")]
    MultimodalSetup,
}
