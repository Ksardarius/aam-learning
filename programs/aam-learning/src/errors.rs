use anchor_lang::prelude::*;

#[error_code]
pub enum AamErrorCode {
    #[msg("The pool has already been initialized.")]
    AlreadyInitialized,
    #[msg("Initial liquidity must be sufficient to cover MINIMUM_LIQUIDITY.")]
    InsufficientInitialLiquidity,
    #[msg("Amounts provided for liquidity are zero.")]
    ZeroAmount,
    #[msg("Insufficient liquidity provided for existing pool ratio.")]
    InsufficientLiquidity,
    #[msg("Liquidity amounts do not match current pool ratio.")]
    LiquidityRatioMismatch,
    #[msg("Integer overflow or underflow.")]
    MathOverflow,
    #[msg("Division by zero.")]
    ZeroDivision,
    #[msg("Invalid token mint account.")]
    InvalidMint,
    #[msg("Invalid token vault account.")]
    InvalidVault,
    #[msg("Minimum output balance exceed.")]
    MinimumOutputBalanceExceed,
    #[msg("Same token swap.")]
    SameTokenSwap
}