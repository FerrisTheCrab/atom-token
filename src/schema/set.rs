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
pub struct SetReq {
    pub token: String,
    pub label: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SetRes {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(feature = "core")]
impl SetRes {
    pub fn success(_: ()) -> Self {
        Self::Set
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
            Self::Set => StatusCode::OK,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "core")]
impl InternalRouter {
    pub async fn set(instance: &TokenInstance, payload: SetReq) -> SetRes {
        Token::set(instance, &payload.token, &payload.label)
            .await
            .map(SetRes::success)
            .unwrap_or_else(SetRes::failure)
    }
}

#[cfg(feature = "core")]
impl Router {
    pub async fn set(
        State(instance): State<TokenInstance>,
        Json(payload): Json<SetReq>,
    ) -> (StatusCode, Json<SetRes>) {
        let res = InternalRouter::set(&instance, payload).await;
        (res.status(), Json(res))
    }
}
