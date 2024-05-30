#![no_std]

mod test;
mod token;

use num_integer::Roots;
use soroban_sdk::{
    contract, contractimpl, contractmeta, Address, BytesN, ConversionError, Env, IntoVal,
    TryFromVal, Val,
};
use token::create_contract;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Usdc = 0,
    TokenShare = 1,
    TotalShares = 2,
}

fn get_usdc(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Usdc).unwrap()
}
fn get_token_share(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenShare).unwrap()
}

fn get_total_shares(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalShares).unwrap()
}

fn get_balance(e: &Env, contract: Address) -> i128 {
    token::Client::new(e, &contract).balance(&e.current_contract_address())
}

fn get_balance_shares(e: &Env) -> i128 {
    get_balance(e, get_token_share(e))
}

fn get_balance_usdc(e: &Env) -> i128 {
    get_balance(e, get_usdc(e))
}

fn put_token_usdc(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::Usdc, &contract);
}

fn put_token_share(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::TokenShare, &contract);
}

fn put_total_shares(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalShares, &amount)
}

fn burn_shares(e: &Env, amount: i128) {
    let total = get_total_shares(e);
    let share_contract = get_token_share(e);

    token::Client::new(e, &share_contract).burn(&e.current_contract_address(), &amount);
    put_total_shares(e, total - amount);
}

fn mint_shares(e: &Env, to: Address, amount: i128) {
    let total = get_total_shares(e);
    let share_contract_id = get_token_share(e);

    token::Client::new(e, &share_contract_id).mint(&to, &amount);

    put_total_shares(e, total + amount);
}

fn transfer(e: &Env, token: Address, to: Address, amount: i128) {
    token::Client::new(e, &token).transfer(&e.current_contract_address(), &to, &amount);
}

#[contract]
struct GetFinanced;

#[contractimpl]
impl GetFinanced {
    fn initialize(e: Env, token_wasm_hash: BytesN<32>, usdc: Address) {
        let share_contract = create_contract(&e, token_wasm_hash, &usdc);
        token::Client::new(&e, &share_contract).initialize(
            &e.current_contract_address(),
            &7u32,
            &"GF Yield-bearing USDC".into_val(&e),
            &"gfUSDC".into_val(&e),
        );

        put_token_usdc(&e, usdc);
        put_token_share(&e, share_contract);
        put_total_shares(&e, 0);
    }

    fn deposit(e: Env, from: Address, amount: i128) {
        // Depositor needs to authorize the deposit
        from.require_auth();

        // Transfer the amount to the contract
        let usdc_client = token::Client::new(&e, &get_usdc(&e));
        usdc_client.transfer(&from, &e.current_contract_address(), &amount);

        // Now calculate how many new pool shares to mint
        let balance_usdc = get_balance_usdc(&e);
        if balance_usdc == 0 {
            mint_shares(&e, from, amount);
        } else {
            let total_shares = get_total_shares(&e);
            let new_shares =  total_shares * amount / balance_usdc;
            mint_shares(&e, from, new_shares);
        }
    }

    fn withdraw(e: Env, to: Address, share_amount: i128) {
        to.require_auth();

        // First transfer the pool shares that need to be redeemed
        let share_token_client = token::Client::new(&e, &get_token_share(&e));
        share_token_client.transfer(&to, &e.current_contract_address(), &share_amount);

        let balance_usdc = get_balance_usdc(&e);
        let balance_shares = get_balance_shares(&e);
        let total_shares = get_total_shares(&e);

        // Now calculate the withdraw amounts
        let withdraw = (balance_usdc * balance_shares) / total_shares;

        burn_shares(&e, balance_shares);
        transfer(&e, get_usdc(&e), to, withdraw);
    }

    fn request_loan(e: Env, from: Address, amount: i128, )

    fn get_usdc_address(e: Env) -> Address {
        get_usdc(&e)
    }

    fn get_share_token_address(e: Env) -> Address {
        get_token_share(&e)
    }

    fn get_shares(e: Env) -> i128 {
        get_total_shares(&e)
    }
}
