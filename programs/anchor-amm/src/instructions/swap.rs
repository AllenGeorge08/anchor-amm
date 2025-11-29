

use crate::{error::AmmError, state::Config};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::{ConstantProduct, LiquidityPair, SwapResult};

#[derive(Accounts)]
#[instruction(seed: u64, is_x: bool,amount: u64,min: u64)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub mint_x: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()], //e unique for each config
        bump = config.lp_bump
    )]
    pub mint_lp: Box<Account<'info, Mint>>,
    #[account(
        mut,
        has_one = mint_x, //e just a constraint check...
        has_one = mint_y,
        seeds = [b"config",seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp, //e user
        associated_token::authority = user,
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap_tokens_in(&mut self, is_x: bool, amount: u64, min: u64) -> Result<()> {
        //We calculate the swap, if optimal , we perform it..
        require!(self.config.locked == false, AmmError::PoolLocked);

        let pair_to_swap = match is_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };

        let result = self.calculate_swap(pair_to_swap, amount, min)?;
        let (amount_to_deposit, amount_to_withdraw, fee) =
            (result.deposit, result.withdraw, result.fee);

        require!(amount_to_deposit + fee >= amount, AmmError::InvalidAmount);

        //Transferring from user to vault
        let (from, to) = match is_x {
            true => (
                self.user_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.user_y.to_account_info(),
                self.vault_y.to_account_info(),
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(ctx, amount)?;

        self.swap_back_out(is_x, amount_to_withdraw)?;
        Ok(())
    }

    pub fn calculate_swap(
        &mut self,
        pair_to_swap: LiquidityPair,
        amount: u64,
        min: u64,
    ) -> Result<SwapResult> {
        require_eq!(self.config.locked, false, AmmError::PoolLocked);

        let mut curve = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            self.config.fee,
            Some(6),
        ).map_err(|e| 
        {   msg!("Error while initializing curve: {} ",e);
            AmmError::DefaultError})?;
        

        
        // let p = match is_x {
        //     true => LiquidityPair::X,
        //     false => LiquidityPair::Y
        // };

        let swap_result = curve.swap(pair_to_swap, amount, min).map_err(|_| AmmError::CurveError)?;

        Ok(swap_result)
    }

    pub fn swap_back_out(&mut self, is_x: bool, amount: u64) -> Result<()> {
        //Transferring from user to vault
        let (from, to) = match is_x {
            true => (
                self.vault_y.to_account_info(),
                self.user_y.to_account_info(),
            ),
            false => (
                self.vault_x.to_account_info(),
                self.user_x.to_account_info(),
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };

        let seeds: &[&[u8]; 3] = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump],
        ];

        let signer_seeds: &[&[&[u8]]; 1] = &[&seeds[..]];

        let ctx: CpiContext<'_, '_, '_, '_, Transfer<'_>> =
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(ctx, amount)?;

        Ok(())
    }
}
