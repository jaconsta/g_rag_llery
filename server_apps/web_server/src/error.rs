use crypto_box::aead::Error as CryptoError;
use db_storage::QueryError;
use hex::FromHexError;
// use std::error::Error;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;
// This is ths one that shoul work
// pub type Result<T> = core::result::Result<T, Errors>;

#[derive(Debug)]
pub enum Error {
    Custom(String),
    InvalidLength { expected: usize, received: usize },
    CryptoError(CryptoError),
    AuthError,
    DbError,
    Duplicated,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength { expected, received } => write!(
                f,
                "Invalid key length. Expected {} bytes, received {}.",
                expected, received
            ),
            Error::CryptoError(e) => write!(f, "Error during Crypto operation: {}.", e),
            Error::AuthError => write!(f, "Error during user auth process."),
            Error::DbError => write!(f, "Error during db operation."),
            Error::Duplicated => write!(f, "Error the requested information is duplicated."),
            Error::Custom(e) => write!(f, "Error {}.", e),
        }
    }
}

impl Error {
    // pub fn custom(val: impl std::fmt::Display) -> Self {
    //     Self::Custom(val.to_string())
    // }
}

impl std::error::Error for Error {}

impl From<hex::FromHexError> for Error {
    fn from(err: FromHexError) -> Self {
        println!("Error From hex: {}", err);
        Error::AuthError
    }
}
impl From<CryptoError> for Error {
    fn from(err: CryptoError) -> Self {
        Error::CryptoError(err)
    }
}
impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Custom(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Custom(err)
    }
}

impl From<QueryError> for Error {
    fn from(err: QueryError) -> Self {
        log::error!("{err:?}");
        Error::DbError
    }
}
