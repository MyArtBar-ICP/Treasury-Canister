use ic_cdk::query;

use crate::TRANSFER_HISTORY;

use super::updates::TransferHistory;

#[query]
pub fn get_transfer_history() -> Vec<TransferHistory> {
    TRANSFER_HISTORY.with(|history| {
        history.borrow().iter().map(|(_, v)| v.clone()).collect::<Vec<TransferHistory>>()
    })
}