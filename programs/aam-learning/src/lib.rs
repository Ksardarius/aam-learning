use anchor_lang::prelude::*;

mod state;
mod instructions;
mod errors;

use instructions::*;

declare_id!("ezysgxbCQG6D3bjcCqVNEq7K1vGov7uWYk4h81zmkMx");

#[program]
pub mod aam_learning {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, trading_fees: u16) -> Result<()> {
        instructions::initialize_pool(ctx, trading_fees)
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        instructions::add_liquidity(ctx, amount_a, amount_b)
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, minimum_output_amount: u64) -> Result<()> {
        instructions::swap(ctx, amount_in, minimum_output_amount)
    }
}

