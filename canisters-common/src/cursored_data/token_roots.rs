use std::str::FromStr;

use candid::{Nat, Principal};
use canisters_client::{
    individual_user_template::Result15,
    sns_ledger::{self, SnsLedger},
};
use futures_util::{
    future,
    stream::{self, FuturesOrdered},
    StreamExt,
};
use grpc_traits::TokenInfoProvider;

use crate::{
    consts::SUPPORTED_NON_YRAL_TOKENS_ROOT,
    utils::token::{RootType, TokenMetadata},
    Canisters, Error, Result,
};

use super::{CursoredDataProvider, KeyedData, PageEntry};

impl KeyedData for RootType {
    type Key = RootType;

    fn key(&self) -> Self::Key {
        self.clone()
    }
}

#[derive(Clone)]
pub struct TokenRootList<TkInfo: TokenInfoProvider> {
    pub canisters: Canisters<false>,
    pub user_canister: Principal,
    pub user_principal: Principal,
    pub nsfw_detector: TkInfo,
}

async fn get_balance<'a>(user_principal: Principal, ledger: &SnsLedger<'a>) -> Option<Nat> {
    ledger
        .icrc_1_balance_of(sns_ledger::Account {
            owner: user_principal,
            subaccount: None,
        })
        .await
        .ok()
}

pub async fn eligible_non_yral_supported_tokens(
    cans: &Canisters<false>,
    nsfw_detector: &impl TokenInfoProvider,
    user_principal: Principal,
) -> Result<Vec<RootType>> {
    let tasks = SUPPORTED_NON_YRAL_TOKENS_ROOT
        .iter()
        .map(|&token_root| async move {
            let token_root = Principal::from_text(token_root).ok()?;
            let metadata_res = cans
                .token_metadata_by_root(nsfw_detector, Some(user_principal), token_root)
                .await
                .ok()?;
            Some((token_root, metadata_res))
        })
        .collect::<FuturesOrdered<_>>()
        .filter_map(|res| {
            let Some((
                token_root,
                Some(TokenMetadata {
                    balance: Some(balance),
                    ..
                }),
            )) = res
            else {
                return future::ready(None);
            };
            if balance
                .map_balance_ref(|b| b.e8s > 0u64)
                .unwrap_or_default()
            {
                return future::ready(Some(RootType::Other(token_root)));
            }

            future::ready(None)
        });

    Ok(tasks.collect().await)
}

impl<TkInfo: TokenInfoProvider + Send + Sync> CursoredDataProvider for TokenRootList<TkInfo> {
    type Data = RootType;
    type Error = Error;

    async fn get_by_cursor_inner(&self, start: usize, end: usize) -> Result<PageEntry<Self::Data>> {
        let user = self.canisters.individual_user(self.user_canister).await;
        let tokens = user
            .get_token_roots_of_this_user_with_pagination_cursor(start as u64, end as u64)
            .await?;
        let mut tokens: Vec<RootType> = match tokens {
            Result15::Ok(v) => v
                .into_iter()
                .map(|t| RootType::from_str(&t.to_text()).unwrap())
                .collect(),
            Result15::Err(_) => vec![],
        };

        let list_end = tokens.len() < (end - start);

        if start == 0 {
            let mut rep = stream::iter([
                RootType::from_str("btc").unwrap(),
                RootType::from_str("usdc").unwrap(),
            ])
            .filter_map(|root_type| async move {
                let cans = &self.canisters;

                match root_type {
                    RootType::BTC { ledger, .. } => {
                        let ledger = cans.sns_ledger(ledger).await;
                        let bal = get_balance(self.user_principal, &ledger).await?;

                        if bal != 0u64 {
                            Some(root_type)
                        } else {
                            None
                        }
                    }
                    RootType::USDC { ledger, .. } => {
                        let ledger = cans.sns_ledger(ledger).await;
                        let bal = get_balance(self.user_principal, &ledger).await?;

                        if bal != 0u64 {
                            Some(root_type)
                        } else {
                            None
                        }
                    }
                    _ => Some(root_type),
                }
            })
            .collect::<Vec<_>>()
            .await;

            rep.extend(
                eligible_non_yral_supported_tokens(
                    &self.canisters,
                    &self.nsfw_detector,
                    self.user_principal,
                )
                .await?,
            );
            tokens.splice(0..0, rep);
        }
        Ok(PageEntry {
            data: tokens,
            end: list_end,
        })
    }
}
