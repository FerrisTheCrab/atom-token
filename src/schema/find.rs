use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    instance::TokenInstance,
    router::{InternalRouter, Router},
    Token,
};

#[derive(Serialize, Deserialize)]
pub struct FindReq {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FindRes {
    #[serde(rename = "found")]
    Found {
        user_id: u64,
        label: String,
        created: u64,
    },
    #[serde(rename = "error")]
    Error { reason: String },
}

impl FindRes {
    pub fn success(token: Token) -> Self {
        Self::Found {
            user_id: token.user_id,
            label: token.label,
            created: token.created,
        }
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
            Self::Found { .. } => StatusCode::OK,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn find(instance: &TokenInstance, payload: FindReq) -> FindRes {
        Token::get(instance, payload.token)
            .await
            .map(FindRes::success)
            .unwrap_or_else(FindRes::failure)
    }
}

impl Router {
    pub async fn find(
        State(instance): State<TokenInstance>,
        Json(payload): Json<FindReq>,
    ) -> (StatusCode, Json<FindRes>) {
        let res = InternalRouter::find(&instance, payload).await;
        (res.status(), Json(res))
    }
}
