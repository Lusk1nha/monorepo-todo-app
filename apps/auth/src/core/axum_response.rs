use axum::{
  extract::{rejection::JsonRejection, FromRequest, Request},
  Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::errors::axum_response_errors::ServerError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
  T: DeserializeOwned + Validate,
  S: Send + Sync,
  Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
  type Rejection = ServerError;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
      let Json(value) = Json::<T>::from_request(req, state).await?;
      value.validate()?;
      Ok(ValidatedJson(value))
  }
}