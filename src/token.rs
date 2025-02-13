use chrono::Utc;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Bson},
    error::WriteFailure,
};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};

use crate::instance::TokenInstance;

macro_rules! not_found {
    () => {
        mongodb::error::Error::custom("token not found".to_string())
    };
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "userID")]
    pub user_id: u64,
    pub created: u64, // unix timestamp UTC
    pub label: String,
}

impl Token {
    async fn find(
        instance: &TokenInstance,
        token: String,
    ) -> Result<Option<Token>, mongodb::error::Error> {
        instance
            .tokens
            .find_one(doc! { "_id": Bson::String(token) })
            .await
    }

    async fn list(
        instance: &TokenInstance,
        user_id: u64,
        page: u64,
    ) -> Result<Vec<Token>, mongodb::error::Error> {
        let mut cursor = instance
            .tokens
            .find(doc! { "userID": Bson::Int64(user_id as i64) })
            .skip(page * instance.config.page_size)
            .limit(instance.config.page_size as i64)
            .await?;

        let mut tokens = Vec::new();

        while let Some(token) = cursor.next().await {
            tokens.push(token?);
        }

        Ok(tokens)
    }

    async fn update(
        instance: &TokenInstance,
        token: &str,
        label: &str,
    ) -> Result<(), mongodb::error::Error> {
        if instance
            .tokens
            .update_one(doc! { "_id": &token}, doc! { "$set": {"label": label}})
            .await?
            .matched_count
            == 0
        {
            Err(not_found!())
        } else {
            Ok(())
        }
    }

    #[allow(clippy::wrong_self_convention, clippy::new_ret_no_self)]
    #[async_recursion::async_recursion]
    async fn new(
        instance: &TokenInstance,
        user_id: u64,
        label: String,
    ) -> Result<String, mongodb::error::Error> {
        let id = Alphanumeric.sample_string(&mut rand::rng(), instance.config.token_length);

        let to_insert = Self {
            id: id.clone(),
            user_id,
            created: Utc::now().timestamp() as u64,
            label: label.clone(),
        };

        match instance.tokens.insert_one(to_insert).await {
            Ok(_) => Ok(id),
            Err(e) => match *e.kind {
                mongodb::error::ErrorKind::Write(WriteFailure::WriteError(write_error))
                    if write_error.code == 11000 =>
                {
                    Self::new(instance, user_id, label).await // if two tokens somehow clashes, do it again
                }
                _ => Err(e),
            },
        }
    }

    async fn delete(instance: &TokenInstance, token: &str) -> Result<bool, mongodb::error::Error> {
        Ok(instance
            .tokens
            .delete_one(doc! { "_id": token })
            .await?
            .deleted_count
            == 1)
    }
}

impl Token {
    pub async fn create(
        instance: &TokenInstance,
        user_id: u64,
        label: String,
    ) -> Result<String, mongodb::error::Error> {
        Self::new(instance, user_id, label).await
    }

    pub async fn set(
        instance: &TokenInstance,
        token: &str,
        label: &str,
    ) -> Result<(), mongodb::error::Error> {
        Self::update(instance, token, label).await
    }

    pub async fn remove(
        instance: &TokenInstance,
        token: &str,
    ) -> Result<(), mongodb::error::Error> {
        if Self::delete(instance, token).await? {
            Ok(())
        } else {
            Err(not_found!())
        }
    }

    pub async fn get(
        instance: &TokenInstance,
        token: String,
    ) -> Result<Self, mongodb::error::Error> {
        match Self::find(instance, token).await? {
            Some(got) => Ok(got),
            None => Err(not_found!()),
        }
    }

    pub async fn show(
        instance: &TokenInstance,
        user_id: u64,
        page: u64,
    ) -> Result<Vec<Self>, mongodb::error::Error> {
        Self::list(instance, user_id, page).await
    }
}
