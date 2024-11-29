use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::get_feed_id_from_hex;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::Escrow;

pub fn oracle_price(ctx: Context<OraclePrice>) -> Result<()> {
    let price_update = &mut ctx.accounts.price_update;
    let maximum_age: u64 = 30;
    let feed_id: [u8; 32] = get_feed_id_from_hex(
        "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace", //42amVS4KgzR9rA28tkVYqVXjq9Qa8dcZQMbH5EYFX6XC
    )?;
    let price = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
    msg!(
        "The price is ({} ± {}) * 10^{}",
        price.price,
        price.conf,
        price.exponent
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct OraclePrice<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub price_update: Account<'info, PriceUpdateV2>,
}

pub fn normalize_price(
    escrow: &mut Escrow,
    mut current_price: i64,
    current_exponent: i32,
) -> Result<[i64; 2]> {
    let mut initial_price = escrow.eth_price;
    let initial_exponent = escrow.eth_exponent;

    if current_exponent != initial_exponent {
        if current_exponent > initial_exponent {
            let mutliplier = current_exponent - initial_exponent;
            current_price = current_price * (10 as i64).pow(mutliplier.try_into().unwrap());
        } else {
            let mutliplier = initial_exponent - current_exponent;
            initial_price = current_price * (10 as i64).pow(mutliplier.try_into().unwrap());
        }
    }

    Ok([initial_price, current_price])
}
