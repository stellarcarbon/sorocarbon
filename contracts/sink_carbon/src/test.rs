#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke}, 
    Address, IntoVal, String, Symbol,
};

use crate::fixtures::set_up_contracts_and_funder;

#[test]
fn test_quantize_to_kg() {
    let tons_with_remainder = 50_000_123_i64;
    let quantized_tons = quantize_to_kg(tons_with_remainder);
    assert_eq!(quantized_tons, 50_000_000_i64);

    let decigrams = 999_999_i64;
    let quantized_dgs = quantize_to_kg(decigrams);
    assert_eq!(quantized_dgs, 990_000_i64);
}

#[test]
fn test_sink_carbon_happy() {
    let setup = set_up_contracts_and_funder(10_000_000);
    let env = setup.env;
    let funder = setup.funder;
    let carbon_sac = setup.carbon_sac;
    let carbonsink_sac = setup.carbonsink_sac;
    let contract_id = setup.contract_id;

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

#[test]
fn test_sink_carbon_twice() {
    let setup = set_up_contracts_and_funder(10_000_000);
    let env = setup.env;
    let funder = setup.funder;
    let carbon_sac = setup.carbon_sac;
    let carbonsink_sac = setup.carbonsink_sac;
    let contract_id = setup.contract_id;

    // have the funder sink 0.3 CARBON
    let client = SinkContractClient::new(&env, &contract_id);
    let amount_a = 3_000_000_i64;
    let project_id = Symbol::new(&env,"VCS1360");
    let memo_a = String::from_str(&env,"300 kg 🌳🌴");
    let email = String::from_str(&env, "");
    client
        .mock_auths(&[MockAuth {
            address: &funder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "sink_carbon",
                args: (
                    funder.clone(), funder.clone(), amount_a, 
                    project_id.clone(), memo_a.clone(), email.clone()
                ).into_val(&env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &carbon_sac.address(),
                    fn_name: "burn",
                    args: (&funder, &3_000_000_i128).into_val(&env),
                    sub_invokes: &[],
                }],
            },
        }])
        .sink_carbon(
            &funder, &funder, &amount_a, &project_id, &memo_a, &email
        );

    // have the funder sink 0.1 CARBON
    let amount_b = 1_000_000_i64;
    let memo_b = String::from_str(&env,"100 kg 🌳🌴");
    client
        .mock_auths(&[MockAuth {
            address: &funder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "sink_carbon",
                args: (
                    funder.clone(), funder.clone(), amount_b, 
                    project_id.clone(), memo_b.clone(), email.clone()
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
            &funder, &funder, &amount_b, &project_id, &memo_b, &email
        );

    // assert the effect on balances
    let carbon_client = TokenClient::new(&env, &carbon_sac.address());
    let carbonsink_client = TokenClient::new(&env, &carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&funder), 6_000_000);
    assert_eq!(carbonsink_client.balance(&funder), 4_000_000);
}

#[test]
fn test_sink_carbon_separate_recipient() {
    let setup = set_up_contracts_and_funder(10_000_000);
    let env = setup.env;
    let funder = setup.funder;
    let recipient = Address::generate(&env);
    let carbon_sac = setup.carbon_sac;
    let carbonsink_sac = setup.carbonsink_sac;
    let contract_id = setup.contract_id;

    // have the funder sink 0.333 CARBON for the recipient
    let client = SinkContractClient::new(&env, &contract_id);
    let amount = 3_330_000_i64;
    let project_id = Symbol::new(&env,"OFP1234567890");
    let memo_text = String::from_str(&env,"333 kg 🍄‍🟫🍄🍄‍🟫");
    let email = String::from_str(&env, "");
    client
        .mock_auths(&[MockAuth {
            address: &funder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "sink_carbon",
                args: (
                    funder.clone(), recipient.clone(), amount, 
                    project_id.clone(), memo_text.clone(), email.clone()
                ).into_val(&env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &carbon_sac.address(),
                    fn_name: "burn",
                    args: (&funder, &3_330_000_i128).into_val(&env),
                    sub_invokes: &[],
                }],
            },
        }])
        .sink_carbon(
            &funder, &recipient, &amount, &project_id, &memo_text, &email
        );

    // assert the effect on balances
    let carbon_client = TokenClient::new(&env, &carbon_sac.address());
    let carbonsink_client = TokenClient::new(&env, &carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&funder), 6_670_000);
    assert_eq!(carbon_client.balance(&recipient), 0);
    assert_eq!(carbonsink_client.balance(&funder), 0);
    assert_eq!(carbonsink_client.balance(&recipient), 3_330_000);
}

#[test]
fn test_sink_amount_too_low() {
    let setup = set_up_contracts_and_funder(10_000_000);
    let env = setup.env;
    let funder = setup.funder;
    let carbon_sac = setup.carbon_sac;
    let carbonsink_sac = setup.carbonsink_sac;
    let contract_id = setup.contract_id;

    // attempt to sink 0.099 CARBON
    let client = SinkContractClient::new(&env, &contract_id);
    let amount = 990_000_i64;
    let project_id = Symbol::new(&env,"SOMEPROJECT");
    let memo_text = String::from_str(&env,"99 kg 🌳🌴");
    let email = String::from_str(&env, "");

    // it should fail because the amount is lower than the minimum
    assert!(
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
        .try_sink_carbon(
            &funder, &funder, &amount, &project_id, &memo_text, &email
        ).is_err()
    );

    // assert the lack of effect on balances
    let carbon_client = TokenClient::new(&env, &carbon_sac.address());
    let carbonsink_client = TokenClient::new(&env, &carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&funder), 10_000_000);
    assert_eq!(carbonsink_client.balance(&funder), 0);
}
