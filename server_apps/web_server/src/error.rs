use crypto_box::aead::Error as CryptoError;
use hex::FromHexError;
use std::error::Error;

pub type ResultR<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum Errors {
    WithMessage(String),
    InvalidLength { expected: usize, received: usize },
    CryptoError(CryptoError),
    AuthError,
}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::InvalidLength { expected, received } => write!(
                f,
                "Invalid key length. Expected {} bytes, received {}.",
                expected, received
            ),
            Errors::CryptoError(e) => write!(f, "Error during Crypto operation: {}.", e),
            Errors::AuthError => write!(f, "Error durung user auth process"),
            Errors::WithMessage(e) => write!(f, "Error {}", e),
        }
    }
}

impl Error for Errors {}

impl From<hex::FromHexError> for Errors {
    fn from(err: FromHexError) -> Self {
        println!("Error From hex: {}", err);
        Errors::AuthError
    }
}
impl From<CryptoError> for Errors {
    fn from(err: CryptoError) -> Self {
        Errors::CryptoError(err)
    }
}
impl From<String> for Errors {
    fn from(err: String) -> Self {
        Errors::WithMessage(err)
    }
}
