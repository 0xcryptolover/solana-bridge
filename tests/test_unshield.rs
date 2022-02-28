#![cfg(feature = "test-bpf")]

use {
    assert_matches::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    solana_sdk::{signature::Signer, transaction::Transaction},
    solana_validator::test_validator::*,
};
use solana_bridge::{
    instruction::BridgeInstruction,
    processor::process_instruction,
    state::{IncognitoProxy, MAX_BEACON_ADDRESSES},
};
use spl_token::instruction::approve;
use solana_program_test::*;

#[tokio::test]
async fn test_unshield_success() {
    // new vault program id
    let program_id = Pubkey::new_unique();
    // new shield maker account
    let shield_maker = Pubkey::new_unique();
    // incognito proxy account
    let incognito_proxy = Pubkey::new_unique();

    let mut test = ProgramTest::new(
        "incognito_vault",
        program_id,
        processor!(process_instruction),
    );

    // limit to track compute unit increase
    test.set_compute_max_units(38_000);

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