use anchor_lang::prelude::*;

use super::Status;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub eth_price: i64,
    pub eth_exponent: i32,
    pub entry_fee: u64,

    pub is_leg_up: bool,
    pub leg_up: Pubkey,
    pub token_account_leg_up: Pubkey,
    pub leg_down: Pubkey,
    pub token_account_leg_down: Pubkey,

    pub status: Status,

    pub seed: u64,
    pub bump: u8,
}

impl<'info> Escrow {
    pub fn initialize(&mut self, fee: u64, seed: u64, bump: u8) -> Result<()> {
        self.entry_fee = fee;
        self.seed = seed;
        self.bump = bump;
        Ok(())
    }

    pub fn new(
        &mut self,
        eth_price: i64,
        eth_exponent: i32,
        is_leg_up: bool,
        player: Pubkey,
        token_account: Pubkey,
    ) -> Result<()> {
        self.eth_price = eth_price;
        self.eth_exponent = eth_exponent;
        self.is_leg_up = is_leg_up;

        if is_leg_up {
            self.leg_up = player;
            self.token_account_leg_up = token_account;
        } else {
            self.leg_down = player;
            self.token_account_leg_down = token_account;
        }

        self.status = Status::Initialize;

        Ok(())
    }

    pub fn accept(&mut self, owner: Pubkey, token_account: Pubkey) -> Result<()> {
        if self.is_leg_up {
            self.leg_down = owner;
            self.token_account_leg_down = token_account;
        } else {
            self.leg_up = owner;
            self.token_account_leg_up = token_account;
        }

        self.status = Status::Accepted;
        Ok(())
    }

    pub fn close_it(&mut self) -> Result<()> {
        self.status = Status::Closed;
        Ok(())
    }
}
