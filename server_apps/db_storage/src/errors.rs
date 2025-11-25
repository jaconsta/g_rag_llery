use thiserror::Error;

pub type QueryResult<T> = Result<T, QueryError>;

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
}

impl From<sqlx::Error> for QueryError {
    fn from(value: sqlx::Error) -> Self {
        log::error!("Failed to run query, {:?}", value);
        QueryError::Query
    }
}
