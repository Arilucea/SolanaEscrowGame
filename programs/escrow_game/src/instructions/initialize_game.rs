use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{Escrow, Price};

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(mut,
        seeds = [b"escrow".as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = player,
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    // #[account(mut)]
    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    #[account(
        init,
        payer = player,
        associated_token::mint = usdc_mint,
        associated_token::authority = escrow,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub oracle_mock: Account<'info, Price>,
}

pub fn initialize_game(ctx: Context<InitializeGame>, is_leg_up: bool) -> Result<()> {
    let eth_price = ctx.accounts.oracle_mock.price;
    let eth_exponent = ctx.accounts.oracle_mock.exponent;
    let entry_fee = ctx.accounts.escrow.entry_fee;

    ctx.accounts.escrow.new(
        eth_price,
        eth_exponent,
        is_leg_up,
        ctx.accounts.player.key(),
        ctx.accounts.player_token_account.key(),
    )?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.player_token_account.to_account_info(),
                to: ctx.accounts.escrow_token_account.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        ),
        entry_fee,
    )?;

    Ok(())
}
