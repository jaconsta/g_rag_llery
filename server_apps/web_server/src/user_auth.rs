use std::ops::Add;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, sync::Arc};

use hex::ToHex;
use libsodium_rs::crypto_box::Nonce;
use libsodium_rs::{self, SodiumError, crypto_box};

use hex;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, time::Duration};
use tonic::{Request, Response, Status};

use twox_hash::XxHash3_64;
pub use user_auth_rpc::auth_greeter_server::{AuthGreeter, AuthGreeterServer};
use user_auth_rpc::{
    EmptyRequest, EmptyResponse, ServerPublicKeys, UserAuthResponse, UserPublicAuth,
};

use crate::error::Result;

pub mod user_auth_rpc {
    tonic::include_proto!("user_auth"); // The string specified here must match the proto package name
}

#[derive(Debug)]
pub struct SessionValidator {
    sessions: Arc<RwLock<UserSessions>>,
}
impl SessionValidator {
    pub fn new(sessions: Arc<RwLock<UserSessions>>) -> Self {
        Self { sessions }
    }

    pub async fn get_user<T>(&self, r: &Request<T>) -> Result<UserId> {
        let token = match get_token(r) {
            Ok(t) => t,
            Err(x) => return Err(x.message().into()),
        };

        let user_id = match self
            .sessions
            .read()
            .await
            .validate_user_session(token)
            .await
        {
            Some(u) => u,
            None => return Err("session expired".into()),
        };

        Ok(user_id)
    }
}

pub struct UserAuthGreeter {
    box_key_pair: crypto_box::KeyPair,
    sessions: Arc<RwLock<UserSessions>>,
}

impl Default for UserAuthGreeter {
    fn default() -> Self {
        let box_key_pair = crypto_box::KeyPair::generate();

        Self {
            box_key_pair,
            sessions: Arc::new(RwLock::new(UserSessions::new())),
        }
    }
}

impl UserAuthGreeter {
    pub fn new(sessions: Arc<RwLock<UserSessions>>) -> Self {
        Self {
            sessions,
            ..Default::default()
        }
    }
    fn get_public_key(&self) -> String {
        self.box_key_pair.public_key.encode_hex()
    }

    // Utility function to convert a hex into public or secret key
    fn hex_to_key<T>(s: &str) -> Result<T>
    where
        T: for<'a> TryFrom<&'a [u8], Error = SodiumError>,
    {
        let bytes = hex::decode(s)?;
        let key = T::try_from(bytes.as_slice())?;
        Ok(key)
    }

    fn decode_message(&self, user_content: &UserPublicAuth) -> Result<String> {
        let bob_public = crypto_box::PublicKey::from_bytes(
            hex::decode(user_content.ephemeral_public_key.as_bytes())?.as_ref(),
        )?;

        let bob_nonce: Nonce = Self::hex_to_key(&user_content.nonce)?;
        let cipher = hex::decode(&user_content.message)?;

        let decrypted = crypto_box::open(
            &cipher,
            &bob_nonce,
            &bob_public,
            &self.box_key_pair.secret_key,
        )?;
        let decipher_message = String::from_utf8(decrypted)?;
        Ok(decipher_message)
    }
}

#[tonic::async_trait]
impl AuthGreeter for UserAuthGreeter {
    async fn greet_auth(
        &self,
        _request: Request<EmptyRequest>,
    ) -> std::result::Result<Response<ServerPublicKeys>, Status> {
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
        let sessions = self.sessions.clone();

        let user_code = match self.decode_message(alice) {
            Ok(user_unique_code) => user_unique_code,
            Err(_) => {
                return Err(Status::failed_precondition("Wrong data"));
            }
        };

        match sessions.write().await.generate_new_session(user_code).await {
            Ok((token, token_expire)) => Ok(Response::new(UserAuthResponse {
                status: "OK".to_string(),
                bearer: token,
                expires: token_expire.as_millis() as u64,
            })),
            Err(_) => Err(Status::failed_precondition("Error")),
        }
    }

    async fn logout(
        &self,
        request: Request<EmptyRequest>,
    ) -> std::result::Result<Response<EmptyResponse>, Status> {
        let jwt_token = get_token(&request)?;

        self.sessions
            .write()
            .await
            .from_token_and_pop(jwt_token)
            .await;

        Ok(Response::new(EmptyResponse {}))
    }
}

