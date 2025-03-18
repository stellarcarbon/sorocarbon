use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SinkError {
    ContractDeactivated = 1,
    AmountTooLow = 2,
    NegativeAmount = 3,
}
