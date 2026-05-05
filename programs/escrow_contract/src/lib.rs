pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("BrNJgohZjuj8Wru2fRhFwhccvEqjkqciRrAt4P1f5heW");

#[program]
pub mod escrow_contract {
    use super::*;
    pub fn make(ctx: Context<Make>, seed: u64, amount: u64, recieve: u64) -> Result<()> {
        instructions::make::handler(ctx, seed, amount, recieve)
    }
    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::handler(ctx)
    }
}
