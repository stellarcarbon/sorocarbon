use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SinkError {
    ContractDeactivated = 1,
    AmountTooLow = 2,
    NegativeAmount = 3,
    InsufficientBalance = 4,
    AccountMissing = 5,
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
