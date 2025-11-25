use std::array::TryFromSliceError;

use crypto_box::{
    ChaChaBox, PublicKey, SecretKey,
    aead::{Aead, OsRng},
};
use hex;
use tonic::{Request, Response, Status};

pub use user_auth_rpc::auth_greeter_server::{AuthGreeter, AuthGreeterServer};
use user_auth_rpc::{EmptyRequest, ServerPublicKeys, UserAuthResponse, UserPublicAuth};

use crate::error::{Error, Result};

pub mod user_auth_rpc {
    tonic::include_proto!("user_auth"); // The string specified here must match the proto package name
}

#[derive(Debug)]
pub struct UserAuthGreeter {
    secret_key: SecretKey,
}

const KEY_LEN: usize = 32;

impl Default for UserAuthGreeter {
    fn default() -> Self {
        let bob_secret = SecretKey::generate(&mut OsRng);
        Self {
            secret_key: bob_secret,
        }
    }
}

impl UserAuthGreeter {
    fn get_public_key(&self) -> String {
        let bob_public = self.secret_key.public_key();
        let bob_public_key = hex::encode(bob_public.as_bytes());
        bob_public_key
    }

    // Utility function to convert a hex into public or secret key
    fn hex_to_key<T>(s: &str) -> Result<T>
    where
        T: for<'a> TryFrom<&'a [u8], Error = TryFromSliceError>,
    {
        let bytes = hex::decode(s)?;
        if bytes.len() != KEY_LEN {
            return Err(Error::InvalidLength {
                expected: KEY_LEN,
                received: bytes.len(),
            }
            .into());
        }
        let key = T::try_from(bytes.as_slice())?;
        Ok(key)
    }

    fn decode_message(&self, user_content: &UserPublicAuth) -> Result<String> {
        let user_public: PublicKey =
            UserAuthGreeter::hex_to_key(&user_content.ephemeral_public_key)?;
        let bob_box = ChaChaBox::new(&user_public, &self.secret_key);

        // Decrypt the message
        let nonce = hex::decode(user_content.nonce.clone()).unwrap();
        let nonce = nonce.as_slice().try_into()?;
        let user_contentten = hex::decode(&user_content.message).unwrap(); // Message is cipher(ed)
        let decrypted_data = bob_box
            .decrypt(nonce, user_contentten.as_slice())
            .map_err(|e| Error::CryptoError(e))?; //  "Decryption failed!")?;

        let msg = String::from_utf8(decrypted_data)?;
        Ok(msg)
    }
}

#[tonic::async_trait]
impl AuthGreeter for UserAuthGreeter {
    async fn greet_auth(
        &self,
        _request: Request<EmptyRequest>,
    ) -> std::result::Result<Response<ServerPublicKeys>, Status> {
        println!("Got a greet request");
        let public_keys = ServerPublicKeys {
            public_key: self.get_public_key(),
        };

        Ok(Response::new(public_keys))
    }

    async fn exchange_auth(
        &self,
        request: Request<UserPublicAuth>,
    ) -> std::result::Result<Response<UserAuthResponse>, Status> {
        // request is alice
        let alice = request.get_ref();

        match self.decode_message(alice) {
            Ok(msg) => {
                // Leak the keyword
                println!("{}", msg);

                Ok(Response::new(UserAuthResponse {
                    status: "OK".to_string(),
                }))
            }
            Err(_) => Ok(Response::new(UserAuthResponse {
                status: "Error".to_string(),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crypto_box::aead::AeadCore;

    #[derive(Serialize, Deserialize)]
    struct SecretsMessageDemo {
        ephemeral_public: String,
        nonce: String,
        ciphertext: String,
    }

    fn simulate_alice_aka_client(bob_public: String, message: &str) -> Result<SecretsMessageDemo> {
        let message = message.as_bytes(); // The user "auth" key

        let alice_ephemeral_secret = SecretKey::generate(&mut OsRng);
        let alice_ephemeral_public = alice_ephemeral_secret.public_key();
        // Alice Creates a new "box" with
        let bob_public_key: PublicKey = UserAuthGreeter::hex_to_key(bob_public.as_str())?;
        let alice_box = ChaChaBox::new(&bob_public_key, &alice_ephemeral_secret);

        // Generate a unique nonce
        let nonce = ChaChaBox::generate_nonce(&mut OsRng);

        // Encrypt the message
        let ciphertext = alice_box
            .encrypt(&nonce, message)
            .map_err(|_| "Encryption error")?;

        Ok(SecretsMessageDemo {
            ephemeral_public: hex::encode(alice_ephemeral_public.as_bytes()),
            nonce: hex::encode(nonce),
            ciphertext: hex::encode(&ciphertext),
        })
    }

    #[test]
    fn test_simulate_alice_user() {
        let bob_greeter = UserAuthGreeter::default();
        let bob_public = bob_greeter.get_public_key();
        let alice_message = "This is an user id";

        let alice_ciphers = simulate_alice_aka_client(bob_public, alice_message).unwrap();
        let alice_public_auth = UserPublicAuth {
            nonce: alice_ciphers.nonce,
            message: alice_ciphers.ciphertext,
            ephemeral_public_key: alice_ciphers.ephemeral_public,
        };

        let decrypted_key = bob_greeter.decode_message(&alice_public_auth).unwrap();

        assert_eq!(alice_message, decrypted_key);
    }
}
