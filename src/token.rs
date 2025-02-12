use chrono::Utc;
use futures::StreamExt;
use mongodb::{
    bson::{doc, Bson},
    error::WriteFailure,
};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};

use crate::config::{MasterConfig, MongoConfig};

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
    async fn find(token: String) -> Result<Option<Token>, mongodb::error::Error> {
        MongoConfig::get()
            .find_one(doc! { "_id": Bson::String(token) })
            .await
    }

    async fn list(user_id: u64, page: u64) -> Result<Vec<Token>, mongodb::error::Error> {
        let mut cursor = MongoConfig::get()
            .find(doc! { "userID": Bson::Int64(user_id as i64) })
            .skip(page * MasterConfig::get().page_size)
            .limit(MasterConfig::get().page_size as i64)
            .await?;

        let mut tokens = Vec::new();

        while let Some(token) = cursor.next().await {
            tokens.push(token?);
        }

        Ok(tokens)
    }

    async fn update(token: &str, label: &str) -> Result<(), mongodb::error::Error> {
        if MongoConfig::get()
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
    async fn new(user_id: u64, label: String) -> Result<String, mongodb::error::Error> {
        let id = Alphanumeric.sample_string(&mut rand::rng(), MasterConfig::get().token_length);

        let to_insert = Self {
            id: id.clone(),
            user_id,
            created: Utc::now().timestamp() as u64,
            label: label.clone(),
        };

        match MongoConfig::get().insert_one(to_insert).await {
            Ok(_) => Ok(id),
            Err(e) => match *e.kind {
                mongodb::error::ErrorKind::Write(WriteFailure::WriteError(write_error))
                    if write_error.code == 11000 =>
                {
                    Self::new(user_id, label).await // if two tokens somehow clashes, do it again
                }
                _ => Err(e),
            },
        }
    }

    async fn delete(token: &str) -> Result<bool, mongodb::error::Error> {
        Ok(MongoConfig::get()
            .delete_one(doc! { "_id": token })
            .await?
            .deleted_count
            == 1)
    }
}

impl Token {
    pub async fn create(user_id: u64, label: String) -> Result<String, mongodb::error::Error> {
        Self::new(user_id, label).await
    }

    pub async fn set(token: &str, label: &str) -> Result<(), mongodb::error::Error> {
        Self::update(token, label).await
    }

    pub async fn remove(token: &str) -> Result<(), mongodb::error::Error> {
        if Self::delete(token).await? {
            Ok(())
        } else {
            Err(not_found!())
        }
    }

    pub async fn get(token: String) -> Result<Self, mongodb::error::Error> {
        match Self::find(token).await? {
            Some(got) => Ok(got),
            None => Err(not_found!()),
        }
    }

    pub async fn show(user_id: u64, page: u64) -> Result<Vec<Self>, mongodb::error::Error> {
        Self::list(user_id, page).await
    }
}
