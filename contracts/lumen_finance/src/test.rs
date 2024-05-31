#![cfg(test)]
extern crate std;

use crate::{token, LumenFinance, LumenFinanceClient};

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Ledger},
    Address, BytesN, Env, IntoVal
};

fn create_token_contract<'a>(e: &Env, admin: &Address) -> token::Client<'a> {
    token::Client::new(e, &e.register_stellar_asset_contract(admin.clone()))
}

fn create_lumenfinance_contract<'a>(
    e: &Env,
    token_wasm_hash: &BytesN<32>,
    usdc: &Address,
    admin: &Address,
    insurance: &Address,
) -> LumenFinanceClient<'a> {
    let lumen = LumenFinanceClient::new(e, &e.register_contract(None, LumenFinance));
    lumen.initialize(token_wasm_hash, usdc, admin, insurance);
    lumen
}

fn install_token_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "token/soroban_token_contract.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
}

#[test]
fn test_deposit() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    // Balance before
    assert_eq!(usdc.balance(&depositor), 1000);
    assert_eq!(usdc.balance(&lumen.address), 0);

    lumen.deposit(&depositor, &1000);

    // Balance after
    assert_eq!(token_share.balance(&depositor), 1000);
    assert_eq!(usdc.balance(&depositor), 0);
    assert_eq!(usdc.balance(&lumen.address), 1000);
}

#[test]
fn test_withdraw() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let depositor2 = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);
    usdc.mint(&depositor2, &200);
    lumen.deposit(&depositor2, &200);

    // Withdraw
    lumen.withdraw(&depositor, &500);

    // Check balance
    assert_eq!(token_share.balance(&depositor), 500);
    assert_eq!(usdc.balance(&depositor), 500);
}

#[test]
#[should_panic]
fn test_request_loan_not_whitelisted() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.request_loan(&borrower, &800, &231u32, &1745156);
}

#[test]
fn test_request_loan() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.whitelist(&borrower);
    lumen.request_loan(&borrower, &800, &231u32, &1745156);
}

#[test]
fn test_approve_loan() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.whitelist(&borrower);
    lumen.request_loan(&borrower, &800, &231u32, &1745156);
    lumen.approve_loan(&231u32, &10i128);
}

#[test]
#[should_panic]
fn test_approve_loan_bad_fee_rate() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.whitelist(&borrower);
    lumen.request_loan(&borrower, &800, &231u32, &1745156);
    lumen.approve_loan(&231u32, &101i128);
}

#[test]
fn test_claim_loan() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.whitelist(&borrower);
    lumen.request_loan(&borrower, &800, &231u32, &1745156);
    lumen.approve_loan(&231u32, &10i128);
    lumen.claim_loan(&231u32);

    let amount_after_fee: i128 = 800 * 90/ 100;
    assert_eq!(usdc.balance(&borrower), amount_after_fee);
}

#[test]
#[should_panic]
fn test_repay_loan_not_time_yet() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.whitelist(&borrower);
    lumen.request_loan(&borrower, &800, &231u32, &1745156);
    lumen.approve_loan(&231u32, &10i128);
    lumen.claim_loan(&231u32);
    lumen.repay_loan(&231u32);
}

#[test]
fn test_repay_loan() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.whitelist(&borrower);
    lumen.request_loan(&borrower, &800, &231u32, &1745156);
    lumen.approve_loan(&231u32, &10i128);
    lumen.claim_loan(&231u32);
    let amount_after_fee: i128 = 800 * 90/ 100; // same to loan.loan_amount
    assert_eq!(usdc.balance(&borrower), amount_after_fee);

    // Advance the time
    e.ledger().with_mut(|li| {
        li.timestamp = 1745156 + 1;
    });

    // Check repayment and portion that goes to insurance
    let loan = lumen.get_loan_details(&231u32);
    let fees = loan.invoice_amount - loan.loan_amount;
    usdc.mint(&borrower, &fees); // mint back the fees
    let (_, insurance_fee) = lumen.repay_loan(&231u32);
    assert_eq!(usdc.balance(&borrower), 0);
    assert_eq!(usdc.balance(&lumen.address), 1000 + fees - insurance_fee);
    assert_eq!(lumen.get_fees_earned(), fees - insurance_fee);
}

#[test]
fn test_withdraw_with_fees_earned() {
    let e = Env::default();
    e.mock_all_auths();

    let mut admin = Address::generate(&e);
    let insurance = Address::generate(&e);
    let depositor = Address::generate(&e);
    let borrower = Address::generate(&e);

    let mut usdc = create_token_contract(&e, &admin);

    let lumen = create_lumenfinance_contract(
        &e,
        &install_token_wasm(&e),
        &usdc.address,
        &admin,
        &insurance,
    );

    let token_share = token::Client::new(&e, &lumen.share_id());

    usdc.mint(&depositor, &1000);
    lumen.deposit(&depositor, &1000);

    lumen.whitelist(&borrower);
    lumen.request_loan(&borrower, &800, &231u32, &1745156);
    lumen.approve_loan(&231u32, &10i128);
    lumen.claim_loan(&231u32);

    // Advance the time
    e.ledger().with_mut(|li| {
        li.timestamp = 1745156 + 1;
    });

    // Check repayment and portion that goes to insurance
    let loan = lumen.get_loan_details(&231u32);
    let fees = loan.invoice_amount - loan.loan_amount;
    usdc.mint(&borrower, &fees); // mint back the fees
    let (_, insurance_fee) = lumen.repay_loan(&231u32);

    // Check withdrawal with earnings
    lumen.withdraw(&depositor, &1000);
    assert_eq!(usdc.balance(&depositor), 1000 + fees - insurance_fee);
}
