#![cfg(feature = "mercury")]

use retroshade_sdk::Retroshade;
use soroban_sdk::{contracttype, Address, String, Symbol};

#[derive(Retroshade)]
#[contracttype]
pub struct SinkEvent {
    pub funder: Address, 
    pub recipient: Address, 
    pub amount: i64, 
    pub project_id: Symbol,
    pub memo_text: String,
    pub email: String,
}
