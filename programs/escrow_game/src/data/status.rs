use anchor_lang::prelude::*;
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum Status {
    Initialize,
    Accepted,
    Closed,
}
