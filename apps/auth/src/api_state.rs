use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::TimeDelta;
use mail_service::{MailService, errors::MailServiceError};
use tracing::error;

use crate::{
    database::DatabaseApp,
    environment::EnvironmentApp,
    repositories::{
        auth_providers_repository::AuthProvidersRepository,
        auth_refresh_token_repository::AuthRefreshTokensRepository,
        credentials_repository::CredentialsRepository,
        email_verifications_repository::EmailVerificationsRepository,
        otp_codes_repository::OTPCodesRepository, users_repository::UsersRepository,
    },
    services::{
        auth_providers_service::AuthProvidersService,
        auth_refresh_token_service::AuthRefreshTokensService, auth_service::AuthService,
        credentials_service::CredentialsService,
        email_verifications_service::EmailVerificationsService, otp_codes_service::OTPCodesService,
        users_service::UsersService,
    },
    utils::jwt::JwtConfig,
};

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub database: DatabaseApp,
    pub environment: EnvironmentApp,

    pub auth_service: Arc<AuthService>,
    pub mail_service: Arc<MailService>,

    pub users_service: Arc<UsersService>,
    pub otp_service: Arc<OTPCodesService>,
    pub auth_providers_service: Arc<AuthProvidersService>,
    pub auth_refresh_tokens_service: Arc<AuthRefreshTokensService>,
    pub credentials_service: Arc<CredentialsService>,
}

impl AppState {
    pub async fn new(
        database: DatabaseApp,
        environment: EnvironmentApp,
    ) -> Result<Arc<Self>, anyhow::Error> {
        let otp_service = Arc::new(OTPCodesService::new(
            OTPCodesRepository::new(database.clone()),
            TimeDelta::minutes(5),
        ));

        let email_verifications_service = Arc::new(EmailVerificationsService::new(
            EmailVerificationsRepository::new(database.clone()),
            TimeDelta::days(1),
            environment.hmac_secret.clone(),
            environment.web_page_config.clone(),
        ));

        let users_service = Arc::new(UsersService::new(
            UsersRepository::new(database.clone()),
            otp_service.clone(),
        ));

        let credentials_service = Arc::new(CredentialsService::new(CredentialsRepository::new(
            database.clone(),
        )));

        let auth_providers_service = Arc::new(AuthProvidersService::new(
            AuthProvidersRepository::new(database.clone()),
        ));

        let jwt_config = JwtConfig {
            secret: environment.jwt_secret.clone(),
            access_token_duration: TimeDelta::days(1),
            refresh_token_duration: TimeDelta::days(7),
        };

        let auth_refresh_tokens_service = Arc::new(AuthRefreshTokensService::new(
            AuthRefreshTokensRepository::new(database.clone()),
            jwt_config,
        ));

        let mail_service = Arc::new(Self::mail_deliver_service(&environment).await?);

        let auth_service = Arc::new(AuthService::new(
            Arc::clone(&users_service),
            Arc::clone(&credentials_service),
            Arc::clone(&auth_refresh_tokens_service),
            Arc::clone(&otp_service),
            Arc::clone(&email_verifications_service),
            Arc::clone(&mail_service),
        ));

        Ok(Arc::new(Self {
            database,
            environment,

            auth_service,
            mail_service,

            users_service,
            otp_service,
            auth_providers_service,
            auth_refresh_tokens_service,
            credentials_service,
        }))
    }

    async fn mail_deliver_service(
        environment: &EnvironmentApp,
    ) -> Result<MailService, MailServiceError> {
        const TEMPLATES_DIR: &str = "src/templates";

        let config = environment.smtp_config.clone();
        let source_dir = environment.manifest_dir.clone();

        let join_current = PathBuf::from(source_dir).join(TEMPLATES_DIR);

        let template_dir = Path::new(&join_current);

        if !template_dir.exists() {
            error!("Templates directory '{}' not found", TEMPLATES_DIR);

            return Err(MailServiceError::SetupService(
                format!("Templates directory '{}' does not exist", TEMPLATES_DIR).into(),
            ));
        }

        let mail = MailService::new(config.clone(), template_dir, 100)
            .await
            .expect("Failed to create mail service");

        Ok(mail)
    }
}
