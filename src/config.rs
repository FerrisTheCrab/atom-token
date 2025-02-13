use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use mongodb::{
    options::{AuthMechanism, ClientOptions, Credential},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::token::Token;

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone)]
pub struct MasterConfig {
    #[serde_inline_default(8080)]
    pub port: u16,
    #[serde_inline_default(64)]
    #[serde(rename = "tokenLength")]
    pub token_length: usize,
    #[serde_inline_default(10)]
    #[serde(rename = "pageSize")]
    pub page_size: u64,
    #[serde(default)]
    pub mongodb: MongoConfig,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone)]
pub struct MongoConfig {
    #[serde_inline_default("mongodb://localhost:27017".to_string())]
    pub address: String,
    #[serde_inline_default("bob".to_string())]
    pub username: String,
    #[serde_inline_default("cratchit".to_string())]
    pub password: String,
    #[serde_inline_default("admin".to_string())]
    #[serde(rename = "authDB")]
    pub auth_db: String,
    #[serde_inline_default("atomics".to_string())]
    #[serde(rename = "pwDB")]
    pub pw_db: String,
}

impl MasterConfig {
    fn create(path: &Path) {
        let ser = serde_json::to_vec_pretty(&Self::default()).unwrap();

        if !path.parent().unwrap().exists() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
        }

        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap()
            .write_all(&ser)
            .unwrap();
    }

    pub fn read(path: &Path) -> Self {
        if !path.exists() {
            Self::create(path);
        }

        let content = fs::read(path).unwrap();
        serde_json::from_slice(&content).expect("bad JSON")
    }
}

impl MongoConfig {
    pub fn load(&self) -> Collection<Token> {
        futures::executor::block_on(async { self.get_collection().await })
    }

    async fn get_collection(&self) -> Collection<Token> {
        let mut client_opts = ClientOptions::parse(&self.address).await.unwrap();

        let scram_sha_1_cred = Credential::builder()
            .username(self.username.clone())
            .password(self.password.clone())
            .mechanism(AuthMechanism::ScramSha1)
            .source(self.auth_db.clone())
            .build();

        client_opts.credential = Some(scram_sha_1_cred);
        let client = Client::with_options(client_opts).unwrap();
        client.database(&self.pw_db).collection("token")
    }
}
