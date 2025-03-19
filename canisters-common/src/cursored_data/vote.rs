use candid::Principal;
use canisters_client::individual_user_template::PlacedBetDetail;

use crate::{utils::vote::VoteDetails, Canisters, Error};

use super::{CursoredDataProvider, KeyedData, PageEntry};

impl KeyedData for VoteDetails {
    type Key = (Principal, u64);

    fn key(&self) -> Self::Key {
        (self.canister_id, self.post_id)
    }
}

/// Note that this can only provide details for
/// 10 votes at a time
/// any less or any more, the fetching will panic
#[derive(Clone)]
pub struct VotesProvider {
    canisters: Canisters<false>,
    user: Principal,
}

impl VotesProvider {
    pub fn new(canisters: Canisters<false>, user: Principal) -> Self {
        Self { canisters, user }
    }
}

impl CursoredDataProvider for VotesProvider {
    type Data = VoteDetails;
    type Error = Error;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<VoteDetails>, Error> {
        let user = self.canisters.individual_user(self.user).await;
        assert_eq!(end - start, 10);
        let bets = user
            .get_hot_or_not_bets_placed_by_this_profile_with_pagination(start as u64)
            .await?;
        let list_end = bets.len() < (end - start);
        Ok(PageEntry {
            data: bets.into_iter().map(PlacedBetDetail::into).collect(),
            end: list_end,
        })
    }
}
