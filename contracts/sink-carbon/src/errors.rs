use soroban_sdk::{contracterror, Env, InvokeError};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SinkError {
    ContractDeactivated = 1066,
    AmountTooLow = 1067,
    NegativeAmount = 1068,
    InsufficientBalance = 1069,
    AccountOrTrustlineMissing = 1070,
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
