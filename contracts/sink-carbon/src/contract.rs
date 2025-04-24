use soroban_sdk::{
    contract, contractimpl, panic_with_error,
    token::{StellarAssetClient, TokenClient}, 
    Address, Env, String, Symbol
};

use crate::errors::{SACError, SinkError};
use crate::storage_types::{extend_instance_ttl, set_is_active, DataKey};
use crate::utils::quantize_to_kg;


#[contract]
pub struct SinkContract;

#[contractimpl]
impl SinkContract {
    pub fn __constructor(env: Env, admin: Address, carbon_id: Address, carbonsink_id: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CarbonID, &carbon_id);
        env.storage().instance().set(&DataKey::CarbonSinkID, &carbonsink_id);
        env.storage().instance().set(&DataKey::IsActive, &true);
        env.storage().instance().set(&DataKey::SinkMinimum, &1_000_000_i64);  // 100 kg
    }

    pub fn sink_carbon(
        env: Env, 
        funder: Address, 
        recipient: Address, 
        amount: i64, 
        project_id: Symbol,
        memo_text: String,
        email: String,
    ) -> Result<(), SinkError> {
        #[cfg(feature = "mercury")]
        {
            // emit sink event
            crate::retroshades::SinkEvent {
                funder,
                recipient,
                amount,
                project_id,
                memo_text,
                email,
                ledger: env.ledger().sequence(),
            }
            .emit(&env);
            // and return early to spare the ZVM
            return Ok(());
        }
        #[allow(unused)]
        let (project_id, memo_text, email) = (project_id, memo_text, email);

        extend_instance_ttl(&env);
        if !Self::is_active(env.clone()) {
            panic_with_error!(&env, SinkError::ContractDeactivated);
        }

        // quantize `amount` to kg resolution
        let amount = quantize_to_kg(amount);

        // check if `amount` equals or exceeds minimum
        let minimum_sink_amount: i64 = env.storage().instance().get(&DataKey::SinkMinimum).unwrap();
        if amount < minimum_sink_amount {
            return Err(SinkError::AmountTooLow);
        }

        // `funder` burns `amount` of CARBON
        funder.require_auth();
        let carbon_id = env.storage().instance().get(&DataKey::CarbonID).unwrap();
        let carbon_client = TokenClient::new(&env, &carbon_id);
        match carbon_client.try_burn(&funder, &amount.into()) {
            Ok(_) => {}
            Err(Ok(err)) => {
                let error_code = err.get_code();
                if error_code == SACError::BalanceError as u32 {
                    // most likely the funder's CARBON balance is too low
                    return Err(SinkError::InsufficientBalance);
                } else if error_code == SACError::TrustlineMissingError as u32 {
                    // burn internals check the trustline; account is only checked for native transfer
                    return Err(SinkError::AccountOrTrustlineMissing);
                } // re-panic for unexpected errors
                panic_with_error!(&env, err);
            },
            Err(Err(invoke_err)) => {
                panic!("InvokeError: {:?}", invoke_err);
            }
        }

        // `recipient` receives `amount` of CarbonSINK
        let carbonsink_id = env.storage().instance().get(&DataKey::CarbonSinkID).unwrap();
        let carbonsink_client = StellarAssetClient::new(&env, &carbonsink_id);
        match carbonsink_client.try_set_authorized(&recipient, &true) {
            Ok(_) => {}
            Err(Ok(err)) => {
                if err.get_code() == SACError::TrustlineMissingError as u32 {
                    // `set_authorization` reads the trustline entry, not the account entry
                    return Err(SinkError::AccountOrTrustlineMissing);
                } // re-panic for unexpected errors
                panic_with_error!(&env, err);
            },
            Err(Err(invoke_err)) => {
                panic!("InvokeError: {:?}", invoke_err);
            }
        };
        match carbonsink_client.try_mint(&recipient, &amount.into()) {
            Ok(_) => {}
            Err(Ok(err)) => {
                if err.get_code() == SACError::BalanceError as u32 {
                    return Err(SinkError::TrustlineLimitReached);
                } // re-panic for unexpected errors
                panic_with_error!(&env, err);
            },
            Err(Err(invoke_err)) => {
                panic!("InvokeError: {:?}", invoke_err);
            }
        }
        carbonsink_client.set_authorized(&recipient, &false);

        Ok(())
    }

    pub fn get_minimum_sink_amount(env: Env) -> i64 {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::SinkMinimum).unwrap()
    }

    pub fn is_active(env: Env) -> bool {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::IsActive).unwrap()
    }

    // ADMIN FUNCTIONS

    pub fn set_minimum_sink_amount(env: Env, amount: i64) -> Result<(), SinkError> {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if amount < 0 {
            panic_with_error!(&env, SinkError::NegativeAmount);
        } else {
            env.storage().instance().set(&DataKey::SinkMinimum, &amount);
            Ok(())
        }
    }

    pub fn reset_admin(env: Env) -> Address {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        // set contract admin as the CarbonSINK SAC admin
        let carbonsink_id = env.storage().instance().get(&DataKey::CarbonSinkID).unwrap();
        let carbonsink_client = StellarAssetClient::new(&env, &carbonsink_id);
        carbonsink_client.set_admin(&admin);

        // deactivate this contract
        set_is_active(&env, false);

        admin
    }

    pub fn activate(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        set_is_active(&env, true);
    }

    pub fn deactivate(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        set_is_active(&env, false);
    }
}
