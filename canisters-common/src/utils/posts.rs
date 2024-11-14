use candid::Principal;
use canisters_client::individual_user_template::{PostDetailsForFrontend, PostStatus};
use serde::{Deserialize, Serialize};
use web_time::Duration;

use crate::{Canisters, Result};

use super::profile::propic_from_principal;

#[derive(Clone, PartialEq, Ord, PartialOrd, Debug, Hash, Eq, Serialize, Deserialize)]
pub struct PostDetails {
    pub canister_id: Principal, // canister id of the publishing canister.
    pub post_id: u64,
    pub uid: String,
    pub description: String,
    pub views: u64,
    pub likes: u64,
    pub display_name: String,
    pub propic_url: String,
    /// Whether post is liked by the authenticated
    /// user or not, None if unknown
    pub liked_by_user: Option<bool>,
    pub poster_principal: Principal,
    pub hastags: Vec<String>,
    pub is_nsfw: bool,
    pub hot_or_not_feed_ranking_score: Option<u64>,
    pub created_at: Duration,
}

impl PostDetails {
    pub fn from_canister_post(
        authenticated: bool,
        canister_id: Principal,
        details: PostDetailsForFrontend,
    ) -> Self {
        Self {
            canister_id,
            post_id: details.id,
            uid: details.video_uid,
            description: details.description,
            views: details.total_view_count,
            likes: details.like_count,
            display_name: details
                .created_by_display_name
                .or(details.created_by_unique_user_name)
                .unwrap_or_else(|| details.created_by_user_principal_id.to_text()),
            propic_url: details
                .created_by_profile_photo_url
                .unwrap_or_else(|| propic_from_principal(details.created_by_user_principal_id)),
            liked_by_user: authenticated.then_some(details.liked_by_me),
            poster_principal: details.created_by_user_principal_id,
            hastags: details.hashtags,
            is_nsfw: details.is_nsfw,
            hot_or_not_feed_ranking_score: details.hot_or_not_feed_ranking_score,
            created_at: Duration::new(
                details.created_at.secs_since_epoch,
                details.created_at.nanos_since_epoch,
            ),
        }
    }

    pub fn is_hot_or_not(&self) -> bool {
        self.hot_or_not_feed_ranking_score.is_some()
    }
}

impl<const A: bool> Canisters<A> {
    pub async fn get_post_details(
        &self,
        user_canister: Principal,
        post_id: u64,
    ) -> Result<Option<PostDetails>> {
        let post_creator_can = self.individual_user(user_canister).await;
        let post_details = match post_creator_can
            .get_individual_post_details_by_id(post_id)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                log::warn!(
                    "failed to get post details for {} {}: {}, skipping",
                    user_canister.to_string(),
                    post_id,
                    e
                );
                return Ok(None);
            }
        };

        // TODO: temporary patch in frontend to not show banned videos, to be removed later after NSFW tagging
        if matches!(post_details.status, PostStatus::BannedDueToUserReporting) {
            return Ok(None);
        }

        let post_uuid = &post_details.video_uid;
        let req_url = format!(
            "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/{}/manifest/video.m3u8",
            post_uuid,
        );
        let res = reqwest::Client::default().head(req_url).send().await;
        if res.is_err() || (res.is_ok() && res.unwrap().status() != 200) {
            return Ok(None);
        }

        Ok(Some(PostDetails::from_canister_post(
            A,
            user_canister,
            post_details,
        )))
    }
}

impl Canisters<true> {
    pub async fn post_like_info(
        &self,
        post_canister: Principal,
        post_id: u64,
    ) -> Result<(bool, u64)> {
        let individual = self.individual_user(post_canister).await;
        let post = individual
            .get_individual_post_details_by_id(post_id)
            .await?;
        Ok((post.liked_by_me, post.like_count))
    }
}
