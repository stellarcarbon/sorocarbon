#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, IssuerFlags, Ledger, MockAuth, MockAuthInvoke, StellarAssetContract},
    token::StellarAssetClient,
    Address, Env, IntoVal,
};

use crate::contract::{SinkContract, SinkContractClient};

pub struct Setup<'a> {
    pub env: Env,
    pub funder: Address,
    pub carbon_sac: StellarAssetContract,
    pub carbonsink_issuer: Address,
    pub carbonsink_sac: StellarAssetContract,
    pub contract_id: Address,
    pub sink_client: SinkContractClient<'a>,
}

pub fn set_up_contracts_and_funder<'a>(funder_balance: i128, env_opt: Option<Env>) -> Setup<'a> {
    let env = env_opt.unwrap_or_default();
    let funder = Address::generate(&env);
    let carbon_issuer = Address::generate(&env);  // this is a C-address
    let carbonsink_issuer = Address::generate(&env);  // this is a C-address
    let carbon_sac = env.register_stellar_asset_contract_v2(carbon_issuer.clone());
    let carbonsink_sac = env.register_stellar_asset_contract_v2(carbonsink_issuer.clone());
    // WARNING: carbon_sac.issuer().address() is a G-address (some conversion by testutils)
    carbonsink_sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    carbonsink_sac.issuer().set_flag(IssuerFlags::RequiredFlag);

    // set CarbonSINK issuer as the sink contract admin
    let contract_id = env.register(
        SinkContract, 
        (&carbonsink_issuer, &carbon_sac.address(), &carbonsink_sac.address())
    );
    let carbon_sac_client = StellarAssetClient::new(&env, &carbon_sac.address());
    let carbonsink_sac_client = StellarAssetClient::new(&env, &carbonsink_sac.address());
    // set the sink contract as the CarbonSINK SAC admin
    carbonsink_sac_client
        .mock_auths(&[MockAuth {
            address: &carbonsink_issuer,
            invoke: &MockAuthInvoke {
                contract: &carbonsink_sac.address(),
                fn_name: "set_admin",
                args: (&contract_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_admin(&contract_id);

    // give the funder an initial balance of `funder_balance` CARBON
    carbon_sac_client
        .mock_auths(&[MockAuth {
            address: &carbon_issuer,
            invoke: &MockAuthInvoke {
                contract: &carbon_sac.address(),
                fn_name: "mint",
                args: (&funder, &funder_balance).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .mint(&funder, &funder_balance);

    let sink_client = SinkContractClient::new(&env, &contract_id);

    Setup {
        env, funder, carbon_sac, carbonsink_issuer, carbonsink_sac, contract_id, sink_client,
    }
}

pub fn create_env_with_ttl_settings(max_entry_ttl: u32) -> Env {
    let env = Env::default();
    env.ledger().with_mut(|li| {
        // Current ledger sequence
        li.sequence_number = 100_000;
        // Minimum TTL for new instance and persistent entries
        // set to a value that avoids auth entry issues
        li.min_persistent_entry_ttl = 35_001;
        // Minimum TTL for new temporary entries
        li.min_temp_entry_ttl = 1_001;
        // Maximum TTL of any entry
        li.max_entry_ttl = max_entry_ttl;
    });
    env
}

pub fn set_up_contracts_with_ttl_settings<'a>(max_entry_ttl: u32) -> Setup<'a> {
    let env = create_env_with_ttl_settings(max_entry_ttl);
    let setup = set_up_contracts_and_funder(10_000_000, Some(env));
    setup
}
