use std::io::Cursor;

use candid::{Nat, Principal};
use canisters_client::{
    sns_index::{self, GetAccountTransactionsArgs, GetTransactionsResult},
    sns_ledger,
};
use ic_agent::hash_tree::{HashTree, LookupResult};

use crate::{
    utils::{
        token::balance::TokenBalance,
        transaction::{TxnInfoType, TxnInfoWallet},
    },
    Canisters, Error, Result,
};

use super::{CursoredDataProvider, KeyedData, PageEntry};

impl KeyedData for TxnInfoWallet {
    type Key = u64;

    fn key(&self) -> Self::Key {
        self.id
    }
}

#[derive(Clone, Copy)]
pub enum IndexOrLedger {
    Index {
        key_principal: Principal,
        index: Principal,
    },
    Ledger(Principal),
}

#[derive(Clone)]
pub struct TxnHistory {
    pub canisters: Canisters<false>,
    pub source: IndexOrLedger,
    pub decimals: u8,
}

impl TxnHistory {
    fn parse_transactions_ledger(
        txn: sns_ledger::Transaction,
        id: u64,
        decimals: u8,
    ) -> Result<TxnInfoWallet> {
        let timestamp = txn.timestamp;

        match txn {
            sns_ledger::Transaction {
                mint: Some(mint), ..
            } => Ok(TxnInfoWallet {
                tag: TxnInfoType::Mint { to: mint.to.owner },
                timestamp,
                amount: TokenBalance::new(mint.amount, decimals),
                id,
            }),
            sns_ledger::Transaction {
                burn: Some(burn), ..
            } => Ok(TxnInfoWallet {
                tag: TxnInfoType::Burn {
                    from: burn.from.owner,
                },
                timestamp,
                amount: TokenBalance::new(burn.amount, decimals),
                id,
            }),
            sns_ledger::Transaction {
                transfer: Some(transfer),
                ..
            } => Ok(TxnInfoWallet {
                tag: TxnInfoType::Transfer {
                    from: transfer.from.owner,
                    to: transfer.to.owner,
                },
                timestamp,
                amount: TokenBalance::new(transfer.amount, decimals),
                id,
            }),
            _ => Err(Error::ParseTransaction),
        }
    }

    fn parse_transactions_index(
        txn: sns_index::TransactionWithId,
        user_principal: Principal,
        decimals: u8,
    ) -> Result<TxnInfoWallet> {
        let timestamp = txn.transaction.timestamp;
        let id = txn.id.0.to_u64_digits()[0];

        match txn.transaction {
            sns_index::Transaction {
                mint: Some(mint), ..
            } => Ok(TxnInfoWallet {
                tag: TxnInfoType::Mint { to: mint.to.owner },
                timestamp,
                amount: TokenBalance::new(mint.amount, decimals),
                id,
            }),
            sns_index::Transaction {
                burn: Some(burn), ..
            } => Ok(TxnInfoWallet {
                tag: TxnInfoType::Burn {
                    from: user_principal,
                },
                timestamp,
                amount: TokenBalance::new(burn.amount, decimals),
                id,
            }),
            sns_index::Transaction {
                transfer: Some(transfer),
                ..
            } => {
                if user_principal == transfer.from.owner {
                    // User is sending funds
                    Ok(TxnInfoWallet {
                        tag: TxnInfoType::Sent {
                            to: transfer.to.owner,
                        },
                        timestamp,
                        amount: TokenBalance::new(transfer.amount, decimals),
                        id,
                    })
                } else if user_principal == transfer.to.owner {
                    // User is receiving funds
                    Ok(TxnInfoWallet {
                        tag: TxnInfoType::Received {
                            from: transfer.from.owner,
                        },
                        timestamp,
                        amount: TokenBalance::new(transfer.amount, decimals),
                        id,
                    })
                } else {
                    Err(Error::ParseTransaction)
                }
            }
            _ => Err(Error::ParseTransaction),
        }
    }

    async fn get_by_cursor_index(
        &self,
        key_principal: Principal,
        index: Principal,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<TxnInfoWallet>> {
        let index_canister = self.canisters.sns_index(index).await;

        // Fetch transactions up to the 'end' index
        let max_results = end; // Fetch enough transactions to cover 'end'

        let history = index_canister
            .get_account_transactions(GetAccountTransactionsArgs {
                max_results: Nat::from(max_results),
                start: None, // No cursor, fetch the latest transactions
                account: sns_index::Account {
                    owner: key_principal,
                    subaccount: None,
                },
            })
            .await?;

        let transactions = match history {
            GetTransactionsResult::Ok(v) => v.transactions,
            GetTransactionsResult::Err(e) => {
                return Err(Error::GetTransactions(e.message));
            }
        };

        let transactions = transactions.into_iter().skip(start).take(end - start);
        let txns_len = transactions.len();
        let data: Vec<TxnInfoWallet> = transactions
            .filter_map(|txn| {
                Self::parse_transactions_index(txn, key_principal, self.decimals).ok()
            })
            .collect();

        let is_end = txns_len < (end - start);

        Ok(PageEntry { data, end: is_end })
    }

    async fn get_latest_ledger_transaction(&self, ledger_id: Principal) -> Result<u64> {
        let ledger = self.canisters.sns_ledger(ledger_id).await;
        let tip_certificate = ledger
            .icrc_3_get_tip_certificate()
            .await?
            .ok_or(Error::TipCertificate)?;

        let cursor = Cursor::new(&tip_certificate.hash_tree);

        let hash_tree: HashTree<Vec<u8>> = ciborium::from_reader(cursor)?;

        let lookup_path = [b"last_block_index"];
        let LookupResult::Found(res) = hash_tree.lookup_path(lookup_path) else {
            return Err(Error::TipCertificate);
        };

        let last_block_index =
            u64::from_be_bytes(res.try_into().map_err(|_| Error::TipCertificate)?);
        Ok(last_block_index)
    }

    async fn get_by_cursor_ledger(
        &self,
        ledger_id: Principal,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<TxnInfoWallet>> {
        let total_transactions = self.get_latest_ledger_transaction(ledger_id).await?;

        let start_index = if total_transactions > end as u64 {
            total_transactions - end as u64
        } else {
            0
        };

        let length = if total_transactions > start as u64 {
            total_transactions - start_index - start as u64
        } else {
            total_transactions - start_index
        };

        let ledger = self.canisters.sns_ledger(ledger_id).await;

        let history = ledger
            .get_transactions(sns_ledger::GetTransactionsRequest {
                start: start_index.into(),
                length: length.into(),
            })
            .await?;

        let list_end = start_index == 0;

        Ok(PageEntry {
            data: history
                .transactions
                .into_iter()
                .enumerate()
                .filter_map(|(i, txn)| {
                    let idx = (history.first_index.clone() + i).0.to_u64_digits();
                    if idx.is_empty() {
                        None
                    } else {
                        Self::parse_transactions_ledger(txn, idx[0], self.decimals).ok()
                    }
                })
                .rev()
                .collect(),
            end: list_end,
        })
    }
}

impl CursoredDataProvider for TxnHistory {
    type Data = TxnInfoWallet;
    type Error = Error;

    async fn get_by_cursor_inner(&self, start: usize, end: usize) -> Result<PageEntry<Self::Data>> {
        match self.source {
            IndexOrLedger::Ledger(ledger_id) => {
                self.get_by_cursor_ledger(ledger_id, start, end).await
            }
            IndexOrLedger::Index {
                key_principal,
                index,
            } => {
                self.get_by_cursor_index(key_principal, index, start, end)
                    .await
            }
        }
    }
}
