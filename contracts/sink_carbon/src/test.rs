#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, vec, Env, String};

#[test]
fn test() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let carbonsink_sac = Address::generate(&env);
    let contract_id = env.register(
        SinkContract, 
        (admin, carbonsink_sac)
    );
    let client = SinkContractClient::new(&env, &contract_id);

    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        vec![
            &env,
            String::from_str(&env, "Hello"),
            String::from_str(&env, "Dev"),
        ]
    );
}
