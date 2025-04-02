use super::sealed_metric::SealedMetric;
use candid::Principal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct VideoDurationWatched {
    #[schema(value_type = String)]
    pub publisher_user_id: Principal,
    #[schema(value_type = String)]
    pub user_id: Principal,
    pub is_logged_in: bool,
    pub display_name: String,
    #[schema(value_type = String)]
    pub canister_id: Principal,
    pub video_id: String,
    pub video_category: String,
    pub creator_category: String,
    pub hashtag_count: u32,
    pub is_nsfw: bool,
    pub is_hot_or_not: bool,
    pub feed_type: String,
    pub view_count: u32,
    pub like_count: u32,
    pub share_count: u32,
    pub percentage_watched: f64,
    pub absolute_watched: f64,
    pub video_duration: f64,
    pub post_id: u64,
    pub publisher_canister_id: String,
    pub nsfw_probability: f64,
}

impl SealedMetric for VideoDurationWatched {
    fn tag(&self) -> String {
        "video_duration_watched".to_string()
    }

    fn user_id(&self) -> Option<String> {
        Some(self.user_id.to_text())
    }

    fn user_canister(&self) -> Option<Principal> {
        Some(self.canister_id)
    }
}
