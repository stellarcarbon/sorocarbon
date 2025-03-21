#![cfg(test)]

use soroban_sdk::{token, Address, String, Symbol};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::TokenClient;

use crate::errors::{SACError, SinkError};
use crate::tests::fixtures::set_up_contracts_and_funder;
use crate::tests::utils::{
    SinkTestData, create_account_entry, create_trustline, deploy_native_sac, sink_carbon_with_auth
};
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
        funder: &setup.funder,
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
        funder: &setup.funder,
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
        funder: &setup.funder,
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
        funder: &setup.funder, 
        recipient: &setup.funder,
        amount: 990_000_i64,
        project_id: "SOMEPROJECT",
        memo_text: "99 kg 🌳🌴",
        email: ""
    };
    // it should fail because the amount is lower than the minimum
    let sink_res = sink_carbon_with_auth(&setup, &test_data);
    assert_eq!(sink_res.unwrap_err(), SinkError::AmountTooLow);

    // assert the lack of effect on balances
    let carbon_client = TokenClient::new(&setup.env, &setup.carbon_sac.address());
    let carbonsink_client = TokenClient::new(&setup.env, &setup.carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&setup.funder), 10_000_000);
    assert_eq!(carbonsink_client.balance(&setup.funder), 0);
}

#[test]
#[should_panic = "ContractDeactivated"]
fn test_sink_contract_inactive() {
    let setup = set_up_contracts_and_funder(10_000_000);
    setup.sink_client.mock_all_auths().deactivate();

    // attempt to sink 1 CARBON
    let test_data = SinkTestData { 
        funder: &setup.funder,
        recipient: &setup.funder,
        amount: 10_000_000_i64,
        project_id: "SOMEPROJECT",
        memo_text: "A TON",
        email: ""
    };
    // it should fail because the contract is not active
    sink_carbon_with_auth(&setup, &test_data).unwrap();
}

#[test]
fn test_funder_balance_too_low() {
    let setup = set_up_contracts_and_funder(500);

    // attempt to sink 0.1 CARBON
    let test_data = SinkTestData { 
        funder: &setup.funder,
        recipient: &setup.funder,
        amount: 1_000_000_i64,
        project_id: "VCS1360",
        memo_text: "100 kg 🌳🌴",
        email: ""
    };
    // it should fail because the funder has an insufficient balance
    let sink_res = sink_carbon_with_auth(&setup, &test_data);
    assert_eq!(sink_res.unwrap_err(), SinkError::InsufficientBalance);
}

#[test]
fn test_funder_account_or_trustline_missing() {
    let setup = set_up_contracts_and_funder(0);
    let env = &setup.env;
    let client = &setup.sink_client;
    let native_asset_address = deploy_native_sac(env);
    let native_client = token::Client::new(env, &native_asset_address);

    let funder_pubkey = "GA2H3SJYGIUG2DXXUZ7IN3LNO2AIMVWCDCL25PKQHKMC76OWW3HYQHY4";
    let funder = Address::from_str(&setup.env, funder_pubkey);
    let amount = 1_000_000_i64;
    let project_id = Symbol::new(env, "VCS1360");
    let memo_text = String::from_str(env, "100 kg 🌳🌴");
    let email = String::from_str(env, "");

    // check native balance, should fail
    let xlm_balance_res = native_client.try_balance(&funder);
    assert_eq!(xlm_balance_res.unwrap_err().unwrap(), SACError::AccountMissingError.into());
    // attempt to sink with non-existing account
    let sink_res = client
        .mock_all_auths()
        .try_sink_carbon(&funder, &funder, &amount, &project_id, &memo_text, &email);
    // it should fail because the funder account wasn't created
    assert_eq!(sink_res.unwrap_err().unwrap(), SinkError::AccountOrTrustlineMissing);

    // create a ledger entry for the funder Stellar Account
    create_account_entry(env, funder_pubkey);
    // check native balance, should succeed
    let xlm_balance = native_client.balance(&funder);
    assert_eq!(xlm_balance, 10_000_000_000);
    // attempt to sink with non-existing trustline
    let sink_res = client
        .mock_all_auths()
        .try_sink_carbon(&funder, &funder, &amount, &project_id, &memo_text, &email);
    // it should fail because the trustline was never set up
    assert_eq!(sink_res.unwrap_err().unwrap(), SinkError::AccountOrTrustlineMissing);
}

#[test]
fn test_recipient_account_or_trustline_issues() {
    let setup = set_up_contracts_and_funder(10_000_000);
    let env = &setup.env;
    let native_asset_address = deploy_native_sac(env);
    let native_client = token::Client::new(env, &native_asset_address);

    let recipient_pubkey = "GBDCUWVK2SXI6DA6RD23KDBVIWMJSKLDEN3KZBIMX3TH23OJUR4YSQR4";
    let recipient = Address::from_str(&setup.env, recipient_pubkey);

    // attempt to sink 0.1 CARBON for the recipient
    let test_data = SinkTestData { 
        funder: &setup.funder,
        recipient: &recipient,
        amount: 1_000_000_i64,
        project_id: "VCS1360",
        memo_text: "100 kg 🌳🌴",
        email: ""
    };

    // check native balance, should fail
    let xlm_balance_res = native_client.try_balance(&recipient);
    assert_eq!(xlm_balance_res.unwrap_err().unwrap(), SACError::AccountMissingError.into());
    // attempt to sink with non-existing recipient account
    let sink_res = sink_carbon_with_auth(&setup, &test_data);
    // it should fail because the recipient account wasn't created
    assert_eq!(sink_res.unwrap_err(), SinkError::AccountOrTrustlineMissing);

    // create a ledger entry for the recipient Stellar Account
    create_account_entry(env, recipient_pubkey);
    // check native balance, should succeed
    let xlm_balance = native_client.balance(&recipient);
    assert_eq!(xlm_balance, 10_000_000_000);
    // attempt to sink with non-existing trustline
    let sink_res = sink_carbon_with_auth(&setup, &test_data);
    // it should fail because the trustline was never set up
    assert_eq!(sink_res.unwrap_err(), SinkError::AccountOrTrustlineMissing);

    // create a ledger entry for the CarbonSINK trustline
    create_trustline(
        env, recipient_pubkey, &setup.carbonsink_sac.issuer(), [b'a', b'a', b'a', 0], 100
    );
    // attempt to sink with trustline that has a small limit
    let sink_res = sink_carbon_with_auth(&setup, &test_data);
    // it should fail because the trustline limit is too low
    assert_eq!(sink_res.unwrap_err(), SinkError::TrustlineLimitReached);

    // for good measure, test the happy flow as well
    create_trustline(
        env, recipient_pubkey, &setup.carbonsink_sac.issuer(), [b'a', b'a', b'a', 0], -1
    );
    assert!(sink_carbon_with_auth(&setup, &test_data).is_ok());

    // assert the effect on balances
    let carbon_client = TokenClient::new(&setup.env, &setup.carbon_sac.address());
    let carbonsink_client = TokenClient::new(&setup.env, &setup.carbonsink_sac.address());
    assert_eq!(carbon_client.balance(&setup.funder), 9_000_000);
    let recipient_balance_res = carbon_client.try_balance(test_data.recipient);
    assert_eq!(recipient_balance_res.unwrap_err().unwrap(), SACError::TrustlineMissingError.into());
    assert_eq!(carbonsink_client.balance(&setup.funder), 0);
    assert_eq!(carbonsink_client.balance(test_data.recipient), 1_000_000);
}
