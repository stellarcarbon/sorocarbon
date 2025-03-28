#![cfg(test)]

use soroban_sdk::testutils::{storage::Instance, Ledger};

use crate::tests::{fixtures::set_up_contracts_with_ttl_settings, utils::{sink_carbon_with_auth, SinkTestData}};

const DAY_IN_LEDGERS: u32 = 17280;

#[test]
fn test_extend_ttl_sink_carbon() {
    let setup = set_up_contracts_with_ttl_settings(60 * DAY_IN_LEDGERS);
    let env = setup.env.clone();

    // Create initial entries and make sure their TTLs correspond to
    // `min_persistent_entry_ttl` and `min_temp_entry_ttl` values set in
    // `create_env()`.
    env.as_contract(&setup.contract_id, || {
        // Note, that TTL doesn't include the current ledger, but when entry
        // is created the current ledger is counted towards the number of
        // ledgers specified by `min_persistent/temp_entry_ttl`, thus
        // the TTL is 1 ledger less than the respective setting.
        assert_eq!(35_000, env.storage().instance().get_ttl());
        // TODO: extend with the other storage types if they are used in the future
        // assert_eq!(env.storage().persistent().get_ttl(&DataKey::MyKey), 499);
        // assert_eq!(env.storage().temporary().get_ttl(&DataKey::MyKey), 99);
    });

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

    // expected TTL is 30 days
    env.as_contract(&setup.contract_id, || {
        assert_eq!(30 * DAY_IN_LEDGERS, env.storage().instance().get_ttl());
    });

    // Now bump the ledger sequence by 2 days in order to sanity-check
    // the threshold settings of `extend_ttl` operations.
    env.ledger().with_mut(|li| {
        li.sequence_number = 100_000 + 2 * DAY_IN_LEDGERS;
    });
    env.as_contract(&setup.contract_id, || {
        assert_eq!(28 * DAY_IN_LEDGERS, env.storage().instance().get_ttl());
    });

    // sink more CARBON, expected TTL is 30 days again
    assert!(sink_carbon_with_auth(&setup, &test_data).is_ok());
    env.as_contract(&setup.contract_id, || {
        assert_eq!(30 * DAY_IN_LEDGERS, env.storage().instance().get_ttl());
    });
}

#[test]
fn test_extend_ttl_is_active() {
    let setup = set_up_contracts_with_ttl_settings(60 * DAY_IN_LEDGERS);
    let env = setup.env.clone();
    let client = setup.sink_client;

    env.as_contract(&setup.contract_id, || {
        assert_eq!(35_000, env.storage().instance().get_ttl());
    });
    assert_eq!(true, client.is_active());

    // expected TTL is 30 days
    env.as_contract(&setup.contract_id, || {
        assert_eq!(30 * DAY_IN_LEDGERS, env.storage().instance().get_ttl());
    });

    // Now bump the ledger sequence by 2 days in order to sanity-check
    // the threshold settings of `extend_ttl` operations.
    env.ledger().with_mut(|li| {
        li.sequence_number = 100_000 + 2 * DAY_IN_LEDGERS;
    });
    env.as_contract(&setup.contract_id, || {
        assert_eq!(28 * DAY_IN_LEDGERS, env.storage().instance().get_ttl());
    });

    // call is_active again, expected TTL is back to 30 days
    assert_eq!(true, client.is_active());
    env.as_contract(&setup.contract_id, || {
        assert_eq!(30 * DAY_IN_LEDGERS, env.storage().instance().get_ttl());
    });
}
