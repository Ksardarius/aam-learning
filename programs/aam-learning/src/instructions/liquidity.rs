use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, Transfer},
};

use crate::{errors::AamErrorCode, state::PoolState};

// A small constant used to prevent division by zero for initial liquidity
// Uniswap V2 uses 1000 for this. We'll use 1000 for consistency in calculations.
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = payer
    )]
    pub user_token_a_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = payer
    )]
    pub user_token_b_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = payer,
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = token_a_mint.key() == pool_state.token_a_mint @ AamErrorCode::InvalidMint
    )]
    pub token_a_mint: Account<'info, Mint>,

    #[account(
        constraint = token_b_mint.key() == pool_state.token_b_mint @ AamErrorCode::InvalidMint
    )]
    pub token_b_mint: Account<'info, Mint>,

    #[account(
        mut,
        address = pool_state.lp_token_mint @ AamErrorCode::InvalidMint
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = pool_token_a_vault.key() == pool_state.token_a_vault @ AamErrorCode::InvalidVault,
    )]
    pub pool_token_a_vault: Account<'info, TokenAccount>, // Pool's Token A vault (mutable for transfer)

    #[account(
        mut,
        constraint = pool_token_b_vault.key() == pool_state.token_b_vault @ AamErrorCode::InvalidVault,
    )]
    pub pool_token_b_vault: Account<'info, TokenAccount>, // Pool's Token B vault (mutable for transfer)

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

pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;
    let lp_token_mint = &mut ctx.accounts.lp_token_mint;

    if amount_a == 0 || amount_b == 0 {
        return err!(AamErrorCode::ZeroAmount);
    }

    let lp_tokens_to_mint;
    let pool_a_reserves = ctx.accounts.pool_token_a_vault.amount;
    let pool_b_reserves = ctx.accounts.pool_token_b_vault.amount;
    let total_lp_supply = lp_token_mint.supply;

    if total_lp_supply == 0 {
        lp_tokens_to_mint = (amount_a
            .checked_mul(amount_b)
            .ok_or(AamErrorCode::MathOverflow)?)
        .isqrt()
        .checked_sub(MINIMUM_LIQUIDITY)
        .ok_or(AamErrorCode::MathOverflow)?;

        if lp_tokens_to_mint == 0 {
            return err!(AamErrorCode::InsufficientInitialLiquidity);
        }

        msg!("Initial Liquidity: Minting {} LP tokens", lp_tokens_to_mint);
    } else {
        // Subsequent Liquidity Provision
        // Calculate how many LP tokens can be minted for each token at the current pool ratio.
        // Take the minimum to ensure the ratio is maintained.
        let lp_tokens_from_a = (amount_a as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(AamErrorCode::MathOverflow)?
            .checked_div(pool_a_reserves as u128)
            .ok_or(AamErrorCode::ZeroDivision)? as u64;

        let lp_tokens_from_b = (amount_b as u128)
            .checked_mul(total_lp_supply as u128)
            .ok_or(AamErrorCode::MathOverflow)?
            .checked_div(pool_b_reserves as u128)
            .ok_or(AamErrorCode::ZeroDivision)? as u64;

        lp_tokens_to_mint = lp_tokens_from_a.min(lp_tokens_from_b);

        if lp_tokens_to_mint == 0 {
            return err!(AamErrorCode::InsufficientLiquidity);
        }

        // Adjust actual amounts received based on the calculated LP tokens
        // This ensures the correct ratio is maintained.
        let actual_amount_a = (lp_tokens_to_mint as u128)
            .checked_mul(pool_a_reserves as u128)
            .ok_or(AamErrorCode::MathOverflow)?
            .checked_div(total_lp_supply as u128)
            .ok_or(AamErrorCode::ZeroDivision)? as u64;

        let actual_amount_b = (lp_tokens_to_mint as u128)
            .checked_mul(pool_b_reserves as u128)
            .ok_or(AamErrorCode::MathOverflow)?
            .checked_div(total_lp_supply as u128)
            .ok_or(AamErrorCode::ZeroDivision)? as u64;

        // Ensure the user sent at least the required amounts for the calculated LP tokens
        if amount_a < actual_amount_a || amount_b < actual_amount_b {
            return err!(AamErrorCode::LiquidityRatioMismatch);
        }

        // Potentially refund excess if user sent more than strictly needed for the ratio
        // For simplicity, we'll just transfer the full amount they sent here.
        // A more complex AMM might refund excess amounts.

        msg!(
            "Subsequent Liquidity: Minting {} LP tokens",
            lp_tokens_to_mint
        );
        msg!("Actual Token A transferred: {}", actual_amount_a);
        msg!("Actual Token B transferred: {}", actual_amount_b);
    }

    // thansfer token A
    let cpi_accounts_a = Transfer {
        from: ctx.accounts.user_token_a_account.to_account_info(),
        to: ctx.accounts.pool_token_a_vault.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_a);

    token::transfer(cpi_context, amount_a)?;

    // transfer token B
    let cpi_accounts_b = Transfer {
        from: ctx.accounts.user_token_b_account.to_account_info(),
        to: ctx.accounts.pool_token_b_vault.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_b);

    token::transfer(cpi_context, amount_b)?;

    // mint LP tokens
    let key = pool_state.key();
    let authority_seeds = [
        b"pool_authority",
        key.as_ref(),
        &[pool_state.authority_bump], // Use the stored bump directly!
    ];
    let authority_seeds = [&authority_seeds[..]];

    let cpi_accounts_mint = MintTo {
        mint: lp_token_mint.to_account_info(),
        to: ctx.accounts.user_lp_token_account.to_account_info(),
        authority: ctx.accounts.pool_authority_pda.to_account_info(),
    };

    let cpi_context_mint = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts_mint,
        &authority_seeds,
    );

    token::mint_to(cpi_context_mint, lp_tokens_to_mint)?;

    // update pool state
    pool_state.lp_supply = lp_token_mint.supply;

    // Note: For actual reserve updates, it's often more robust to read the vault amounts directly
    // after CPIs, but for basic tracking, this is fine.
    // For production AMMs, it would likely update the pool_state.reserve_a and reserve_b
    // fields based on the actual amounts received, which might be less than `amount_a`/`amount_b`
    // if the client sent excess during subsequent liquidity.
    // For now, we assume `amount_a` and `amount_b` are the *actual* amounts the pool receives.
    pool_state.token_a_reserves = ctx.accounts.pool_token_a_vault.amount;
    pool_state.token_b_reserves = ctx.accounts.pool_token_b_vault.amount;


    msg!("Liquidity added successfully!");

    Ok(())
}
