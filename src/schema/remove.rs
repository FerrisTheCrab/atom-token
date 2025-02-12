use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    router::{InternalRouter, Router},
    Token,
};

#[derive(Serialize, Deserialize)]
pub struct RemoveReq {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoveRes {
    #[serde(rename = "removed")]
    Removed,
    #[serde(rename = "error")]
    Error { reason: String },
}

impl RemoveRes {
    pub fn success(_: ()) -> Self {
        Self::Removed
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
            Self::Removed => StatusCode::OK,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn remove(payload: RemoveReq) -> RemoveRes {
        Token::remove(&payload.token)
            .await
            .map(RemoveRes::success)
            .unwrap_or_else(RemoveRes::failure)
    }
}

impl Router {
    pub async fn remove(Json(payload): Json<RemoveReq>) -> (StatusCode, Json<RemoveRes>) {
        let res = InternalRouter::remove(payload).await;
        (res.status(), Json(res))
    }
}
