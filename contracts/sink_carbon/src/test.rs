#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, IssuerFlags, MockAuth, MockAuthInvoke}, 
    Address, Env, IntoVal, String, Symbol,
};

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

    // give the funder an initial balance of 1 CARBON
    carbon_sac_client
        .mock_auths(&[MockAuth {
            address: &carbon_issuer,
            invoke: &MockAuthInvoke {
                contract: &carbon_sac.address(),
                fn_name: "mint",
                args: (&funder, &10_000_000_i128).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .mint(&funder, &10_000_000);

    // have the funder sink 0.1 CARBON
    let client = SinkContractClient::new(&env, &contract_id);
    let amount = 1_000_000_i64;
    let project_id = Symbol::new(&env,"VCS1360");
    let memo_text = String::from_str(&env,"100 kg 🌳🌴");
    let email = String::from_str(&env, "");
    client
        .mock_auths(&[MockAuth {
            address: &funder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "sink_carbon",
                args: (
                    funder.clone(), funder.clone(), amount, 
                    project_id.clone(), memo_text.clone(), email.clone()
                ).into_val(&env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &carbon_sac.address(),
                    fn_name: "burn",
                    args: (&funder, &1_000_000_i128).into_val(&env),
                    sub_invokes: &[],
                }],
            },
        }])
        .sink_carbon(
            &funder, &funder, &amount, &project_id, &memo_text, &email
        );

    // assert the effect on balances
    let carbon_client = TokenClient::new(&env, &carbon_sac.address());
    let carbonsink_client = TokenClient::new(&env, &carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&funder), 9_000_000);
    assert_eq!(carbonsink_client.balance(&funder), 1_000_000);

}
