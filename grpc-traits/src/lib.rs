use std::future::Future;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenListItemFS {
    pub user_id: String,
    pub name: String,
    pub token_name: String,
    pub token_symbol: String,
    pub logo: String,
    pub description: String,
    pub created_at: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub is_nsfw: bool,
}

pub trait TokenInfoProvider {
    type Error;

    fn get_token_by_id(
        &self,
        token_id: String,
    ) -> impl Future<Output = Result<TokenListItemFS, Self::Error>> + Send;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AirdropConfig {
    pub cycle_duration: u64,
    pub claim_limit: usize,
}

pub trait AirdropConfigProvider {
    fn get_airdrop_config(&self, symbol: String) -> impl Future<Output = AirdropConfig> + Send;
}