pub fn get_token<'a, T>(r: &'a Request<T>) -> std::result::Result<&'a str, Status> {
    let jwt_token = r
        .metadata()
        .get("x-authorization")
        .ok_or(Status::unauthenticated("No access token specified"))?
        .to_str()
        .map_err(|_| Status::unauthenticated("No access token specified"))?;

    Ok(jwt_token)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    // Audience.
    aud: String,
    // Subject. Whom the token refers to.
    sub: String,
    // Expiration Datetime in seconds.
    exp: u64,
    // The key for User session hashmap.
    user_id: String,
}

impl Claims {
    pub fn aud() -> &'static str {
        "jaconsta"
    }
    pub fn sub() -> &'static str {
        "me@jaconsta.com"
    }
}

/// Hash key that points to UserId in the session map.
/// Note: Try to keep it private.
type AuthId = u64;

/// The link for the user information.
/// The user_id stored in db tables.
pub type UserId = String;
#[derive(Debug)]
pub struct UserSessions {
    /// In-memory storage of the user sessions.
    /// (eventually Valkey when shared session between servers becomes necessary?)
    user_sessions: Arc<RwLock<HashMap<AuthId, UserId>>>,
    /// Token and session expiry. In minutes.
    ttl_mins: u64,
    /// Signature secret for the jwt.
    jwt_secret: String,
    /// Seed for hash map key and user_id key.
    hash_seed: u64,
}

impl UserSessions {
    pub fn new() -> Self {
        // Old jwts will be nulled after each reset.
        let jwt_secret = Alphanumeric.sample_string(&mut rand::rng(), 32);

        Self {
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            ttl_mins: 120,
            jwt_secret,
            hash_seed: 0xdead_cafe,
        }
    }

    /// Stores the information in storage and returns JWT
    pub async fn generate_new_session(
        &mut self,
        user_true_code: String,
    ) -> Result<(String, Duration)> {
        // The user session id should be unique to prevent traces of it
        // in the system.
        let user_session_id = Alphanumeric.sample_string(&mut rand::rng(), 16);
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_millis(0));
        let token_exp = since_the_epoch.add(Duration::from_mins(self.ttl_mins));

        let claims = Claims {
            aud: Claims::aud().into(),
            sub: Claims::sub().into(),
            exp: token_exp.as_secs(),
            user_id: user_session_id.clone(),
        };

