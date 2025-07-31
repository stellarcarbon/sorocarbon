#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    token::{StellarAssetClient, TokenClient}, 
    Address, Bytes, Env, IntoVal, String, Symbol, Val, Vec
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
pub struct TestSinkContract;

#[contractimpl]
impl TestSinkContract {
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

        // call the SinkContract constructor
        let constructor_args: Vec<Val> = (admin_addr, carbon_addr, csink_addr).into_val(&env);
        let _: () = env.invoke_contract(&sink_addr, &Symbol::new(&env, "__constructor"), constructor_args);
    }
  
    pub fn test_active(env: Env) -> bool {
        let sink_addr: Address = env.storage().instance().get(&DataKey::SinkID).unwrap();
        let sink_client = sink_contract::Client::new(&env, &sink_addr);

        // call the `is_active` method of the sink contract
        let active = sink_client.is_active();
      
        // check if the constructor has been succesfully called
        active == true
    }

    pub fn test_swap_is_atomic(
        env: Env,
        funder: Address, 
        recipient: Address, 
        amount: i64, 
        project_id: Symbol,
        // TODO: String param, check blanket try...except
    ) -> bool {
        // bail if `amount` is not valid for mint
        // TODO: check on `assume` status in komet#74
        if 1 > amount {
            return true;
        }
        // create SAC token clients
        let carbon_addr = env.storage().instance().get(&DataKey::CarbonID).expect(
            "CARBON SAC address must be set."
        );
        let csink_addr = env.storage().instance().get(&DataKey::CarbonSinkID).expect(
            "CarbonSINK SAC address must be set."
        );
        let carbon_token_client = TokenClient::new(&env, &carbon_addr);
        let csink_token_client = TokenClient::new(&env, &csink_addr);
        
        // credit the funder with exactly `amount` of CARBON
        let carbon_sac_client = StellarAssetClient::new(&env, &csink_addr);
        carbon_sac_client.mint(&funder, &amount.into());

        // create the SinkContract client
        let sink_addr: Address = env.storage().instance().get(&DataKey::SinkID).unwrap();
        let sink_client = sink_contract::Client::new(&env, &sink_addr);

        // collect balances before the swap
        let contract_carbon_before = carbon_token_client.balance(&sink_addr);
        let contract_csink_before = csink_token_client.balance(&sink_addr);
        let funder_carbon_before = carbon_token_client.balance(&funder);
        let funder_csink_before = csink_token_client.balance(&funder);
        let recipient_carbon_before = carbon_token_client.balance(&recipient);
        let recipient_csink_before = csink_token_client.balance(&recipient);

        // Call the `sink_carbon` method of the sink contract
        let empty_string = String::from_str(&env, "");
        let sink_res = sink_client.try_sink_carbon(
            &funder, &recipient, &amount, &project_id, &empty_string, &empty_string
        );
        // TODO: check for SinkError::AmountTooLow

        // collect balances after the swap
        let contract_carbon_after = carbon_token_client.balance(&sink_addr);
        let contract_csink_after = csink_token_client.balance(&sink_addr);
        let funder_carbon_after = carbon_token_client.balance(&funder);
        let funder_csink_after = csink_token_client.balance(&funder);
        let recipient_carbon_after = carbon_token_client.balance(&recipient);
        let recipient_csink_after = csink_token_client.balance(&recipient);        

        // assert all contract balances are empty
        let contract_balances = [
            contract_carbon_before, contract_csink_before, 
            contract_carbon_after, contract_csink_after
        ];
        let max_balance = contract_balances.iter().max().unwrap();
        if *max_balance > 0 { return false; }

        // assert the effect on funder balances
        if funder_carbon_before != amount as i128 { return false; }
        if funder_carbon_after >= 10_000 { return false; }
        if funder_csink_before != 0 { return false; }
        if funder_csink_after != 0 { return false; }

        // assert the effect on recipient balances
        if recipient_carbon_before != 0 { return false; }
        if recipient_carbon_after != 0 { return false; }
        if recipient_csink_before != 0 { return false; }
        let quantization_remainder = amount as i128 - recipient_csink_after;
        if quantization_remainder >= 10_000 { return false; }
      
        // assert quantization remainders are equal
        funder_carbon_after == quantization_remainder
    }
}
