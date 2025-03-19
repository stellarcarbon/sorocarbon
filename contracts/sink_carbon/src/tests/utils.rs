#![cfg(test)]

extern crate std;
use std::rc;

use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke}, vec, xdr, Address, Env, IntoVal, String, Symbol
};
use stellar_strkey;

use crate::{errors::SinkError, tests::fixtures::Setup};

#[derive(Clone)]
pub struct SinkTestData<'a> {
    pub funder: &'a Address,
    pub recipient: &'a Address, 
    pub amount: i64, 
    pub project_id: &'static str,
    pub memo_text: &'static str,
    pub email: &'static str,
}

pub fn sink_carbon_with_auth(setup: &Setup, test_data: &SinkTestData) -> Result<(), SinkError> {
    let env = &setup.env;
    let carbon_sac = &setup.carbon_sac;
    let contract_id = &setup.contract_id;

    // have the funder sink `amount` of CARBON for the recipient
    let client = &setup.sink_client;
    let funder = test_data.funder;
    let recipient = test_data.recipient;
    let amount = test_data.amount;
    let project_id = Symbol::new(env, test_data.project_id);
    let memo_text = String::from_str(env, test_data.memo_text);
    let email = String::from_str(env, test_data.email);
    match client
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
        ) {
        Ok(Ok(())) => Ok(()),
        Err(Ok(sink_err)) => Err(sink_err),
        Ok(Err(conversion_err)) => panic!("ConversionError: {:?}", conversion_err),
        Err(Err(invoke_err)) => panic!("InvokeError: {:?}", invoke_err),
    }
}

pub fn create_account_entry(env: &Env, pubkey: &str) {
    // Parse the public key from the Stellar address
    let raw_pubkey = stellar_strkey::ed25519::PublicKey::from_string(pubkey)
        .unwrap().0;
    
    // Create an AccountId XDR
    let account_id = xdr::AccountId::from(
        xdr::PublicKey::PublicKeyTypeEd25519(xdr::Uint256(raw_pubkey))
    );
    
    // Create the account entry
    env.host().with_mut_storage(|storage| {
        let key = rc::Rc::new(xdr::LedgerKey::Account(xdr::LedgerKeyAccount {
            account_id: account_id.clone(),
        }));
        
        // Create account entry data with basic values
        let entry = rc::Rc::new(xdr::LedgerEntry {
            last_modified_ledger_seq: 0,
            data: xdr::LedgerEntryData::Account(xdr::AccountEntry {
                account_id: account_id,
                balance: 10_000_000_000, // 1000 XLM in stroops
                seq_num: xdr::SequenceNumber(1),
                num_sub_entries: 0,
                inflation_dest: None,
                flags: 0,
                home_domain: xdr::String32::default(),
                thresholds: xdr::Thresholds([1, 0, 0, 0]),
                signers: xdr::VecM::default(),
                ext: xdr::AccountEntryExt::V0,
            }),
            ext: xdr::LedgerEntryExt::V0,
        });
        
        // Add the entry to storage
        storage.put(
            &key,
            &entry,
            None,
            soroban_env_host::budget::AsBudget::as_budget(env.host()),
        ).unwrap();
        
        Ok(())
    }).unwrap();
}
