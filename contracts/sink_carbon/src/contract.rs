use soroban_sdk::{
    contract, contractimpl, 
    token::{TokenClient, StellarAssetClient}, 
    Address, Env, String, Symbol,
};

use crate::storage_types::{DataKey, extend_instance_ttl, set_is_active};
use crate::utils::quantize_to_kg;

#[contract]
pub struct SinkContract;

#[contractimpl]
impl SinkContract {
    pub fn __constructor(env: Env, admin: Address, carbon_id: Address, carbonsink_id: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CarbonID, &carbon_id);
        env.storage().instance().set(&DataKey::CarbonSinkID, &carbonsink_id);
        env.storage().instance().set(&DataKey::IsActive, &true);
        env.storage().instance().set(&DataKey::SinkMinimum, &1_000_000_i64);  // 100 kg
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
        if !Self::is_active(env.clone()) {
            panic!("SinkContract has been deactivated");
        }

        // quantize `amount` to kg resolution
        let amount = quantize_to_kg(amount);

        // check if `amount` equals or exceeds minimum
        let minimum_sink_amount: i64 = env.storage().instance().get(&DataKey::SinkMinimum).unwrap();
        if amount < minimum_sink_amount {
            panic!("sink amount is smaller than minimum");
        }

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

    pub fn get_minimum_sink_amount(env: Env) -> i64 {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::SinkMinimum).unwrap()
    }

    pub fn is_active(env: Env) -> bool {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::IsActive).unwrap()
    }

    // ADMIN FUNCTIONS

    pub fn set_minimum_sink_amount(env: Env, amount: i64) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::SinkMinimum, &amount);
    }

    pub fn reset_admin(env: Env) -> Address {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        // set contract admin as the CarbonSINK SAC admin
        let carbonsink_id = env.storage().instance().get(&DataKey::CarbonSinkID).unwrap();
        let carbonsink_client = StellarAssetClient::new(&env, &carbonsink_id);
        carbonsink_client.set_admin(&admin);

        // deactivate this contract
        set_is_active(&env, false);

        admin
    }

    pub fn activate(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        set_is_active(&env, true);
    }

    pub fn deactivate(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        set_is_active(&env, false);
    }
}
