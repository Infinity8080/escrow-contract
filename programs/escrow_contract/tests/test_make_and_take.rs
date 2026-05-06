use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::{AnchorLiteSVM, Signer};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token::spl_token;
use litesvm_utils::AssertionHelpers;
use litesvm_utils::TestHelpers;

use crate::escrow_contract::client::accounts;
use crate::escrow_contract::client::args;

declare_program!(escrow_contract);

#[test]
fn make_and_test() {
    const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/escrow_contract.so");

    let program_id = self::escrow_contract::ID;
    let mut ctx = AnchorLiteSVM::build_with_program(program_id, PROGRAM_BYTES);
    //Test accounts
    let maker = ctx.svm.create_funded_account(100_000_000_000).unwrap();
    let taker = ctx.svm.create_funded_account(100_000_000_000).unwrap();

    //Create mint accounts
    let mint_a = ctx.svm.create_token_mint(&maker, 9).unwrap();
    let mint_b = ctx.svm.create_token_mint(&maker, 9).unwrap();

    //Create ATAs

    let maker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &maker)
        .unwrap();
    ctx.svm
        .mint_to(&mint_a.pubkey(), &maker_ata_a, &maker, 50_000_000_000)
        .unwrap();

    let maker_ata_b = ctx
        .svm
        .create_associated_token_account(&mint_b.pubkey(), &maker)
        .unwrap();

    let taker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &taker)
        .unwrap();

    let taker_ata_b = ctx
        .svm
        .create_associated_token_account(&mint_b.pubkey(), &taker)
        .unwrap();
    ctx.svm
        .mint_to(&mint_b.pubkey(), &taker_ata_b, &maker, 5_000_000_000)
        .unwrap();

    ctx.svm.assert_token_balance(&maker_ata_a, 50_000_000_000);

    //Test instructions

    // 1. Make
    let seed: u64 = 42;
    let escrow_pda = ctx.svm.get_pda(
        &[b"escrow", maker.pubkey().as_ref(), &seed.to_le_bytes()],
        &program_id,
    );
    let vault = get_associated_token_address(&escrow_pda, &mint_a.pubkey());
    let make_ix = ctx
        .program()
        .accounts(accounts::Make {
            escrow: escrow_pda,
            maker: maker.pubkey(),
            mint_a: mint_a.pubkey(),
            maker_ata_a,
            token_program: spl_token::id(),
            mint_b: mint_b.pubkey(),
            system_program: system_program::id(),
            associated_token_program: spl_associated_token_account::program::id(),
            vault,
        })
        .args(args::Make {
            amount: 10_000_000_000,
            recieve: 5_000_000_000,
            seed,
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(make_ix, &[&maker])
        .unwrap()
        .assert_success();

    ctx.svm.assert_token_balance(&vault, 10_000_000_000);

    //2. Take
    let take_ix = ctx
        .program()
        .accounts(accounts::Take {
            associated_token_program: spl_associated_token_account::program::id(),
            escrow: escrow_pda,
            maker: maker.pubkey(),
            maker_ata_b,
            mint_a: mint_a.pubkey(),
            mint_b: mint_b.pubkey(),
            system_program: system_program::id(),
            taker: taker.pubkey(),
            taker_ata_a,
            taker_ata_b,
            token_program: spl_token::id(),
            vault,
        })
        .args(args::Take {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(take_ix, &[&taker])
        .unwrap()
        .assert_success();

    ctx.svm.assert_account_closed(&vault);
    ctx.svm.assert_account_closed(&escrow_pda);
    ctx.svm.assert_token_balance(&maker_ata_b, 5_000_000_000);
    ctx.svm.assert_token_balance(&taker_ata_b, 0);
    ctx.svm.assert_token_balance(&taker_ata_a, 10_000_000_000);
}

#[test]
fn make_and_refund() {
    const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/escrow_contract.so");

    let program_id = self::escrow_contract::ID;
    let mut ctx = AnchorLiteSVM::build_with_program(program_id, PROGRAM_BYTES);
    //Test accounts
    let maker = ctx.svm.create_funded_account(100_000_000_000).unwrap();
    let taker = ctx.svm.create_funded_account(100_000_000_000).unwrap();

    //Create mint accounts
    let mint_a = ctx.svm.create_token_mint(&maker, 9).unwrap();
    let mint_b = ctx.svm.create_token_mint(&maker, 9).unwrap();

    //Create ATAs

    let maker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &maker)
        .unwrap();
    ctx.svm
        .mint_to(&mint_a.pubkey(), &maker_ata_a, &maker, 50_000_000_000)
        .unwrap();

    let maker_ata_b = ctx
        .svm
        .create_associated_token_account(&mint_b.pubkey(), &maker)
        .unwrap();

    let taker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &taker)
        .unwrap();

    let taker_ata_b = ctx
        .svm
        .create_associated_token_account(&mint_b.pubkey(), &taker)
        .unwrap();
    ctx.svm
        .mint_to(&mint_b.pubkey(), &taker_ata_b, &maker, 5_000_000_000)
        .unwrap();

    ctx.svm.assert_token_balance(&maker_ata_a, 50_000_000_000);

    //Test instructions

    // 1. Make
    let seed: u64 = 42;
    let escrow_pda = ctx.svm.get_pda(
        &[b"escrow", maker.pubkey().as_ref(), &seed.to_le_bytes()],
        &program_id,
    );
    let vault = get_associated_token_address(&escrow_pda, &mint_a.pubkey());
    let make_ix = ctx
        .program()
        .accounts(accounts::Make {
            escrow: escrow_pda,
            maker: maker.pubkey(),
            mint_a: mint_a.pubkey(),
            maker_ata_a,
            token_program: spl_token::id(),
            mint_b: mint_b.pubkey(),
            system_program: system_program::id(),
            associated_token_program: spl_associated_token_account::program::id(),
            vault,
        })
        .args(args::Make {
            amount: 10_000_000_000,
            recieve: 5_000_000_000,
            seed,
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(make_ix, &[&maker])
        .unwrap()
        .assert_success();

    ctx.svm.assert_token_balance(&vault, 10_000_000_000);

    //2. Take
    let refund_ix = ctx
        .program()
        .accounts(accounts::Refund {
            associated_token_program: spl_associated_token_account::program::id(),
            escrow: escrow_pda,
            maker_ata_a: maker_ata_a,
            maker: maker.pubkey(),
            mint_a: mint_a.pubkey(),
            mint_b: mint_b.pubkey(),
            system_program: system_program::id(),
            token_program: spl_token::id(),
            vault,
        })
        .args(args::Refund {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(refund_ix, &[&maker])
        .unwrap()
        .assert_success();

    ctx.svm.assert_account_closed(&vault);

    ctx.svm.assert_token_balance(&maker_ata_a, 50_000_000_000);
}
