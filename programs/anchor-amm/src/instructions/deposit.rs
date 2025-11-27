use crate::{error::AmmError, state::Config};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
#[instruction(seed: u64,amount_to_deposit: u64, max_x: u64, max_y: u64)]
pub struct Deposit<'info> {
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

impl<'info> Deposit<'info> {
    pub fn deposit(
        &mut self,
        _seed: u64,
        amount_to_deposit: u64,
        max_x: u64,
        max_y: u64,
    ) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require_gte!(amount_to_deposit, 0, AmmError::InvalidAmount);

        //e To check if the user is using the liquidity pool for the first time, so we're creating the state of the amm...
        let (x, y) = match self.mint_lp.supply == 0
            && self.vault_x.amount == 0
            && self.vault_y.amount == 0
        {
            true => (max_x, max_y),
            false => {
                let amount = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount_to_deposit,
                    6,
                )
                .unwrap();
                (amount.x, amount.y)
            }
        };

        require!(x <= max_x && y <= max_y, AmmError::SlippageExceded);
        self.deposit_tokens(true, x)?;
        self.deposit_tokens(false, y)?;
        self.mint_lp_tokens(amount_to_deposit)?;

        Ok(())
    }

    pub fn deposit_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
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

        let cpi_program: AccountInfo<'_> = self.token_program.to_account_info();

        let cpi_accounts: Transfer<'_> = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx: CpiContext<'_, '_, '_, '_, _> = CpiContext::new(cpi_program, cpi_accounts);
        transfer(ctx, amount)?;

        Ok(())
    }

    pub fn mint_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let seeds: &[&[u8]; 3] = &[
            &b"config"[..],
            &self.config.seed.to_le_bytes(),
            &[self.config.config_bump], //e No LP Bump as the authority is signing here..
        ];

        let signer_seeds: &[&[&[u8]]; 1] = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        mint_to(ctx, amount)?;

        Ok(())
    }
}
