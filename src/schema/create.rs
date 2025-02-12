use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
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

impl InternalRouter {
    pub async fn create(payload: CreateReq) -> CreateRes {
        Token::create(payload.user_id, payload.label)
            .await
            .map(CreateRes::success)
            .unwrap_or_else(CreateRes::failure)
    }
}

impl Router {
    pub async fn create(Json(payload): Json<CreateReq>) -> (StatusCode, Json<CreateRes>) {
        let res = InternalRouter::create(payload).await;
        (res.status(), Json(res))
    }
}
