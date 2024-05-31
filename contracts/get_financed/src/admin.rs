use soroban_sdk::{Address, Env};

use crate::lib::AdminDataKey;

pub fn has_administrator(e: &Env) -> bool {
    let key = AdminDataKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Address {
    let key = AdminDataKey::Admin;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = AdminDataKey::Admin;
    e.storage().instance().set(&key, id);
}
