use chrono::Utc;
use std::sync::Arc;

use crate::{
    entities::{auth_refresh_token_entity::Session, user_entity::User},
    errors::auth_service_errors::AuthServiceError,
    utils::password::verify_hash_password,
};

use super::{
    auth_refresh_token_service::AuthRefreshTokensService, credentials_service::CredentialsService,
    users_service::UsersService,
};

#[derive(Clone)]
pub struct AuthService {
    users_service: Arc<UsersService>,
    credentials_service: Arc<CredentialsService>,
    auth_refresh_tokens_service: Arc<AuthRefreshTokensService>,
}

impl AuthService {
    pub fn new(
        users_service: Arc<UsersService>,
        credentials_service: Arc<CredentialsService>,
        auth_refresh_tokens_service: Arc<AuthRefreshTokensService>,
    ) -> Self {
        Self {
            users_service,
            credentials_service,
            auth_refresh_tokens_service,
        }
    }

    pub async fn register_user_with_credentials(
        &self,
        email: &str,
        password: &str,
    ) -> Result<User, AuthServiceError> {
        let user: User = self.users_service.create_user(&email).await?;

        self.credentials_service
            .create_credential(&user.id, &password)
            .await?;

        Ok(user)
    }

    pub async fn login_with_credentials(
        &self,
        user: &User,
        password: &str,
    ) -> Result<Session, AuthServiceError> {
        let Some(credential) = self
            .credentials_service
            .get_credential_by_user_id(&user.id)
            .await?
        else {
            return Err(AuthServiceError::InvalidCredentials);
        };

        if !verify_hash_password(password, &credential.password_hash)? {
            return Err(AuthServiceError::InvalidCredentials);
        }

        self.auth_refresh_tokens_service
            .create_session(&user.id)
            .await
            .map_err(|_| AuthServiceError::CreateAuthRefreshTokenError)
    }

    pub async fn create_new_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<Session, AuthServiceError> {
        let stored_session = self
            .auth_refresh_tokens_service
            .find_session_by_hash(&refresh_token)
            .await
            .map_err(|_| AuthServiceError::CreateAuthRefreshTokenError)?;

        if stored_session.is_none() {
            return Err(AuthServiceError::RefreshTokenNotFound);
        }

        let stored_session = stored_session.unwrap();

        if stored_session.expires_at < Utc::now() {
            return Err(AuthServiceError::RefreshTokenNotFound);
        }

        self.auth_refresh_tokens_service
            .update_session(&stored_session)
            .await
            .map_err(|_| AuthServiceError::CreateAuthRefreshTokenError)
    }

    pub async fn revoke_refresh_token(&self, refresh_token: &str) -> Result<(), AuthServiceError> {
        let stored_session = self
            .auth_refresh_tokens_service
            .find_session_by_hash(&refresh_token)
            .await
            .map_err(|_| AuthServiceError::RevokeRefreshTokenError)?;

        if stored_session.is_none() {
            return Err(AuthServiceError::RefreshTokenNotFound);
        }

        let stored_session = stored_session.unwrap();

        self.auth_refresh_tokens_service
            .revoke_session_by_hash(&stored_session.id)
            .await
            .map_err(|_| AuthServiceError::RevokeRefreshTokenError)
    }
}
