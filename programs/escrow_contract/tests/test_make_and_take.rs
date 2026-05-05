use ::escrow_contract::accounts;
use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::{AnchorLiteSVM, Signer};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token::spl_token;
use litesvm_utils::AssertionHelpers;
use litesvm_utils::TestHelpers;

use crate::escrow_contract::client::args;

declare_program!(escrow_contract);

#[test]
fn make_and_test() {
    const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/escrow_contract.so");

    let program_id = self::escrow_contract::ID;
    let mut ctx = AnchorLiteSVM::build_with_program(program_id, PROGRAM_BYTES);
    //Test accounts
    let maker = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let taker = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    //Create mint accounts
    let mint_a = ctx.svm.create_token_mint(&maker, 9).unwrap();
    let mint_b = ctx.svm.create_token_mint(&maker, 9).unwrap();

    //Create ATAs

    let maker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &maker)
        .unwrap();
    ctx.svm
        .mint_to(&mint_a.pubkey(), &maker_ata_a, &maker, 5_000_000_000)
        .unwrap();
    ctx.svm.assert_token_balance(&maker_ata_a, 5_000_000_000);

    //Test instruction

    let seed: i32 = 42;
    let escrow_pda = ctx.svm.get_pda(
        &[b"escrow", maker.pubkey().as_ref(), &seed.to_le_bytes()],
        &program_id,
    );
    let vault = get_associated_token_address(&escrow_pda, &mint_a.pubkey());
    let ix = ctx
        .program()
        .accounts(accounts::Make {
            escrow: escrow_pda,
            maker: maker.pubkey(),
            mint_a: mint_a.pubkey(),
            maker_ata_a,
            associated_token_program: spl_token::id(),
            mint_b: mint_b.pubkey(),
            system_program: system_program::id(),
            token_program: spl_associated_token_account::program::id(),
            vault,
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&maker])
        .unwrap()
        .assert_success();
}
