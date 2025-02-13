use std::path::Path;

use mongodb::Collection;

use crate::{MasterConfig, Token};

#[derive(Clone)]
pub struct TokenInstance {
    pub config: MasterConfig,
    pub tokens: Collection<Token>,
}

impl TokenInstance {
    pub fn load(config: &Path) -> Self {
        let config = MasterConfig::read(config);
        let tokens = config.mongodb.load();
        TokenInstance { config, tokens }
    }
}
