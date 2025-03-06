#![cfg(test)]

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, InvokeError, String, Symbol, TryFromVal, Val, vec
};

use crate::tests::fixtures::Setup;

#[derive(Clone)]
pub struct SinkTestData<'a> { 
    pub recipient: &'a Address, 
    pub amount: i64, 
    pub project_id: &'static str,
    pub memo_text: &'static str,
    pub email: &'static str,
}

pub fn sink_carbon_with_auth(setup: &Setup, test_data: &SinkTestData) -> Result<
    Result<(), <() as TryFromVal<Env, Val>>::Error>, 
    Result<soroban_sdk::Error, InvokeError>
> {
    let env = &setup.env;
    let funder = &setup.funder;
    let carbon_sac = &setup.carbon_sac;
    let contract_id = &setup.contract_id;

    // have the funder sink `amount` of CARBON for the recipient
    let client = &setup.sink_client;
    let recipient = test_data.recipient;
    let amount = test_data.amount;
    let project_id = Symbol::new(env, test_data.project_id);
    let memo_text = String::from_str(env,test_data.memo_text);
    let email = String::from_str(env, test_data.email);
    client
        .mock_auths(&[MockAuth {
            address: funder,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "sink_carbon",
                args: (
                    funder.clone(), recipient.clone(), amount, 
                    project_id.clone(), memo_text.clone(), email.clone()
                ).into_val(env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &carbon_sac.address(),
                    fn_name: "burn",
                    args: vec![env, funder.clone().into_val(env), (amount as i128).into_val(env)],
                    sub_invokes: &[],
                }],
            },
        }])
        .try_sink_carbon(
            &funder, &recipient, &amount, &project_id, &memo_text, &email
        )
}
