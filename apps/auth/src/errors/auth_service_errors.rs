use thiserror::Error;

use super::{credentials_errors::CredentialsError, users_errors::UsersError};

#[derive(Error, Debug)]
pub enum AuthServiceError {
    #[error("Database error: {0}")]
    TransactionError(#[from] sqlx::Error),

    #[error("Create user error: {0}")]
    CreateUser(#[from] UsersError),

    #[error("Create credential error: {0}")]
    CreateCredential(#[from] CredentialsError),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Password hashing error: {0}")]
    PasswordHashingError(#[from] bcrypt::BcryptError),

    #[error("User not found")]
    UserNotFound,

    #[error("Refresh token not found")]
    RefreshTokenNotFound,

    #[error("Create token error")]
    CreateAuthRefreshTokenError,

    #[error("Revoke token error")]
    RevokeRefreshTokenError
}
