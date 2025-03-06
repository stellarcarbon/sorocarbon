#![cfg(test)]

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


    let minimum = client.get_minimum_sink_amount();
    assert_eq!(minimum, 515_000)
}

#[test]
fn test_set_sink_minimum_unauthorized() {
    let setup = set_up_contracts_and_funder(0);
    let client = setup.sink_client;

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
