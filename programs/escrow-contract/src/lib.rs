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

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }
}
