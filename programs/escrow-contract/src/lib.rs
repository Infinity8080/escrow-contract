pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HNTCQ1LqJAswxZUkTY5hdfw99nNoen6VERK9vxvb5t4g");

#[program]
pub mod escrow_contract {
    use super::*;
    pub fn make(ctx: Context<Make>, seed: u64, amount: u64, bump: u8) -> Result<()> {
        instructions::make::handler(ctx, seed, amount, bump)
    }
}
