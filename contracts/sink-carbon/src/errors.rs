use soroban_sdk::{contracterror, Env, InvokeError};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SinkError {
    /// The contract is deactivated and cannot perform sink operations.
    ContractDeactivated = 1066,
    /// The sink amount is below the minimum required amount.
    AmountTooLow = 1067,
    /// A negative amount was provided, which is invalid.
    NegativeAmount = 1068,
    /// The funder's balance is insufficient for the operation.
    InsufficientBalance = 1069,
    /// An account or trustline ledger entry is missing for the operation.
    AccountOrTrustlineMissing = 1070,
    /// The trustline limit has been reached for the recipient.
    TrustlineLimitReached = 1071,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SACError {
    InternalError = 1,
    OperationNotSupportedError = 2,
    AlreadyInitializedError = 3,
    UnauthorizedError = 4,
    AuthenticationError = 5,
    AccountMissingError = 6,
    AccountIsNotClassic = 7,
    NegativeAmountError = 8,
    AllowanceError = 9,
    BalanceError = 10,
    BalanceDeauthorizedError = 11,
    OverflowError = 12,
    TrustlineMissingError = 13,
}

pub fn publish_sac_error(env: &Env, sub_topic: &str, error_code: u32) {
    env.events().publish(
        ("sink_carbon", sub_topic),
        error_code
    );
}

pub fn publish_invoke_error(env: &Env, sub_topic: &str, invoke_err: InvokeError) {
    match invoke_err {
        InvokeError::Abort => {
            env.events().publish(
                ("sink_carbon", "invoke_error", sub_topic),
                "panic_or_host_error"
            );
        }
        InvokeError::Contract(code) => {
            env.events().publish(
                ("sink_carbon", "invoke_error", sub_topic),
                ("contract_error", code)
            );
        }
    }
}
