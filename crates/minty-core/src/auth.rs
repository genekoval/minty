use crate::Result;

use argon2::{
    password_hash::{
        errors::Error, rand_core::OsRng, PasswordHasher, SaltString,
    },
    Argon2, PasswordHash, PasswordVerifier,
};
use minty::text::Password;

#[derive(Debug)]
pub struct Auth {
    context: Argon2<'static>,
}

impl Auth {
    pub fn new() -> Self {
        Self {
            context: Argon2::default(),
        }
    }

    pub fn hash_password(&self, password: Password) -> Result<String> {
        let password = password.as_ref().as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash =
            self.context.hash_password(password, &salt).map_err(|err| {
                crate::Error::Internal(format!(
                    "failed to hash password: {err}"
                ))
            })?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(
        &self,
        password: &str,
        password_hash: &str,
    ) -> Result<bool> {
        let hash = PasswordHash::new(password_hash).map_err(|err| {
            crate::Error::Internal(format!("invalid password hash: {err}"))
        })?;

        match self.context.verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(Error::Password) => Ok(false),
            Err(err) => Err(crate::Error::Internal(format!(
                "failed to verify password: {err}"
            ))),
        }
    }
}
