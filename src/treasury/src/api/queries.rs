use ic_cdk::query;

use crate::TRANSFER_HISTORY;

#[query]
pub fn get_transfer_history() -> u64 {
    TRANSFER_HISTORY.with(|history| {
        let history = history.borrow();
        history.len() as u64
    })
}