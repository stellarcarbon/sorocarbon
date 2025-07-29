#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, 
    Address, Bytes, BytesN, Env, IntoVal, Val, Vec
};

mod komet;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    SinkID,
    Admin,
    CarbonID,
    CarbonSinkID,
}

mod sink_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/sink_carbon.wasm"
    );
}

#[contract]
pub struct TestAdderContract;

#[contractimpl]
impl TestAdderContract {
    pub fn init(env: Env, wasm_hash: Bytes) {
        let sink_bytes = b"sink_ctr________________________";
        let sink_addr = komet::create_contract(&env, &Bytes::from_array(&env, sink_bytes), &wasm_hash);
        env.storage().instance().set(&DataKey::SinkID, &sink_addr);

        let admin_bytes = b"GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJXFF";
        let admin_addr = komet::address_from_bytes(&env, admin_bytes, false);
        env.storage().instance().set(&DataKey::Admin, &admin_addr);
        let carbon_bytes = b"CCABDO7UZXYE4W6GVSEGSNNZTKSLFQGKXXQTH6OX7M7GKZ4Z6CUJNGZN";
        let carbon_addr = komet::address_from_bytes(&env, carbon_bytes, true);
        env.storage().instance().set(&DataKey::CarbonID, &carbon_addr);
        let csink_bytes = b"CDLDVFKHEZ2RVB3NG4UQA4VPD3TSHV6XMHXMHP2BSGCJ2IIWVTOHGDSG";
        let csink_addr = komet::address_from_bytes(&env, csink_bytes, true);
        env.storage().instance().set(&DataKey::CarbonSinkID, &csink_addr);

        let salt = BytesN::from_array(&env, &[0; 32]);
        let constructor_args: Vec<Val> = (admin_addr, carbon_addr, csink_addr).into_val(&env);
        let deployed_address = env
            .deployer()
            .with_address(env.current_contract_address(), salt)
            .deploy_v2(
                BytesN::<32>::from_array(&env, &wasm_hash.try_into().expect("wasm_hash must be 32 bytes")), 
                constructor_args
            );
    }

  
    pub fn test_active(env: Env) -> bool {
        let sink_addr: Address = env.storage().instance().get(&DataKey::SinkID).unwrap();
        let sink_client = sink_contract::Client::new(&env, &sink_addr);
        // let setup = komet::set_up_contracts_and_funder(100_000_000_i128, Some(env));
        // let client = setup.sink_client;
        // let admin = setup.carbonsink_issuer;
        // let carbonsink_sac = setup.carbonsink_sac;

        // Call the `is_active` method of the sink contract
        let active = sink_client.is_active();
      
        // Check if the constructor has been succesfully called
        active == true
    }
}
