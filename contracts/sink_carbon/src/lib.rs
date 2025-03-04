#![no_std]

use soroban_sdk::{
    contract, contractimpl, 
    token::{TokenClient, StellarAssetClient}, 
    Address, Env, String, Symbol,
};

use crate::storage_types::{DataKey, extend_instance_ttl};

#[contract]
pub struct SinkContract;

#[contractimpl]
impl SinkContract {
    pub fn __constructor(env: Env, admin: Address, carbon_id: Address, carbonsink_id: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CarbonID, &carbon_id);
        env.storage().instance().set(&DataKey::CarbonSinkID, &carbonsink_id);
        env.storage().instance().set(&DataKey::IsActive, &true);
    }

    pub fn sink_carbon(
        env: Env, 
        funder: Address, 
        recipient: Address, 
        amount: i64, 
        _project_id: Symbol,
        _memo_text: String,
        _email: String,
    ) {
        extend_instance_ttl(&env);

        // `funder` burns `amount` of CARBON
        funder.require_auth();
        let carbon_id = env.storage().instance().get(&DataKey::CarbonID).unwrap();
        let carbon_client = TokenClient::new(&env, &carbon_id);
        carbon_client.burn(&funder, &amount.into());

        // `recipient` receives `amount` of CarbonSINK
        let carbonsink_id = env.storage().instance().get(&DataKey::CarbonSinkID).unwrap();
        let carbonsink_client = StellarAssetClient::new(&env, &carbonsink_id);
        carbonsink_client.set_authorized(&recipient, &true);
        carbonsink_client.mint(&recipient, &amount.into());
        carbonsink_client.set_authorized(&recipient, &false);
    }
}

mod storage_types;
mod test;
mod fixtures;
