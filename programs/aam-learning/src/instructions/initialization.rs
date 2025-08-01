use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{errors::AamErrorCode, state::PoolState};


#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [b"pool_state", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        space = 8 + PoolState::INIT_SPACE,
    )]
    pub pool_state: Account<'info, PoolState>,

    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = pool_authority_pda
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        seeds = [
            b"pool_authority",
            pool_state.key().as_ref() // Derive from pool_state's key for uniqueness
        ],
        bump,
    )]
    /// CHECK: This account is a PDA and its ownership is verified by the program logic.
    pub pool_authority_pda: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        token::mint = token_a_mint,
        token::authority = pool_authority_pda, // PDA is the owner
    )]
    pub pool_token_a_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        token::mint = token_b_mint,
        token::authority = pool_authority_pda, // PDA is the owner
    )]
    pub pool_token_b_vault: Account<'info, TokenAccount>,


    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>, // The SPL Token Program
    pub system_program: Program<'info, System>,
}

pub fn initialize_pool(ctx: Context<InitializePool>, trading_fees: u16) -> Result<()> {
    require!(!ctx.accounts.pool_state.is_initialized, AamErrorCode::AlreadyInitialized); 

    let pool_state = &mut ctx.accounts.pool_state;

    pool_state.token_a_mint = ctx.accounts.token_a_mint.key();
    pool_state.token_b_mint = ctx.accounts.token_b_mint.key();
    pool_state.lp_token_mint = ctx.accounts.lp_token_mint.key();
    pool_state.token_a_vault = ctx.accounts.pool_token_a_vault.key();
    pool_state.token_b_vault = ctx.accounts.pool_token_b_vault.key();
    pool_state.authority = ctx.accounts.pool_authority_pda.key();
    pool_state.authority_bump = ctx.bumps.pool_authority_pda;
    pool_state.trading_fees = trading_fees;

    pool_state.is_initialized = true;

    // --- 4. Log confirmation ---
    msg!("Pool initialized successfully!");
    msg!("Token A Mint: {}", pool_state.token_a_mint);
    msg!("Token B Mint: {}", pool_state.token_b_mint);
    msg!("LP Token Mint: {}", pool_state.lp_token_mint);
    msg!("Pool Authority PDA: {}", pool_state.authority);

    Ok(())

}