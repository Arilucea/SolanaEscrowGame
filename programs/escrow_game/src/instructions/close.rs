use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{normalize_price, Errors, Escrow, Price, Status};

pub fn close_game(ctx: Context<CloseGame>) -> Result<()> {
    let current_price = ctx.accounts.oracle_mock.price;
    let current_exponent = ctx.accounts.oracle_mock.exponent;
    let escrow = &mut ctx.accounts.escrow;

    let [initial_price, current_price] = normalize_price(escrow, current_price, current_exponent)?;

    msg!("Prices {} {}", initial_price, current_price);

    let mut winner: Option<Pubkey> = None;
    let mut winner_token_account: Option<Pubkey> = None;
    let entry_fee = escrow.entry_fee;

    if current_price > initial_price {
        if current_price * 100 / initial_price >= 105 {
            winner = Some(escrow.leg_up);
            winner_token_account = Some(escrow.token_account_leg_up);
            msg!("Leg up winner {}", winner.unwrap());
        }
    } else {
        if initial_price * 100 / current_price >= 105 {
            winner = Some(escrow.leg_down);
            winner_token_account = Some(escrow.token_account_leg_down);
            msg!("Leg down winner {}", winner.unwrap());
        }
    }

    require!(winner.is_some(), Errors::NotFinished);
    require!(escrow.status == Status::Accepted, Errors::NotAccepted);
    require!(
        escrow.leg_up == ctx.accounts.player.key() || escrow.leg_down == ctx.accounts.player.key(),
        Errors::NotSide
    );
    msg!(
        "Winner {}, sended {}",
        winner_token_account.unwrap().key(),
        ctx.accounts.player_token_account.key()
    );
    require!(
        winner_token_account.unwrap().key() == ctx.accounts.player_token_account.key(),
        Errors::NotSide
    );

    escrow.close_it()?;

    let signer_seeds: [&[&[u8]]; 1] =
        [&[b"escrow", &escrow.seed.to_le_bytes()[..], &[escrow.bump]]];

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
        entry_fee * 2,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct CloseGame<'info> {
    #[account(mut,
        seeds = [b"escrow".as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump)]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub player: Signer<'info>,

    pub usdc_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,

    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = player,
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = escrow,)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub oracle_mock: Account<'info, Price>,
}
