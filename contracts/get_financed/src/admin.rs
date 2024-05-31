use soroban_sdk::{Address, Env};

use crate::AdminDataKey;

pub fn has_administrator(e: &Env) -> bool {
    let key = AdminDataKey::ADMIN;
    e.storage().instance().has(&key)
}

pub fn read_administrator(e: &Env) -> Address {
    let key = AdminDataKey::ADMIN;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    let key = AdminDataKey::ADMIN;
    e.storage().instance().set(&key, id);
}
