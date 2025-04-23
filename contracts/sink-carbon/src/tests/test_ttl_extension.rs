use soroban_env_host::HostError;
use soroban_sdk::testutils::{storage::Instance, Deployer, Ledger};
use soroban_sdk::xdr::{ScErrorCode, ScErrorType};

use crate::tests::fixtures::set_up_contracts_with_ttl_settings;
use crate::tests::utils::{sink_carbon_with_auth, SinkTestData};

const DAY_IN_LEDGERS: u32 = 17280;

#[test]
fn test_extend_ttl_sink_carbon() {
    let setup = set_up_contracts_with_ttl_settings(60 * DAY_IN_LEDGERS);
    let env = setup.env.clone();

    // Check initial ledger/network settings
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

    // check the contract code & instance TTL in this test
    assert_eq!(35_000, env.deployer().get_contract_code_ttl(&setup.contract_id));
    assert_eq!(35_000, env.deployer().get_contract_instance_ttl(&setup.contract_id));
    assert_eq!(true, client.is_active());

    // expected TTL is 30 days
    assert_eq!(30 * DAY_IN_LEDGERS, env.deployer().get_contract_code_ttl(&setup.contract_id));
    assert_eq!(30 * DAY_IN_LEDGERS, env.deployer().get_contract_instance_ttl(&setup.contract_id));

    // advance the ledger by 2 days
    env.ledger().with_mut(|li| {
        li.sequence_number = 100_000 + 2 * DAY_IN_LEDGERS;
    });
    assert_eq!(28 * DAY_IN_LEDGERS, env.deployer().get_contract_code_ttl(&setup.contract_id));
    assert_eq!(28 * DAY_IN_LEDGERS, env.deployer().get_contract_instance_ttl(&setup.contract_id));

    // call is_active again, expected TTL is back to 30 days
    assert_eq!(true, client.is_active());
    assert_eq!(30 * DAY_IN_LEDGERS, env.deployer().get_contract_code_ttl(&setup.contract_id));
    assert_eq!(30 * DAY_IN_LEDGERS, env.deployer().get_contract_instance_ttl(&setup.contract_id));
}

#[test]
fn test_contract_archival() {
    let setup = set_up_contracts_with_ttl_settings(60 * DAY_IN_LEDGERS);
    let env = setup.env.clone();
    let client = setup.sink_client;

    assert_eq!(35_000, env.deployer().get_contract_code_ttl(&setup.contract_id));

    // advance the ledger by 2 days
    const TWO_DAYS: u32 = 2 * DAY_IN_LEDGERS;
    env.ledger().with_mut(|li| {
        li.sequence_number += TWO_DAYS;
    });
    assert_eq!(35_000 - TWO_DAYS, env.deployer().get_contract_code_ttl(&setup.contract_id));

    // extend TTL with get_minimum_sink_amount
    assert!(client.get_minimum_sink_amount() > 0);
    assert_eq!(30 * DAY_IN_LEDGERS, env.deployer().get_contract_code_ttl(&setup.contract_id));

    // advance the ledger by 31 days
    env.ledger().with_mut(|li| {
        li.sequence_number += 31 * DAY_IN_LEDGERS;
    });
    // the contract TTL is expected to have expired: this contract is archived
    let ttl_res = env.host().get_contract_code_live_until_ledger(setup.contract_id.to_object());
    assert!(HostError::result_matches_err(ttl_res, (ScErrorType::Storage, ScErrorCode::InternalError)))
}
