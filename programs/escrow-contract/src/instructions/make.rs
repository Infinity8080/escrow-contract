use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::EscrowAccount;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init,
        payer=maker,
        space=EscrowAccount::INIT_SPACE + EscrowAccount::DISCRIMINATOR.len(),
        seeds=[b"escrow",maker.key().as_ref(),seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, EscrowAccount>,

    #[account(
        mint::token_program=token_program,
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program=token_program,
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
     mut,
     associated_token::mint = mint_a,
            associated_token::authority = maker,
            associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
     init,
     payer=maker,
     associated_token::mint = mint_a,
            associated_token::authority = escrow,
            associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    fn populate_escrow(&mut self, seed: u64, amount: u64, bump: u8) -> Result<()> {
        self.escrow.set_inner(EscrowAccount {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            recieve: amount,
            bump: bump,
        });
        Ok(())
    }
    fn deposit_token(&self, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_address = self.token_program.key();
        let cpi_context = CpiContext::new(cpi_address, cpi_accounts);
        transfer_checked(cpi_context, amount, self.mint_a.decimals)?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Make>, seed: u64, amount: u64, recieve: u64) -> Result<()> {
    require_gt!(amount, 0);
    require_gt!(recieve, 0);
    ctx.accounts
        .populate_escrow(seed, recieve, ctx.bumps.escrow)?;
    ctx.accounts.deposit_token(amount)?;
    Ok(())
}
