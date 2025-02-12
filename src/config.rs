use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use mongodb::{
    options::{AuthMechanism, ClientOptions, Credential},
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::token::Token;

static MASTERCONFIG: OnceLock<MasterConfig> = OnceLock::new();
static TOKENS: OnceLock<Collection<Token>> = OnceLock::new();

#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde)]
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
#[derive(Serialize, Deserialize, DefaultFromSerde)]
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

    fn read() -> Self {
        let path = PathBuf::from(env::var("CONFIG").expect("env CONFIG not set"));

        if !path.exists() {
            Self::create(&path);
        }

        let content = fs::read(path).unwrap();
        serde_json::from_slice(&content).expect("bad JSON")
    }

    pub fn get() -> &'static Self {
        MASTERCONFIG.get_or_init(Self::read)
    }
}

impl MongoConfig {
    pub fn get() -> &'static Collection<Token> {
        TOKENS.get_or_init(Self::load)
    }

    fn load() -> Collection<Token> {
        futures::executor::block_on(async { MasterConfig::get().mongodb.get_collection().await })
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
