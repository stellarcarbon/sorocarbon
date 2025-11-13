use soroban_sdk::{
    contract, contractimpl, panic_with_error,
    token::{StellarAssetClient, TokenClient}, 
    Address, Env, String, Symbol
};

use crate::errors::{publish_invoke_error, publish_sac_error, SACError, SinkError};
use crate::storage_types::{extend_instance_ttl, set_is_active, DataKey};
use crate::utils::quantize_to_kg;


#[contract]
pub struct SinkContract;

#[contractimpl]
impl SinkContract {
    /// Initializes the SinkContract with the admin, CARBON asset ID, and CarbonSINK asset ID.
    /// Sets up initial storage values including admin, asset IDs, active status, and minimum sink amount.
    /// This constructor must be called once during contract deployment.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// * `admin` - The address of the contract administrator.
    /// * `carbon_id` - The address of the CARBON asset contract.
    /// * `carbonsink_id` - The address of the CarbonSINK asset contract.
    pub fn __constructor(env: Env, admin: Address, carbon_id: Address, carbonsink_id: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CarbonID, &carbon_id);
        env.storage().instance().set(&DataKey::CarbonSinkID, &carbonsink_id);
        env.storage().instance().set(&DataKey::IsActive, &true);
        env.storage().instance().set(&DataKey::SinkMinimum, &1_000_000_i64);  // 100 kg
    }

    /// Sinks CARBON tokens by burning them from the `funder` and minting equivalent CarbonSINK tokens to the `recipient`.
    /// Quantizes the amount to kg resolution, checks minimum requirements, and handles authorization for CarbonSINK minting.
    /// Emits an event if the Mercury feature is enabled, otherwise performs the full sink operation.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// * `funder` - The address funding the sink operation (spends CARBON).
    /// * `recipient` - The address receiving the CarbonSINK tokens.
    /// * `amount` - The amount of CARBON to sink in decigrams.
    /// * `project_id` - The impact project ID (e.g. VCS1360).
    /// * `memo_text` - A retirement reason or transaction reference.
    /// 
    /// ## Returns
    /// 
    /// A Result indicating success (unit type) or a `SinkError`.
    /// 
    /// ## Errors
    /// 
    /// * `AmountTooLow` - The quantized amount is below the minimum sink amount.
    /// * `InsufficientBalance` - The funder's CARBON balance is insufficient.
    /// * `AccountOrTrustlineMissing` - The funder or recipient has a missing trustline or account.
    /// * `TrustlineLimitReached` - The recipient's trustline limit is reached for CarbonSINK.
    /// * `ContractDeactivated` - The contract is not currently active, preventing this operation.
    /// 
    /// ## Notes    
    ///  
    /// The function has five balance properties:
    /// 
    /// 1. Total CARBON supply decreases by `amount` (via burn).
    /// 2. Total CarbonSINK supply increases by `amount` (via mint).
    /// 3. Funder's CARBON balance debits by `amount`.
    /// 4. Recipient's CarbonSINK balance credits by `amount`.
    /// 5. Recipient's CarbonSINK balance is locked onto the account (deauthorized).
    /// 
    /// This manifests as an "X-shaped swap". It's a direct transfer: funder sends CARBON,
    /// recipient receives 1:1 CarbonSINK as verifiable retirement proof.
    /// Internally, crossed flows ensure atomicity: funder → CARBON removed from circulation (one axis),
    /// CarbonSINK minter → recipient (other axis).
    /// 
    /// Prerequisites: The contract must be active, the funder must have a sufficient CARBON balance and authorize the burn,
    /// the recipient must have a CarbonSINK trustline with a limit at least as large as the quantized amount,
    /// and the amount must be at least the minimum sink amount after quantization to kg resolution.
    /// 
    /// Common pitfalls: Ensure the funder has enough CARBON and the recipient has a valid trustline; otherwise,
    /// the operation will fail with errors like `InsufficientBalance` or `TrustlineLimitReached`.
    pub fn sink_carbon(
        env: Env, 
        funder: Address, 
        recipient: Address, 
        amount: i64, 
        project_id: Symbol,
        memo_text: String,
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
                ledger: env.ledger().sequence(),
                timestamp: env.ledger().timestamp(),
            }
            .emit(&env);
            // and return early to spare the ZVM
            return Ok(());
        }
        #[allow(unused)]
        let (project_id, memo_text) = (project_id, memo_text);

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
                publish_sac_error(&env, "carbon_sac_error", error_code);
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
                publish_invoke_error(&env, "burn_carbon", invoke_err);
                panic!("InvokeError: {:?}", invoke_err);
            }
        }

        // `recipient` receives `amount` of CarbonSINK
        let carbonsink_id = env.storage().instance().get(&DataKey::CarbonSinkID).unwrap();
        let carbonsink_client = StellarAssetClient::new(&env, &carbonsink_id);
        match carbonsink_client.try_set_authorized(&recipient, &true) {
            Ok(_) => {}
            Err(Ok(err)) => {
                let error_code = err.get_code();
                publish_sac_error(&env, "csink_sac_error", error_code);
                if error_code == SACError::TrustlineMissingError as u32 {
                    // `set_authorization` reads the trustline entry, not the account entry
                    return Err(SinkError::AccountOrTrustlineMissing);
                } // re-panic for unexpected errors
                panic_with_error!(&env, err);
            },
            Err(Err(invoke_err)) => {
                publish_invoke_error(&env, "set_authorized", invoke_err);
                panic!("InvokeError: {:?}", invoke_err);
            }
        };
        match carbonsink_client.try_mint(&recipient, &amount.into()) {
            Ok(_) => {}
            Err(Ok(err)) => {
                let error_code = err.get_code();
                publish_sac_error(&env, "csink_sac_error", error_code);
                if error_code == SACError::BalanceError as u32 {
                    return Err(SinkError::TrustlineLimitReached);
                } // re-panic for unexpected errors
                panic_with_error!(&env, err);
            },
            Err(Err(invoke_err)) => {
                publish_invoke_error(&env, "mint_csink", invoke_err);
                panic!("InvokeError: {:?}", invoke_err);
            }
        }
        carbonsink_client.set_authorized(&recipient, &false);

        Ok(())
    }

    /// Retrieves the minimum sink amount required for a sink operation.
    /// Extends the instance TTL before returning the value.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// 
    /// ## Returns
    /// 
    /// The minimum sink amount as an i64.
    pub fn get_minimum_sink_amount(env: Env) -> i64 {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::SinkMinimum).unwrap()
    }

    /// Checks if the contract is currently active.
    /// A deactivated contract that does not have a successor may be temporarily paused.
    /// Extends the instance TTL before returning the value.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// 
    /// ## Returns
    /// 
    /// A boolean indicating if the contract is active.
    pub fn is_active(env: Env) -> bool {
        extend_instance_ttl(&env);
        env.storage().instance().get(&DataKey::IsActive).unwrap()
    }

    /// Retrieves the contract successor address, or the current contract address if no successor is set.
    /// Note that the successor contract may itself already be superseded by another contract.
    /// To find the latest active contract, call this function on each successor until it returns its own address.
    /// Extends the instance TTL before returning the value.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// 
    /// ## Returns
    /// 
    /// The successor address or the current contract address.
    pub fn get_contract_successor(env: Env) -> Address {
        extend_instance_ttl(&env);

        // provide current contract address if this func is called when contract is still active
        env.storage().instance().get(&DataKey::ContractSuccessor).unwrap_or(
            env.current_contract_address()
        )
    }

    // ADMIN FUNCTIONS

    /// Sets the contract successor address for upgrades.
    /// Requires admin authorization and extends the instance TTL.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// * `successor` - The address of the successor contract.
    pub fn set_contract_successor(env: Env, successor: Address) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        env.storage().instance().set(&DataKey::ContractSuccessor, &successor);
    }

    /// Sets the minimum sink amount for sink operations.
    /// Requires admin authorization, validates the amount is non-negative, and extends the instance TTL.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// * `amount` - The new minimum sink amount (must be >= 0).
    /// 
    /// ## Returns
    /// 
    /// A Result indicating success or an error if the amount is negative.
    /// 
    /// ## Errors
    /// 
    /// * `NegativeAmount` - The provided amount must be positive.
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

    /// Resets the CarbonSINK SAC admin by setting it to the current contract admin and deactivates the contract.
    /// This is called as part of the upgrade process, before setting a new SinkContract as the CarbonSINK SAC admin.
    /// Requires admin authorization and extends the instance TTL.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    /// 
    /// ## Returns
    /// 
    /// The address of the admin.
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

    /// Activates the contract, allowing sink operations.
    /// Requires admin authorization and extends the instance TTL.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    pub fn activate(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        set_is_active(&env, true);
    }

    /// Deactivates the contract, preventing sink operations.
    /// Requires admin authorization and extends the instance TTL.
    /// 
    /// ## Arguments
    /// 
    /// * `env` - The Soroban environment.
    pub fn deactivate(env: Env) {
        extend_instance_ttl(&env);
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        set_is_active(&env, false);
    }
}
