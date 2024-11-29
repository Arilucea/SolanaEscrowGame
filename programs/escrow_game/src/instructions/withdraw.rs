use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{Errors, Escrow, Status};

pub fn withdraw_game(ctx: Context<WithdrawGame>) -> Result<()> {
    require!(
        ctx.accounts.escrow.status == Status::Initialize
            || ctx.accounts.escrow.status == Status::Closed,
        Errors::NotAvailable
    );

    let escrow = &mut ctx.accounts.escrow;
    let entry_fee = escrow.entry_fee;
    let owner: Pubkey;
    if escrow.is_leg_up {
        owner = escrow.leg_up;
    } else {
        owner = escrow.leg_down;
    }

    let signer_seeds: [&[&[u8]]; 1] =
        [&[b"escrow", &escrow.seed.to_le_bytes()[..], &[escrow.bump]]];

    if ctx.accounts.escrow.status == Status::Initialize {
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_token_account.to_account_info(),
                    to: ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            entry_fee,
        )?;
    }

    require_keys_eq!(owner, ctx.accounts.player.key(), Errors::NotEscrowCreator);
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawGame<'info> {
    #[account(mut, close = player,
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
    #[account(mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = escrow,)]
    pub escrow_token_account: Account<'info, TokenAccount>,
}
