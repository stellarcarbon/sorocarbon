use soroban_sdk::{
    Address, Bytes, Env, FromVal, TryFromVal, Val
};

extern "C" {
    fn kasmer_create_contract(addr_val: u64, hash_val: u64) -> u64;
}
extern "C" {
    fn kasmer_address_from_bytes(addr_val: u64, is_contract: u64) -> u64;
}

pub fn create_contract(env: &Env, addr: &Bytes, hash: &Bytes) -> Address {
    unsafe {
        let res = kasmer_create_contract(addr.as_val().get_payload(), hash.as_val().get_payload());
        Address::from_val(env, &Val::from_payload(res))
    }
}

pub fn address_from_bytes<T>(env: &Env, bs: &T, is_contract: bool) -> Address
    where Bytes: TryFromVal<Env, T>
{
    let bs: Bytes = Bytes::try_from_val(env, bs).unwrap();

    unsafe {
        let res = kasmer_address_from_bytes(
            Val::from_val(env, &bs).get_payload(),
            Val::from_val(env, &is_contract).get_payload()
        );
        Address::from_val(env, &Val::from_payload(res))
    }
}    
