use std::borrow::Cow;

use candid::{ CandidType, Decode, Encode, Principal };
use ic_cdk::{
    api::{ management_canister::main::{ canister_status, CanisterIdRecord }, time },
    update,
};
use ic_ledger_types::{ BlockIndex, TransferError };
use ic_stable_structures::{ storable::Bound, Storable };
use icrc_ledger_types::icrc1::{ account::Account, transfer::{ NumTokens, TransferArg } };
use serde::{ Deserialize, Serialize };

use crate::TRANSFER_HISTORY;

#[derive(CandidType, Serialize, Clone, Deserialize)]
pub struct TransferToPrincipal {
    pub principal: Principal,
    pub amount: u64,
    pub ledger_id: String,
}

#[derive(CandidType, Serialize, Clone, Deserialize)]
pub struct TransferToMuliple {
    pub principals: Vec<PrincipalTransfer>,
    pub ledger_id: String,
}

#[derive(CandidType, Serialize, Clone, Deserialize)]
pub struct PrincipalTransfer {
    pub principal: Principal,
    pub amount: u64,
}

#[derive(CandidType, Serialize, Clone, Deserialize)]
pub enum TransferHistory {
    TransferToPrincipal(TransferToPrincipal),
    TransferToMultiple(TransferToMuliple),
}

impl Storable for TransferHistory {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[update]
pub async fn transfer_to_multiple(arg: TransferToMuliple) -> Result<(), String> {
    let caller = ic_cdk::caller();
    if !is_controller(caller).await {
        return Err("Caller is not a controller".to_string());
    }
    for principal in arg.principals.clone() {
        let transfer_amount_arg = TransferArg {
            to: Account {
                owner: principal.principal,
                subaccount: None,
            },
            fee: None,
            memo: None,
            from_subaccount: None,
            created_at_time: Some(time()),
            amount: NumTokens::from(principal.amount),
        };

        transfer_tokens(transfer_amount_arg, &arg.ledger_id).await?;
        let id = TRANSFER_HISTORY.with(|history| {
            let history = history.borrow();
            history.len() as u64
        });
        let transfer_history = TransferHistory::TransferToMultiple(arg.clone());
        TRANSFER_HISTORY.with(|history| {
            history.borrow_mut().insert(id + 1, transfer_history);
        });
    }
    Ok(())
}

#[update]
pub async fn transfer_to_principal(arg: TransferToPrincipal) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    if !is_controller(caller).await {
        return Err("Caller is not a controller".to_string());
    }

    let transfer_amount_arg = TransferArg {
        to: Account {
            owner: arg.principal,
            subaccount: None,
        },
        fee: None,
        memo: None,
        from_subaccount: None,
        created_at_time: Some(time()),
        amount: NumTokens::from(arg.amount),
    };

    let block_index = transfer_tokens(transfer_amount_arg, &arg.ledger_id).await?;
    let history_arg = TransferHistory::TransferToPrincipal(arg.clone());
    let id = TRANSFER_HISTORY.with(|history| {
        let history = history.borrow();
        history.len() as u64
    });
    
    TRANSFER_HISTORY.with(|history| {
        history.borrow_mut().insert(id + 1, history_arg);
    });
    Ok(block_index)
}

async fn transfer_tokens(arg: TransferArg, ledger_id: &str) -> Result<u64, String> {
    ic_cdk
        ::call::<(TransferArg,), (Result<BlockIndex, TransferError>,)>(
            Principal::from_text(ledger_id).expect("Could not decode the principal."),
            "icrc1_transfer",
            (arg,)
        ).await
        .map_err(|e| format!("failed to call ledger: {:?}", e))?
        .0.map_err(|e| format!("ledger transfer error {:?}", e))
}

async fn is_controller(principal: Principal) -> bool {
    let canister_id = ic_cdk::id();

    let result = canister_status(CanisterIdRecord { canister_id }).await;

    match result {
        Ok(status) => { status.0.settings.controllers.contains(&principal) }
        Err(error) => {
            let error_message = format!("{:?}", error);
            if error_message.contains(&principal.to_string()) {
                true
            } else {
                false
            }
        }
    }
}
