use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, transfer_checked, CloseAccount, TransferChecked},
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::EscrowAccount;

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(
        mut,
        close=maker,
        has_one = mint_a,
        has_one = mint_b,
        has_one = maker,
        seeds=[b"escrow",maker.key().as_ref(),escrow.seed.to_le_bytes().as_ref()],
        bump=escrow.bump
    )]
    pub escrow: Box<Account<'info, EscrowAccount>>,
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer=taker,
        associated_token::mint = mint_a,
        associated_token::authority= taker,
        associated_token::token_program=token_program
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer=taker,
        associated_token::mint = mint_b,
        associated_token::authority= taker,
        associated_token::token_program=token_program
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer=taker,
        associated_token::mint = mint_b,
        associated_token::authority= maker,
        associated_token::token_program=token_program
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,


    #[account(
        associated_token::mint = mint_a,
        associated_token::authority= escrow,
        associated_token::token_program=token_program
    )]
    pub vault:Box< InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Take<'info> {
    fn transfer_to_maker(&self) -> Result<()> {
        let cpi_accounts = TransferChecked {
            mint: self.mint_b.to_account_info(),
            from: self.taker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
        };
        let cpi_program = self.token_program.key();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        transfer_checked(cpi_context, self.escrow.recieve, self.mint_b.decimals)?;
        Ok(())
    }

    fn withdraw_and_close_vault(&self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        //Transfer
        let cpi_accounts = TransferChecked {
            mint: self.mint_a.to_account_info(),
            from: self.vault.to_account_info(),
            authority: self.escrow.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
        };
        let cpi_program = self.token_program.key();
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);
        transfer_checked(cpi_context, self.vault.amount, self.mint_a.decimals)?;

        close_account(CpiContext::new_with_signer(
            self.token_program.key(),
            CloseAccount {
                // signer's associated token account
                account: self.vault.to_account_info(),
                // where to send the SOL
                destination: self.maker.to_account_info(),
                authority: self.escrow.to_account_info(),
            },
            &signer_seeds,
        ))?;

        Ok(())

        //Close Account
    }
}

pub fn handler(ctx: Context<Take>) -> Result<()> {
    ctx.accounts.transfer_to_maker()?;
    ctx.accounts.withdraw_and_close_vault()?;
    Ok(())
}
