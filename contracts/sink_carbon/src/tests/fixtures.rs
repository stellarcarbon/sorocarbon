#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, IssuerFlags, MockAuth, MockAuthInvoke, StellarAssetContract},
    token::StellarAssetClient,
    Address, Env, IntoVal,
};

use crate::contract::SinkContract;

pub struct Setup {
    pub env: Env,
    pub funder: Address,
    pub carbon_sac: StellarAssetContract,
    pub carbonsink_sac: StellarAssetContract,
    pub contract_id: Address,
}

pub fn set_up_contracts_and_funder(funder_balance: i128) -> Setup {
    let env = Env::default();
    let funder = Address::generate(&env);
    let carbon_issuer = Address::generate(&env);
    let carbonsink_issuer = Address::generate(&env);
    let carbon_sac = env.register_stellar_asset_contract_v2(carbon_issuer.clone());
    let carbonsink_sac = env.register_stellar_asset_contract_v2(carbonsink_issuer.clone());
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

    Setup {env, funder, carbon_sac, carbonsink_sac, contract_id}
}
