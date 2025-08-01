use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PoolState {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_token_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub authority: Pubkey,
    pub authority_bump: u8,
    pub trading_fees: u16,
    pub is_initialized: bool,
    pub lp_supply: u64,
    pub token_a_reserves: u64,
    pub token_b_reserves: u64,

    // For future extensibility (important for upgradability)
    pub filler: [u8; 128], 
}