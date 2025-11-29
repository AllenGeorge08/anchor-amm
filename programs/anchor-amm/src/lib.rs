use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

pub use instructions::*;


declare_id!("6e4ihRQog474ibebs9TaFLNBXw2TYwZGuFxnNnbw8iZj");

#[program]
pub mod anchor_amm {

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
        authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.init(seed, fee, Some(authority), &ctx.bumps)?;
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        seed: u64,
        amount_to_deposit: u64,
        max_x: u64,
        max_y: u64,
    ) -> Result<()> {
        ctx.accounts
            .deposit(seed, amount_to_deposit, max_x, max_y)?;
        Ok(())
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        seed: u64,
        amount_to_withdraw: u64,
        min_x: u64,
        min_y: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw(amount_to_withdraw, min_x, min_y)?;
        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        seed: u64,
        is_x: bool,
        // pair_to_swap: LiquidityPair,
        amount: u64,
        min: u64,
    ) -> Result<()> {
        ctx.accounts.swap_tokens_in(is_x, amount, min)?;
        Ok(())
    }

    pub fn lock(ctx: Context<Lock>,seed: u64) -> Result<()> {
        ctx.accounts.lock(seed)?;
        Ok(())
    }

    pub fn unlock(ctx: Context<Lock>,seed: u64) -> Result<()> {
        ctx.accounts.unlock(seed)?;
        Ok(())
    }
}
