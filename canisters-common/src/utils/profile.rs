use candid::Principal;
use canisters_client::individual_user_template::UserProfileDetailsForFrontend;
use serde::{Deserialize, Serialize};

use crate::consts::{GOBGOB_PROPIC_URL, GOBGOB_TOTAL_COUNT};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileDetails {
    pub username: Option<String>,
    pub lifetime_earnings: u64,
    pub followers_cnt: u64,
    pub following_cnt: u64,
    pub profile_pic: Option<String>,
    pub display_name: Option<String>,
    pub principal: Principal,
    pub hots: u64,
    pub nots: u64,
}

impl From<UserProfileDetailsForFrontend> for ProfileDetails {
    fn from(user: UserProfileDetailsForFrontend) -> Self {
        Self {
            username: user.unique_user_name,
            lifetime_earnings: user.lifetime_earnings,
            followers_cnt: user.followers_count,
            following_cnt: user.following_count,
            profile_pic: user.profile_picture_url,
            display_name: user.display_name,
            principal: user.principal_id,
            hots: user.profile_stats.hot_bets_received,
            nots: user.profile_stats.not_bets_received,
        }
    }
}

fn index_from_principal(principal: Principal) -> u32 {
    let hash_value = crc32fast::hash(principal.as_slice());
    (hash_value % GOBGOB_TOTAL_COUNT) + 1
}

pub fn propic_from_principal(principal: Principal) -> String {
    let index = index_from_principal(principal);
    format!("{GOBGOB_PROPIC_URL}{index}/public")
}

impl ProfileDetails {
    pub fn username_or_principal(&self) -> String {
        self.username
            .clone()
            .unwrap_or_else(|| self.principal.to_text())
    }

    pub fn principal(&self) -> String {
        self.principal.to_text()
    }

    pub fn display_name_or_fallback(&self) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| self.username_or_principal())
    }

    pub fn profile_pic_or_random(&self) -> String {
        let propic = self.profile_pic.clone().unwrap_or_default();
        if !propic.is_empty() {
            return propic;
        }

        propic_from_principal(self.principal)
    }
}
