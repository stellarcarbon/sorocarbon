#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _, token::TokenClient, Address,
};

use crate::tests::fixtures::set_up_contracts_and_funder;
use crate::tests::utils::{SinkTestData, sink_carbon_with_auth};
use crate::utils::quantize_to_kg;

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

    // have the funder sink 0.1 CARBON
    let test_data = SinkTestData { 
        recipient: &setup.funder,
        amount: 1_000_000_i64,
        project_id: "VCS1360",
        memo_text: "100 kg 🌳🌴",
        email: ""
    };
    assert!(sink_carbon_with_auth(&setup, &test_data).is_ok());

    // assert the effect on balances
    let carbon_client = TokenClient::new(&setup.env, &setup.carbon_sac.address());
    let carbonsink_client = TokenClient::new(&setup.env, &setup.carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&setup.funder), 9_000_000);
    assert_eq!(carbonsink_client.balance(&setup.funder), 1_000_000);
}

#[test]
fn test_sink_carbon_twice() {
    let setup = set_up_contracts_and_funder(10_000_000);

    // have the funder sink 0.3 CARBON
    let test_data_a = SinkTestData { 
        recipient: &setup.funder,
        amount: 3_000_000_i64,
        project_id: "VCS1360",
        memo_text: "300 kg 🌳🌴",
        email: ""
    };
    assert!(sink_carbon_with_auth(&setup, &test_data_a).is_ok());

    // have the funder sink 0.1 CARBON
    let mut test_data_b = test_data_a.clone();
    test_data_b.amount = 1_000_000_i64;
    test_data_b.memo_text = "100 kg 🌳🌴";
    assert!(sink_carbon_with_auth(&setup, &test_data_b).is_ok());

    // assert the effect on balances
    let carbon_client = TokenClient::new(&setup.env, &setup.carbon_sac.address());
    let carbonsink_client = TokenClient::new(&setup.env, &setup.carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&setup.funder), 6_000_000);
    assert_eq!(carbonsink_client.balance(&setup.funder), 4_000_000);
}

#[test]
fn test_sink_carbon_separate_recipient() {
    let setup = set_up_contracts_and_funder(10_000_000);

    // have the funder sink 0.333 CARBON for the recipient
    let test_data = SinkTestData { 
        recipient: &Address::generate(&setup.env),
        amount: 3_330_000_i64,
        project_id: "OFP1234567890",
        memo_text: "333 kg 🍄‍🟫🍄🍄‍🟫",
        email: ""
    };
    assert!(sink_carbon_with_auth(&setup, &test_data).is_ok());

    // assert the effect on balances
    let carbon_client = TokenClient::new(&setup.env, &setup.carbon_sac.address());
    let carbonsink_client = TokenClient::new(&setup.env, &setup.carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&setup.funder), 6_670_000);
    assert_eq!(carbon_client.balance(test_data.recipient), 0);
    assert_eq!(carbonsink_client.balance(&setup.funder), 0);
    assert_eq!(carbonsink_client.balance(test_data.recipient), 3_330_000);
}

#[test]
fn test_sink_amount_too_low() {
    let setup = set_up_contracts_and_funder(10_000_000);

    // attempt to sink 0.099 CARBON
    let test_data = SinkTestData { 
        recipient: &setup.funder,
        amount: 990_000_i64,
        project_id: "SOMEPROJECT",
        memo_text: "99 kg 🌳🌴",
        email: ""
    };
    // it should fail because the amount is lower than the minimum
    assert!(sink_carbon_with_auth(&setup, &test_data).is_err());

    // assert the lack of effect on balances
    let carbon_client = TokenClient::new(&setup.env, &setup.carbon_sac.address());
    let carbonsink_client = TokenClient::new(&setup.env, &setup.carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&setup.funder), 10_000_000);
    assert_eq!(carbonsink_client.balance(&setup.funder), 0);
}
