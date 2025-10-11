use thiserror::Error;

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
