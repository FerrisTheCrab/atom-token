#[cfg(feature = "core")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[cfg(feature = "core")]
use crate::{
    instance::TokenInstance,
    router::{InternalRouter, Router},
    Token,
};

#[derive(Serialize, Deserialize)]
pub struct CreateReq {
    #[serde(rename = "userID")]
    pub user_id: u64,
    pub label: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CreateRes {
    #[serde(rename = "created")]
    Created { token: String },
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl CreateRes {
    pub fn success(token: String) -> Self {
        Self::Created { token }
    }

    pub fn failure(e: mongodb::error::Error) -> Self {
        Self::Error {
            reason: e
                .get_custom::<String>()
                .cloned()
                .unwrap_or(e.kind.to_string()),
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::Created { .. } => StatusCode::OK,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn create(instance: &TokenInstance, payload: CreateReq) -> CreateRes {
        Token::create(instance, payload.user_id, payload.label)
            .await
            .map(CreateRes::success)
            .unwrap_or_else(CreateRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn create(
        State(instance): State<TokenInstance>,
        Json(payload): Json<CreateReq>,
    ) -> (StatusCode, Json<CreateRes>) {
        let res = InternalRouter::create(&instance, payload).await;
        (res.status(), Json(res))
    }
}
