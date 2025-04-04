extern crate serde;
use icrc_ledger_types::icrc1::transfer::BlockIndex;
use api::updates::{TransferHistory, TransferToPrincipal, TransferToMuliple};
use ic_stable_structures::memory_manager::{ MemoryId, MemoryManager, VirtualMemory };
use ic_stable_structures::{ DefaultMemoryImpl, StableBTreeMap };
use std::cell::RefCell;

pub mod api;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static TRANSFER_HISTORY: RefCell<
        StableBTreeMap<u64, TransferHistory, VirtualMemory<DefaultMemoryImpl>>
    > = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))))
    );
}


ic_cdk::export_candid!();