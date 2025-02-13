use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

use crate::{
    instance::TokenInstance,
    router::{InternalRouter, Router},
    Token,
};

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct ListReq {
    #[serde(rename = "userID")]
    pub user_id: u64,
    #[serde_inline_default(0)]
    pub page: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ListedToken {
    pub token: String,
    pub created: u64,
    pub label: String,
}

impl From<Token> for ListedToken {
    fn from(value: Token) -> Self {
        Self {
            token: value.id,
            created: value.created,
            label: value.label,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ListRes {
    #[serde(rename = "list")]
    List { tokens: Vec<ListedToken> },
    #[serde(rename = "error")]
    Error { reason: String },
}

impl ListRes {
    pub fn success(tokens: Vec<Token>) -> Self {
        Self::List {
            tokens: tokens.into_iter().map(ListedToken::from).collect(),
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
            Self::List { .. } => StatusCode::OK,
            Self::Error { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl InternalRouter {
    pub async fn list(instance: &TokenInstance, payload: ListReq) -> ListRes {
        Token::show(instance, payload.user_id, payload.page)
            .await
            .map(ListRes::success)
            .unwrap_or_else(ListRes::failure)
    }
}

impl Router {
    pub async fn list(
        State(instance): State<TokenInstance>,
        Json(payload): Json<ListReq>,
    ) -> (StatusCode, Json<ListRes>) {
        let res = InternalRouter::list(&instance, payload).await;
        (res.status(), Json(res))
    }
}
