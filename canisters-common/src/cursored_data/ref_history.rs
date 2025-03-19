use candid::Principal;
use ic_agent::AgentError;

use crate::Canisters;

use super::{CursoredDataProvider, KeyedData, PageEntry};
use canisters_client::individual_user_template::{MintEvent, Result17, TokenEvent};

#[derive(Clone, Copy)]
pub struct HistoryDetails {
    pub epoch_secs: u64,
    pub referee: Principal,
    pub amount: u64,
}

impl KeyedData for HistoryDetails {
    type Key = Principal;

    fn key(&self) -> Self::Key {
        self.referee
    }
}

#[derive(Clone)]
pub struct ReferralHistory(pub Canisters<true>);

impl CursoredDataProvider for ReferralHistory {
    type Data = HistoryDetails;
    type Error = AgentError;

    async fn get_by_cursor_inner(
        &self,
        from: usize,
        end: usize,
    ) -> Result<PageEntry<HistoryDetails>, AgentError> {
        let individual = self.0.authenticated_user().await;
        let history = individual
            .get_user_utility_token_transaction_history_with_pagination(from as u64, end as u64)
            .await?;
        let history = match history {
            Result17::Ok(history) => history,
            Result17::Err(_) => {
                log::warn!("failed to get posts");
                return Ok(PageEntry {
                    data: vec![],
                    end: true,
                });
            }
        };
        let list_end = history.len() < (end - from);
        let details = history
            .into_iter()
            .filter_map(|(_, ev)| {
                let TokenEvent::Mint {
                    timestamp,
                    details:
                        MintEvent::Referral {
                            referee_user_principal_id,
                            ..
                        },
                    amount,
                } = ev
                else {
                    return None;
                };
                Some(HistoryDetails {
                    epoch_secs: timestamp.secs_since_epoch,
                    referee: referee_user_principal_id,
                    amount,
                })
            })
            .collect();
        Ok(PageEntry {
            data: details,
            end: list_end,
        })
    }
}
