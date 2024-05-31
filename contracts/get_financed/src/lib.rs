#![no_std]

mod test;
mod token;

use num_integer::Roots;
use soroban_sdk::{
    contract, contractimpl, contractmeta, Address, BytesN, ConversionError, Env, IntoVal,
    TryFromVal, Val, Vec
};
use token::create_contract;
use crate::admin::{has_administrator, read_administrator, write_administrator};

pub(crate) const FEES_PORTION_FOR_INSURANCE: u32 = 10;
#[contracttype]
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    Usdc = 0,
    TokenShare = 1,
    InsuranceAddress = 2,
    TotalShares = 3,
    TotalLoanAmount = 4,
    TotalOutstandingLoan = 5,
    FeesEarned = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminDataKey {
    ADMIN,
    WHITELISTED(Address),
    INVNO(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanDetails {
    pub who: Address,
    pub fee_rate: i128,
    pub invoice_amount: i128,
    pub loan_amount: i128,
    pub repayment_date: i128,
    pub approved: bool,
    pub released: bool,
    pub repaid: bool,
}

fn get_usdc(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Usdc).unwrap()
}

fn get_token_share(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::TokenShare).unwrap()
}

fn get_insurance_address(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::InsuranceAddress).unwrap()
}

fn get_total_shares(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalShares).unwrap()
}

fn get_balance(e: &Env, contract: Address) -> i128 {
    token::Client::new(e, &contract).balance(&e.current_contract_address())
}

fn get_total_loan_amount(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalLoanAmount).unwrap_or(0)
}

fn get_total_outstanding_loan(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::TotalOutstandingLoan).unwrap_or(0)
}

fn get_fees_earned(e: &Env) -> i128 {
    e.storage().instance().get(&DataKey::FeesEarned).unwrap_or(0)
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

fn put_insurance_address(e: &Env, contract: Address) {
    e.storage().instance().set(&DataKey::InsuranceAddress, &contract);
}

fn put_total_shares(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalShares, &amount)
}

fn put_total_loan_amount(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalLoanAmount, &amount)
}

fn put_fees_earned(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::FeesEarned, &amount)
}

fn put_total_outstanding_loan(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalOutstandingLoan, &amount)
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
    fn initialize(e: Env, token_wasm_hash: BytesN<32>, usdc: Address, admin: Address, insurance: Address) {
        let share_contract = create_contract(&e, token_wasm_hash, &usdc);
        token::Client::new(&e, &share_contract).initialize(
            &e.current_contract_address(),
            &7u32,
            &"GF Yield-bearing USDC".into_val(&e),
            &"gfUSDC".into_val(&e),
        );

        write_administrator(&e, &admin);
        put_token_usdc(&e, usdc);
        put_token_share(&e, share_contract);
        put_insurance_address(&e, insurance);
        put_total_shares(&e, 0);
    }

    // Deposit and get shares
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

    // Withdraw based on shares amount
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

    // Whitelist borrower's address after their to be able to request financing
    fn whitelist(e: Env, address: Address) {
        read_administrator(&e).require_auth();
        e.storage().instance().set(&AdminDataKey::WHITELISTED(address), &true);
    }

    fn request_loan(e: Env, from: Address, invoice_amount: i128, inv_no: u32, fee_rate: i128, repayment_date: i128) {
        // Borrower needs to authorize the loan request
        from.require_auth();
        // Check if the borrower is whitelisted
        let whitelist = e.storage().instance().get(&AdminDataKey::WHITELISTED(from)).unwrap();
        if !whitelist {
            panic!("borrower is not whitelisted");
        }

        let loan = LoanDetails {
            who: from,
            fee_rate,
            invoice_amount,
            loan_amount: invoice_amount * (100 - fee_rate) / 100,
            repayment_date,
            approved: false,
            released: false,
        };

        e.storage().instance().set(&AdminDataKey::INVNO(inv_no), &loan);

        env.events()
            .publish((AdminDataKey::INVNO(inv_no), symbol_short!("loan_request")), loan);
    }

    fn approve_loan(e: Env, inv_no: u32) {
        let admin = read_administrator(&e);
        admin.require_auth();

        let mut loan = e.storage().instance().get(&AdminDataKey::INVNO(inv_no)).unwrap();
        loan.approved = true;

        e.storage().instance().set(&AdminDataKey::INVNO(inv_no), &loan);

        env.events()
            .publish((AdminDataKey::INVNO(inv_no), symbol_short!("loan_approved")), loan);
    }

    fn release_loan(e: Env, inv_no: u32) {
        let mut loan = e.storage().instance().get(&AdminDataKey::INVNO(inv_no)).unwrap();
        loan.who.require_auth();
        // Must be approved
        if !loan.approved {
            panic!("loan not approved");
        }

        // Release fund to the borrower
        transfer(&e, get_usdc(&e), loan.who, loan.loan_amount);
        loan.released = true;

        // Update loan details and total loan amount
        e.storage().instance().set(&AdminDataKey::INVNO(inv_no), &loan);
        put_total_loan_amount(&e, get_total_loan_amount(&e) + loan.loan_amount);
        put_total_outstanding_loan(&e, get_total_outstanding_loan(&e) + loan.loan_amount);

        env.events()
            .publish((AdminDataKey::INVNO(inv_no), symbol_short!("loan_released")), loan);
    }

    fn repay_loan(e: Env, inv_no: u32) {
        let mut loan = e.storage().instance().get(&AdminDataKey::INVNO(inv_no)).unwrap();
        loan.who.require_auth();
        // Must be released loan and repayment date reached
        if !loan.released && loan.repayment_date < e.ledger().timestamp() {
            panic!("loan not released or repayment date not reached");
        }

        // Repay the loan
        let usdc_client = token::Client::new(&e, &get_usdc(&e));
        usdc_client.transfer(&loan.who, &e.current_contract_address(), &loan.invoice_amount);

        // Update loan details and total loan amount
        loan.repaid = true;
        e.storage().instance().set(&AdminDataKey::INVNO(inv_no), &loan);

        // Update total outstanding loan amount
        put_total_outstanding_loan(&e, get_total_outstanding_loan(&e) - loan.loan_amount);

        // Update fees earned and transfer portion to insurance
        let fees_earned = get_fees_earned(&e);
        let fee = (loan.invoice_amount - loan.loan_amount) * (100 - FEES_PORTION_FOR_INSURANCE) / 100;
        put_fees_earned(&e, fees_earned + fee);
        let fees_to_insurance = (loan.invoice_amount - loan.loan_amount) - fee;
        usdc_client.transfer(&e.current_contract_address(), &get_insurance_address(&e), &fees_to_insurance);
    }

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
