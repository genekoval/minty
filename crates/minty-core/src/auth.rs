mod session;

pub use session::*;

use crate::{Error, Result};

use argon2::{
    password_hash::{
        errors::Error::Password as InvalidPassword, rand_core::OsRng,
        PasswordHasher, SaltString,
    },
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    self as jwt, errors::Result as JwtResult, DecodingKey, EncodingKey, Header,
    Validation,
};
use minty::text::Password;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Claims<T> {
    exp: i64,
    #[serde(flatten)]
    inner: T,
}

impl<T> Claims<T> {
    fn new(exp: Duration, inner: T) -> Self {
        let now = Utc::now();
        let exp = (now + exp).timestamp();

        Self { exp, inner }
    }
}

pub struct Auth {
    context: Argon2<'static>,
    jwt_decoding_key: DecodingKey,
    jwt_encoding_key: EncodingKey,
}

impl Auth {
    pub fn new(jwt_secret: &str) -> Self {
        let jwt_secret = jwt_secret.as_bytes();

        Self {
            context: Argon2::default(),
            jwt_decoding_key: DecodingKey::from_secret(jwt_secret),
            jwt_encoding_key: EncodingKey::from_secret(jwt_secret),
        }
    }

    pub fn decode_jwt<T: DeserializeOwned>(&self, token: &str) -> JwtResult<T> {
        let validation = Validation::default();

        let data = jwt::decode::<Claims<T>>(
            token,
            &self.jwt_decoding_key,
            &validation,
        )?;

        Ok(data.claims.inner)
    }

    pub fn encode_jwt<T: Serialize>(
        &self,
        exp: Duration,
        claims: T,
    ) -> Result<String> {
        let header = Header::default();
        let claims = Claims::new(exp, claims);

        jwt::encode(&header, &claims, &self.jwt_encoding_key).map_err(|err| {
            Error::Internal(format!("failed to encode jwt: {err}"))
        })
    }

    pub fn hash_password(&self, password: Password) -> Result<String> {
        let password = password.as_ref().as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash =
            self.context.hash_password(password, &salt).map_err(|err| {
                Error::Internal(format!("failed to hash password: {err}"))
            })?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(
        &self,
        password: &str,
        password_hash: &str,
    ) -> Result<bool> {
        let hash = PasswordHash::new(password_hash).map_err(|err| {
            Error::Internal(format!("invalid password hash: {err}"))
        })?;

        match self.context.verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(InvalidPassword) => Ok(false),
            Err(err) => Err(Error::Internal(format!(
                "failed to verify password: {err}"
            ))),
        }
    }
}
