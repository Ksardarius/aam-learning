use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};

use crate::{errors::AamErrorCode, state::PoolState};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        mut,
        associated_token::mint = token_from_mint,
        associated_token::authority = payer
    )]
    pub user_token_from_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_to_mint,
        associated_token::authority = payer
    )]
    pub user_token_to_account: Account<'info, TokenAccount>,

    pub token_from_mint: Account<'info, Mint>,

    pub token_to_mint: Account<'info, Mint>,

    #[account(mut, constraint = pool_token_a_vault.key() == pool_state.token_a_vault @ AamErrorCode::InvalidVault)]
    pub pool_token_a_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint = pool_token_b_vault.key() == pool_state.token_b_vault @ AamErrorCode::InvalidVault)]
    pub pool_token_b_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"pool_authority", pool_state.key().as_ref()],
        bump = pool_state.authority_bump, // Use the bump stored in pool_state
    )]
    /// CHECK: This is a PDA used as an authority, not a data account.
    pub pool_authority_pda: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_output_amount: u64) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;
    let from_mint_key = ctx.accounts.token_from_mint.key();
    let to_mint_key = ctx.accounts.token_to_mint.key();
    let pool_a_mint = pool_state.token_a_mint;
    let pool_b_mint = pool_state.token_b_mint;

    // Initial validations
    require!(from_mint_key != to_mint_key, AamErrorCode::SameTokenSwap); // Cannot swap a token for itself
    require!(amount_in > 0, AamErrorCode::ZeroAmount); // Input amount must be greater than zero

    // Determine swap direction and assign correct vaults/reserves
    let (reserve_x, reserve_y, from_vault_account_info, to_vault_account_info) =
        if from_mint_key == pool_a_mint {
            // Swapping A for B: user sends A, receives B
            require!(to_mint_key == pool_b_mint, AamErrorCode::InvalidMint);
            (
                ctx.accounts.pool_token_a_vault.amount,
                ctx.accounts.pool_token_b_vault.amount,
                ctx.accounts.pool_token_a_vault.to_account_info(),
                ctx.accounts.pool_token_b_vault.to_account_info(),
            )
        } else if from_mint_key == pool_b_mint {
            // Swapping B for A: user sends B, receives A
            require!(to_mint_key == pool_a_mint, AamErrorCode::InvalidMint);
            (
                ctx.accounts.pool_token_b_vault.amount,
                ctx.accounts.pool_token_a_vault.amount,
                ctx.accounts.pool_token_b_vault.to_account_info(),
                ctx.accounts.pool_token_a_vault.to_account_info(),
            )
        } else {
            // from_mint_key is neither A nor B of this pool
            return err!(AamErrorCode::InvalidMint);
        };

    // Ensure sufficient liquidity in the pool for the trade
    require!(
        reserve_x > 0 && reserve_y > 0,
        AamErrorCode::InsufficientLiquidity
    );

    // calculate swap amount
    let fee_bps: u128 = pool_state.trading_fees as u128; // Assuming trading_fees is in basis points (e.g., 30 for 0.3%)
    let hundred_percent_bps: u128 = 10_000; // Total basis points for 100%

    let amount_in_u128 = amount_in as u128;
    let fee_amount = amount_in_u128 * fee_bps / hundred_percent_bps;
    let amount_in_after_fees = amount_in_u128 - fee_amount;

    // Constant product formula: amount_out = (reserve_y * amount_in_after_fees) / (reserve_x + amount_in_after_fees)
    let numerator = (reserve_y as u128) * amount_in_after_fees;
    let denominator = (reserve_x as u128) + amount_in_after_fees;

    let amount_out = (numerator / denominator) as u64;

    require!(
        amount_out >= minimum_output_amount,
        AamErrorCode::MinimumOutputBalanceExceed
    );

    let cpi_accounts_from = Transfer {
        from: ctx.accounts.user_token_from_account.to_account_info(),
        to: from_vault_account_info.clone(),
        authority: ctx.accounts.payer.to_account_info(),
    };

    let cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_from,
    );

    token::transfer(cpi_context, amount_in)?;

    let key = pool_state.key();
    let authority_seeds = [
        b"pool_authority",
        key.as_ref(),
        &[pool_state.authority_bump], // Use the stored bump directly!
    ];
    let authority_seeds = [&authority_seeds[..]];

    let cpi_accounts_to = Transfer {
        from: to_vault_account_info.clone(),
        to: ctx.accounts.user_token_to_account.to_account_info(),
        authority: ctx.accounts.pool_authority_pda.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_to,
        &authority_seeds,
    );

    token::transfer(cpi_context, amount_out)?;

    pool_state.token_a_reserves = ctx.accounts.pool_token_a_vault.amount;
    pool_state.token_b_reserves = ctx.accounts.pool_token_b_vault.amount;

    Ok(())
}
