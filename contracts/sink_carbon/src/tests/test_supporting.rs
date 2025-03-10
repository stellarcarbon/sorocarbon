#![cfg(test)]

use soroban_sdk::token::StellarAssetClient;
use soroban_sdk::{Address, IntoVal};
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};

use crate::tests::fixtures::set_up_contracts_and_funder;

#[test]
fn test_get_sink_minimum_default() {
    let setup = set_up_contracts_and_funder(0);
    let client = setup.sink_client;
    let minimum = client.get_minimum_sink_amount();
    assert_eq!(minimum, 1_000_000)
}

#[test]
fn test_set_sink_minimum_as_admin() {
    let setup = set_up_contracts_and_funder(0);
    let client = setup.sink_client;
    let admin = setup.carbonsink_issuer;

    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_minimum_sink_amount",
                args: (515_000_i64,).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .set_minimum_sink_amount(&515_000);

    // the minimum should be set succesfully
    let minimum = client.get_minimum_sink_amount();
    assert_eq!(minimum, 515_000)
}

#[test]
fn test_set_sink_minimum_unauthorized() {
    let setup = set_up_contracts_and_funder(0);
    let client = setup.sink_client;

    // set minimum as a random address should fail
    assert!(
        client
        .mock_auths(&[MockAuth {
            address: &Address::generate(&setup.env),
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_minimum_sink_amount",
                args: (0_i64,).into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .try_set_minimum_sink_amount(&0).is_err()
    );
}

#[test]
fn test_activate_deactivate() {
    let setup = set_up_contracts_and_funder(0);
    let client = setup.sink_client;
    let admin = setup.carbonsink_issuer;

    // the default should be an active contract
    let initial_is_active = client.is_active();
    assert_eq!(initial_is_active, true);

    // disable the contract and check the state
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "deactivate",
                args: ().into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .deactivate();

    let after_deactivate = client.is_active();
    assert_eq!(after_deactivate, false);

    // enable the contract and check the state
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "activate",
                args: ().into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .activate();

    let after_reactivate = client.is_active();
    assert_eq!(after_reactivate, true);
}

#[test]
fn test_deactivate_unauthorized() {
    let setup = set_up_contracts_and_funder(0);
    let client = setup.sink_client;

    // it should fail because the call lacks admin auth
    assert!(client.try_deactivate().is_err());
}

#[test]
fn test_reset_admin() {
    let setup = set_up_contracts_and_funder(0);
    let client = setup.sink_client;
    let admin = setup.carbonsink_issuer;
    let carbonsink_sac = setup.carbonsink_sac;
    let carbonsink_client = StellarAssetClient::new(&setup.env, &carbonsink_sac.address());

    // initial state: CarbonSINK SAC admin is this contract
    let sac_admin = carbonsink_client.admin();
    assert_ne!(sac_admin, admin);
    assert_eq!(sac_admin, setup.contract_id);

    // reset SAC admin to CarbonSINK issuer
    let carbonsink_issuer = client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "reset_admin",
                args: ().into_val(&setup.env),
                sub_invokes: &[],
            },
        }])
        .reset_admin();

    assert_eq!(carbonsink_issuer, admin);
    let new_sac_admin = carbonsink_client.admin();
    assert_eq!(new_sac_admin, admin);
}
