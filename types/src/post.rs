use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostItem {
    pub canister_id: String,
    pub post_id: u64,
    pub video_id: String,
    pub nsfw_probability: f32,
}
