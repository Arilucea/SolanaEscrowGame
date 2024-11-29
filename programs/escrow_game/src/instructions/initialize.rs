use anchor_lang::prelude::*;

use crate::data;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    #[account(init,
        payer = player, 
        space = 8 + data::Escrow::INIT_SPACE,        
        seeds = [b"escrow".as_ref(), &seed.to_le_bytes()],
        bump)]
    pub escrow: Account<'info, data::Escrow>,

    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, seed: u64, entry_fee: u64) -> Result<()> {
    ctx.accounts.escrow.initialize(entry_fee, seed, ctx.bumps.escrow)?;
    msg!("Escrow account initialized.");
    Ok(())
}
