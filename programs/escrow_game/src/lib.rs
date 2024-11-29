use anchor_lang::prelude::*;

declare_id!("Hgfo4eNKuhKEidMiAjThV81KuEUtcjKBXXDUNuRgkfxy");

pub mod cpi;
pub mod data;
pub mod errors;
pub mod instructions;

use cpi::*;
use data::*;
use errors::*;
use instructions::*;

#[program]
pub mod escrow_game {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, entry_fee: u64, seed: u64) -> Result<()> {
        instructions::initialize(ctx, entry_fee, seed)
    }

    pub fn initialize_game(ctx: Context<InitializeGame>, is_leg_up: bool) -> Result<()> {
        instructions::initialize_game(ctx, is_leg_up)
    }

    pub fn accept_game(ctx: Context<AcceptGame>) -> Result<()> {
        instructions::accept_game(ctx)
    }

    pub fn withdraw_game(ctx: Context<WithdrawGame>) -> Result<()> {
        instructions::withdraw_game(ctx)
    }

    pub fn close_game(ctx: Context<CloseGame>) -> Result<()> {
        instructions::close_game(ctx)
    }

    pub fn set_eth_price(ctx: Context<SetPrice>, eth_price: i64, eth_exponent: i32) -> Result<()> {
        cpi::set_eth_price(ctx, eth_price, eth_exponent)
    }
}
