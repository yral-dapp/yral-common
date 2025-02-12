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

#[derive(Debug, Clone)]
pub struct TokenListItemFSWithTimestamp {
    pub user_id: String,
    pub name: String,
    pub token_name: String,
    pub token_symbol: String,
    pub logo: String,
    pub description: String,
    pub created_at: String,
    pub link: String,
    pub is_nsfw: bool,
    pub timestamp: i64,
}

impl TokenListItemFSWithTimestamp {
    pub fn from_token_list_item_fs(
        item: TokenListItemFS,
        timestamp: i64,
    ) -> TokenListItemFSWithTimestamp {
        TokenListItemFSWithTimestamp {
            user_id: item.user_id,
            name: item.name,
            token_name: item.token_name,
            token_symbol: item.token_symbol,
            logo: item.logo,
            description: item.description,
            created_at: item.created_at,
            link: item.link,
            is_nsfw: item.is_nsfw,
            timestamp,
        }
    }
}

pub trait TokenInfoProvider {
    type Error;

    fn get_token_by_id(
        &self,
        token_id: String,
    ) -> impl Future<Output = Result<TokenListItemFSWithTimestamp, Self::Error>> + Send;
}
