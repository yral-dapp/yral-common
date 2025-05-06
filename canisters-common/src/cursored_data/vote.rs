use std::sync::Mutex;

use candid::Principal;
use canisters_client::individual_user_template::PlacedBetDetail;
use hon_worker_common::{GameRes, PaginatedGamesReq, PaginatedGamesRes, WORKER_URL};

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

/// Note that this can only provide details for
/// 10 votes at a time
/// any less or any more, the fetching will panic
#[derive(Clone)]
pub struct VotesWithCentsProvider {
    canisters: Canisters<false>,
    user: Principal,
}

impl VotesWithCentsProvider {
    pub fn new(canisters: Canisters<false>, user: Principal) -> Self {
        Self { canisters, user }
    }
}

impl CursoredDataProvider for VotesWithCentsProvider {
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
            .get_hot_or_not_bets_placed_by_this_profile_with_pagination_v_1(start as u64)
            .await?;
        let list_end = bets.len() < (end - start);
        Ok(PageEntry {
            data: bets.into_iter().map(PlacedBetDetail::into).collect(),
            end: list_end,
        })
    }
}

impl KeyedData for GameRes {
    type Key = (Principal, u64);

    fn key(&self) -> Self::Key {
        (self.post_canister, self.post_id)
    }
}

/// Only goes forward and ignores start and end parameters when paginating
///
/// UB: Retrieving next page while the current page hasn't finished loading will lead to undefine behavior
pub struct VotesWithSatsProvider {
    // Mutex because we need to track next internally without mut ref.
    next: Mutex<Option<String>>,
    user_principal: Principal,
}

// impl clone by hand because Mutex<T> doesn't impl clone on its own
impl Clone for VotesWithSatsProvider {
    fn clone(&self) -> Self {
        let lock = self.next.lock().unwrap();
        let next = lock.clone();
        let user_principal = self.user_principal;

        Self {
            next: Mutex::new(next),
            user_principal,
        }
    }
}

impl CursoredDataProvider for VotesWithSatsProvider {
    type Data = GameRes;
    type Error = Error;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<Self::Data>, Self::Error> {
        let path = format!("/games/{}", self.user_principal);
        let url = WORKER_URL.join(&path).unwrap();
        let cursor = self.get_cursor();
        let req = PaginatedGamesReq {
            page_size: end - start,
            cursor,
        };

        let client = reqwest::Client::new();
        let PaginatedGamesRes { games, next }: PaginatedGamesRes =
            client.get(url).json(&req).send().await?.json().await?;

        let end = next.is_none();

        *self.next.lock().unwrap() = next;

        Ok(PageEntry { data: games, end })
    }
}

impl VotesWithSatsProvider {
    pub fn new(user_principal: Principal) -> Self {
        Self {
            user_principal,
            next: Mutex::new(None),
        }
    }

    fn get_cursor(&self) -> Option<String> {
        self.next.lock().unwrap().clone()
    }
}
