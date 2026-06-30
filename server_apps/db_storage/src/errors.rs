use thiserror::Error;

pub type QueryResult<T> = Result<T, QueryError>;
// pub type StorageResult<T> = Result<T, BucketError>;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Failed to connect")]
    Connection,
    #[error("Failed to create extension")]
    Extensions,
    #[error("Failed to create table")]
    Migration,
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Failed to run query")]
    Query,
    #[error("Not an error")]
    NotAnError,
}

impl From<sqlx::Error> for QueryError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => QueryError::NotAnError,
            err => {
                log::error!("QueryError: {err:?}");
                QueryError::Query
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum BucketError {
    #[error("asString filesystem error")]
    Str,
    #[error("The filepath is wrong")]
    MalformedPath,
    #[error("io filesystem error")]
    Filesystem,
}

impl From<String> for BucketError {
    fn from(value: String) -> Self {
        log::error!("Failed to run local filesystem operation: {:?}", value);
        BucketError::Str
    }
}

impl From<std::io::Error> for BucketError {
    fn from(value: std::io::Error) -> Self {
        log::error!("std::io::Error: {:?}", value);
        BucketError::Filesystem
    }
}
