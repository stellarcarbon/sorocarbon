extern crate std;
use std::rc::Rc;

use soroban_env_host::{budget::AsBudget, Env as _, EnvBase};
use soroban_sdk::xdr;
use soroban_sdk::{
    testutils::{MockAuth, MockAuthInvoke, StellarAssetIssuer}, 
    xdr::{Asset, Limits, ScAddress, WriteXdr}, 
    vec, Address, Env, FromVal, IntoVal, String, Symbol,
};
use stellar_strkey;

use crate::utils::quantize_to_kg;
use crate::{errors::SinkError, tests::fixtures::Setup};

#[derive(Clone)]
pub struct SinkTestData<'a> {
    pub funder: &'a Address,
    pub recipient: &'a Address, 
    pub amount: i64, 
    pub project_id: &'static str,
    pub memo_text: &'static str,
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
    // we need to authorize the quantized amount for the burn call
    let quantized_amount = (quantize_to_kg(amount) as i128).into_val(env);
    match client
        .mock_auths(&[MockAuth {
            address: funder,
            invoke: &MockAuthInvoke {
                contract: contract_id,
                fn_name: "sink_carbon",
                args: (
                    funder.clone(), recipient.clone(), amount, 
                    project_id.clone(), memo_text.clone()
                ).into_val(env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &carbon_sac.address(),
                    fn_name: "burn",
                    args: vec![env, funder.clone().into_val(env), quantized_amount],
                    sub_invokes: &[],
                }],
            },
        }])
        .try_sink_carbon(
            &funder, &recipient, &amount, &project_id, &memo_text,
        ) {
            Ok(Ok(())) => Ok(()),
            Err(Ok(sink_err)) => Err(sink_err),
            Ok(Err(conversion_err)) => panic!("ConversionError: {:?}", conversion_err),
            Err(Err(invoke_err)) => panic!("InvokeError: {:?}", invoke_err),
        }
}

pub fn deploy_native_sac(env: &Env) -> Address {
    let xdr_bytes = Asset::Native.to_xdr(Limits::none()).unwrap();
    let asset_slice = xdr_bytes.as_slice();
    let host = env.host();
    let bytes_obj = host.bytes_new_from_slice(asset_slice).unwrap();
    let sac_res = host.create_asset_contract(bytes_obj);
    Address::from_val(env, &sac_res.unwrap())
}

pub fn bytes_to_contract(env: &Env, bytes: &[u8; 32]) -> Address {
    let contract_id = stellar_strkey::Contract(*bytes).to_string();
    Address::from_str(env, &contract_id)
}

pub fn create_account_entry(env: &Env, pubkey: &str) {
    // Parse the public key from the Stellar address
    let raw_pubkey = stellar_strkey::ed25519::PublicKey::from_string(pubkey)
        .unwrap().0;
    
    // Create an AccountId XDR
    let account_id = xdr::AccountId(
        xdr::PublicKey::PublicKeyTypeEd25519(xdr::Uint256(raw_pubkey))
    );
    
    // Create the account entry
    env.host().with_mut_storage(|storage| {
        let key = Rc::new(xdr::LedgerKey::Account(xdr::LedgerKeyAccount {
            account_id: account_id.clone(),
        }));
        
        // Create account entry data with basic values
        let entry = Rc::new(xdr::LedgerEntry {
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
            env.host().as_budget(),
        ).unwrap();
        
        Ok(())
    }).unwrap();
}

pub fn create_trustline(
    env: &Env, account: &str, asset_issuer: &StellarAssetIssuer, asset_code: [u8; 4], limit: i64
) {
    let account_raw_pubkey = stellar_strkey::ed25519::PublicKey::from_string(account)
        .unwrap().0;
        
    let account_int = xdr::Uint256(account_raw_pubkey);
    let account_id = xdr::AccountId(
        xdr::PublicKey::PublicKeyTypeEd25519(account_int)
    );
    let issuer_sc_address = ScAddress::from(asset_issuer.address());
    let issuer = match issuer_sc_address {
        ScAddress::Account(issuer_account) => issuer_account,
        _ => panic!("Failed to convert Address to AccountId")
    };
    
    // Create the asset
    let asset = xdr::TrustLineAsset::CreditAlphanum4(xdr::AlphaNum4 {
        asset_code: xdr::AssetCode4(asset_code),
        issuer,
    });
    
    env.host().with_mut_storage(|storage| {
        let key = Rc::new(xdr::LedgerKey::Trustline(xdr::LedgerKeyTrustLine {
            account_id: account_id.clone(),
            asset: asset.clone(),
        }));
        
        // Create trustline entry
        let limit = if limit < 1 { i64::MAX } else { limit };
        let entry = Rc::new(xdr::LedgerEntry {
            last_modified_ledger_seq: 0,
            data: xdr::LedgerEntryData::Trustline(xdr::TrustLineEntry {
                account_id,
                asset,
                balance: 0,
                limit,
                flags: 0, // Unauthorized
                ext: xdr::TrustLineEntryExt::V0,
            }),
            ext: xdr::LedgerEntryExt::V0,
        });
        
        storage.put(
            &key,
            &entry,
            None,
            env.host().as_budget(),
        ).unwrap();
        
        Ok(())
    }).unwrap();
}