        let token = match jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        ) {
            Ok(t) => t,
            Err(_) => return Err("Ohh noo".into()),
        };

        // Maybe change the hash function.
        // Something like Blake3 (fast) or Argon2 (passwords)
        let hashed = XxHash3_64::oneshot_with_seed(self.hash_seed, user_true_code.as_ref());
        // This is user_id to store as reference in db
        // Hex representation.
        let user_code_hash = format!("{:x}", hashed);

        // Using xxHash3 for hashmap keys.
        let auth_id = XxHash3_64::oneshot_with_seed(self.hash_seed, user_session_id.as_ref());

        self.set_user(auth_id, user_code_hash.into()).await;
        self.timeout_pop(auth_id);

        Ok((token, token_exp))
    }

    /// Takes the JWT. Validate it and extract the user_id
    /// Currently the user_id is the only "business" information the JWT stores.
    pub async fn validate_user_session(&self, jwt_token: &str) -> Option<UserId> {
        let auth_id = match self.decode_token(jwt_token) {
            Some(id) => id,
            None => return None,
        };

        let user_id = self.get_user(&auth_id).await;

        return user_id;
    }

    pub async fn from_token_and_pop(&mut self, jwt_token: &str) {
        let auth_id = match self.decode_token(jwt_token) {
            Some(id) => id,
            None => return,
        };
        self.pop_user(&auth_id).await;
    }

    fn decode_token(&self, jwt_token: &str) -> Option<AuthId> {
        let mut jwt_validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        jwt_validation.set_audience(&[Claims::aud()]);
        jwt_validation.sub = Some(Claims::sub().to_string());
        jwt_validation.set_required_spec_claims(&["sub", "exp", "user_id"]);

        let token_data = match jsonwebtoken::decode::<Claims>(
            jwt_token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &jwt_validation,
        ) {
            Ok(c) => c,
            Err(_) => return None,
        };

        // Generate hashMap key.
        let auth_id =
            XxHash3_64::oneshot_with_seed(self.hash_seed, token_data.claims.user_id.as_ref());

        auth_id.into()
    }

    /// Expected usecase: When want to validate the current user session.
    pub async fn get_user(&self, auth_id: &AuthId) -> Option<UserId> {
        let sessions = self.user_sessions.read().await;
        sessions.get(auth_id).cloned()
    }

    /// Expected usecase: Add a new user session.
    pub async fn set_user(&mut self, auth_id: AuthId, user_id: UserId) {
        let mut sessions = self.user_sessions.write().await;
        sessions.insert(auth_id, user_id);
    }

    /// Expected usecase: Logout the use.
    pub async fn pop_user(&mut self, auth_id: &AuthId) {
        let mut sessions = self.user_sessions.write().await;
        sessions.remove(auth_id);
    }

    /// Expected usecase: Logout the user after session expiration.
    pub fn timeout_pop(&mut self, auth_id: AuthId) {
        let ttl = self.ttl_mins.clone();
        let session = self.user_sessions.clone();
        // Currently there is no cancelation of the timeout.
        // It is assumed that if `sessions.remove` returns empty
        // object, it means the user is logged out.
        let _timer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_mins(ttl)).await;

            // Not using self.pop_user because of lifetimes issue.
            let mut sessions = session.write().await;
            sessions.remove(&auth_id);
        });
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use tonic::transport::Endpoint;

    use super::*;

    static RPC_SERVER_URL: &'static str = "http://0.0.0.0:50051";
    // Utility function to convert a hex into public or secret key
    fn hex_to_key<T>(s: &str) -> Result<T>
    where
        T: for<'a> TryFrom<&'a [u8], Error = SodiumError>,
    {
        let bytes = hex::decode(s)?;
        let key = T::try_from(bytes.as_slice())?;
        Ok(key)
    }

    #[derive(Serialize, Deserialize)]
    struct SecretsMessageDemo {
        ephemeral_public: String,
        nonce: String,
        ciphertext: String,
    }

    fn simulate_alice_aka_client(bob_public: String, message: &str) -> Result<SecretsMessageDemo> {
        let (alice_sk, alice_pk) = {
            let alice_keypair = crypto_box::KeyPair::generate();
            let alice_public = alice_keypair.public_key.encode_hex();

            (alice_keypair.secret_key, alice_public)
        };

        let bob_pk: crypto_box::PublicKey =
            hex_to_key(&bob_public).expect("Failed to generate key");

        // Generate a random nonce
        let nonce = crypto_box::Nonce::generate();
        let shared_nonce = nonce.encode_hex::<String>();
        let nonce: crypto_box::Nonce = hex_to_key(&shared_nonce).unwrap();

        // Alice encrypts a message for Bob
        let ciphertext = crypto_box::seal(message.as_bytes(), &nonce, &bob_pk, &alice_sk)
            .expect("Failed to seal box");

        Ok(SecretsMessageDemo {
            ephemeral_public: alice_pk,
            nonce: shared_nonce,
            ciphertext: ciphertext.encode_hex(),
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

    #[tokio::test]
    async fn test_query_base_for_public_key() {
        let channel = Endpoint::from_static(RPC_SERVER_URL)
            .connect()
            .await
            .unwrap();

        // Start the process.
        let mut client = user_auth_rpc::auth_greeter_client::AuthGreeterClient::new(channel);
        // Get the client request
        let creds = client
            .greet_auth(tonic::Request::new(EmptyRequest {}))
            .await
            .unwrap();

        assert!(!creds.get_ref().public_key.is_empty());
    }
    #[tokio::test]
    async fn test_generate_new_jwt_user() {
        let channel = Endpoint::from_static(RPC_SERVER_URL)
            .connect()
            .await
            .unwrap();

        // Start the process.
        let mut client = user_auth_rpc::auth_greeter_client::AuthGreeterClient::new(channel);

        let creds = client
            .greet_auth(tonic::Request::new(EmptyRequest {}))
            .await
            .unwrap();
        let bob_public = creds.get_ref().public_key.clone();
        let alice_user_code = "Usercode1234andMaybemin32length.";

        let alice_ciphers = simulate_alice_aka_client(bob_public, alice_user_code).unwrap();
        let alice_public_auth = UserPublicAuth {
            nonce: alice_ciphers.nonce,
            message: alice_ciphers.ciphertext,
            ephemeral_public_key: alice_ciphers.ephemeral_public,
        };

        // Get the client for auth token request
        let creds = client
            .exchange_auth(tonic::Request::new(alice_public_auth))
            .await
            .unwrap();

        let creds = creds.get_ref();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_millis(0));

        assert_eq!(creds.status, "OK");
        assert!(!creds.bearer.is_empty());
        assert!(creds.expires > now.as_secs());
    }
}
