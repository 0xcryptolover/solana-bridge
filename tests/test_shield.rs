// #![cfg(feature = "test-bpf")]
mod helpers;

use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    secp256k1_recover::Secp256k1Pubkey,
};
use solana_bridge::{
    instruction::BridgeInstruction,
    processor::process_instruction,
    state::{IncognitoProxy, MAX_BEACON_ADDRESSES},
};
use spl_token::{
    instruction::approve,
    state::{Account as Token, AccountState, Mint},
};

use crate::helpers::add_packable_account;

#[tokio::test]
async fn test_shield_success() {

    // new shield maker account
    let shield_maker = Keypair::new();
    // shield maker token account
    let shield_maker_token_account = Pubkey::new_unique();
    // new vault program id
    let program_id = Pubkey::new_unique();
    // incognito proxy account
    let incognito_proxy = Pubkey::new_unique();
    // vault token account
    let vault_token_account = Pubkey::new_unique();
    // mint pub key
    let token_mint_pub_key = Pubkey::new_unique();
    // token program spl_token::id()
    // new vault account id
    let vault_account_id = Pubkey::new_unique();

    let deposit_amount: u64 = 100_0000;
    let (incognito_proxy_authority_key, bump_seed) =
        Pubkey::find_program_address(&[incognito_proxy.as_ref()], &spl_token::id());

    let mut test = ProgramTest::new(
        "bridge_solana",
        program_id,
        processor!(process_instruction),
    );

    // limit to track compute unit increase
    test.set_compute_max_units(38_000);
    // todo add beacon
    let mut vec = Vec::new();
    add_packable_account(
        &mut test,
        incognito_proxy,
        u32::MAX as u64,
        &IncognitoProxy::new(IncognitoProxy {
            is_initialized: true,
            bump_seed,
            vault: vault_account_id,
            beacons: vec,
        }),
        &spl_token::id(),
    );

    // init shield maker token account
     add_packable_account(
        &mut test,
        shield_maker_token_account,
        u32::MAX as u64,
        &Token {
            mint: token_mint_pub_key,
            owner: shield_maker.pubkey(),
            amount: deposit_amount,
            state: AccountState::Initialized,
            ..Token::default()
        },
        &spl_token::id(),
    );

    // init vault token account
    add_packable_account(
        &mut test,
        vault_token_account,
        u32::MAX as u64,
        &Token {
            mint: token_mint_pub_key,
            owner: incognito_proxy_authority_key,
            amount: 0,
            state: AccountState::Initialized,
            ..Token::default()
        },
        &spl_token::id(),
    );


    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // let mut transaction = Transaction::new_with_payer(
    //     &[
    //         approve(
    //             &spl_token::id(),
    //             &sol_test_reserve.user_collateral_pubkey,
    //             &user_transfer_authority.pubkey(),
    //             &user_accounts_owner.pubkey(),
    //             &[],
    //             SOL_DEPOSIT_AMOUNT_LAMPORTS,
    //         )
    //             .unwrap(),
    //         deposit_obligation_collateral(
    //             spl_token_lending::id(),
    //             SOL_DEPOSIT_AMOUNT_LAMPORTS,
    //             sol_test_reserve.user_collateral_pubkey,
    //             sol_test_reserve.collateral_supply_pubkey,
    //             sol_test_reserve.pubkey,
    //             test_obligation.pubkey,
    //             lending_market.pubkey,
    //             test_obligation.owner,
    //             user_transfer_authority.pubkey(),
    //         ),
    //     ],
    //     Some(&payer.pubkey()),
    // );
}