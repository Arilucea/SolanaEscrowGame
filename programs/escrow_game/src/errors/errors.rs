use anchor_lang::error_code;

#[error_code]
pub enum Errors {
    #[msg("Escrow deal not available")]
    NotAvailable,
    #[msg("Cannot join game")]
    PriceTooDifferent,
    #[msg("Escrow already accepted")]
    CannotWithdraw,
    #[msg("Not creator of escrow")]
    NotEscrowCreator,
    #[msg("Not participant in the escrow")]
    NotSide,
    #[msg("Not accepted escrow")]
    NotAccepted,
    #[msg("No escrow winner yet")]
    NotFinished,
}
