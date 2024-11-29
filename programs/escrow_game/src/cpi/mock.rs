use anchor_lang::prelude::*;

pub fn set_eth_price(ctx: Context<SetPrice>, eth_price: i64, eth_exponent: i32) -> Result<()> {
    ctx.accounts.oracle_mock.price = eth_price;
    ctx.accounts.oracle_mock.exponent = eth_exponent;

    msg!("Eth price set to {}, decimals {}", eth_price, eth_exponent);
    Ok(())
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    #[account(init_if_needed,
        payer = payer,
        space = 8 + Price::INIT_SPACE)]
    pub oracle_mock: Account<'info, Price>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct Price {
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
}
