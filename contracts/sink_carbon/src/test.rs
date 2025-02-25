#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, IssuerFlags}, Address, Env, String, Symbol};

#[test]
fn test_sink_carbon_happy() {
    let env = Env::default();
    let funder = Address::generate(&env);
    let carbon_issuer = Address::generate(&env);
    let carbonsink_issuer = Address::generate(&env);
    let carbon_sac = env.register_stellar_asset_contract_v2(carbon_issuer.clone());
    let carbonsink_sac = env.register_stellar_asset_contract_v2(carbonsink_issuer.clone());
    carbonsink_sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    carbonsink_sac.issuer().set_flag(IssuerFlags::RequiredFlag);

    // set CarbonSINK issuer as the contract admin
    let contract_id = env.register(
        SinkContract, 
        (&carbonsink_issuer, &carbon_sac.address(), &carbonsink_sac.address())
    );
    let carbon_sac_client = StellarAssetClient::new(&env, &carbon_sac.address());
    let carbonsink_sac_client = StellarAssetClient::new(&env, &carbonsink_sac.address());
    // set the contract as the CarbonSINK SAC admin
    // TODO: authorize this call as the CarbonSINK issuer
    carbonsink_sac_client.set_admin(&contract_id);

    // give the funder an initial balance of 1 CARBON
    carbon_sac_client.mint(&funder, &10_000_000);

    // have the funder sink 0.1 CARBON
    let client = SinkContractClient::new(&env, &contract_id);
    client.sink_carbon(
        &funder, 
        &funder, 
        &1_000_000, 
        &Symbol::new(&env,"VCS1360"), 
        &String::from_str(&env,"100 kg 🌳🌴"), 
        &String::from_str(&env, "")
    );

}
