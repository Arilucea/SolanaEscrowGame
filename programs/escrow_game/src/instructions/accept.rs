use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{normalize_price, Errors, Escrow, Price, Status};

pub fn accept_game(ctx: Context<AcceptGame>) -> Result<()> {
    require!(
        ctx.accounts.escrow.status == Status::Initialize,
        Errors::NotAvailable
    );

    let current_price = ctx.accounts.oracle_mock.price;
    let current_exponent = ctx.accounts.oracle_mock.exponent;
    let escrow = &mut ctx.accounts.escrow;

    let [initial_price, current_price] = normalize_price(escrow, current_price, current_exponent)?;
    msg!("Prices {} {}", initial_price, current_price);
    let entry_fee = ctx.accounts.escrow.entry_fee;

    if current_price > initial_price {
        require!(
            current_price * 100 / initial_price < 101,
            Errors::PriceTooDifferent
        )
    } else {
        require!(
            initial_price * 100 / current_price < 101,
            Errors::PriceTooDifferent
        )
    }

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

    ctx.accounts.escrow.accept(
        ctx.accounts.player.key(),
        ctx.accounts.player_token_account.key(),
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct AcceptGame<'info> {
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

    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    #[account(mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = escrow,)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub oracle_mock: Account<'info, Price>,
}
