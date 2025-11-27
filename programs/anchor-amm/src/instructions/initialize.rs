use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction(seed: u64,fee: u16,authority: Pubkey)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>, //e Make sure the account names are justified to the roles
    #[account(mut)]
    pub mint_x: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = initializer,
        seeds = [b"lp", config.key().as_ref()], //e unique for each config
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = initializer,
        seeds = [b"config",seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Config::INIT_SPACE,
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init(
        &mut self,
        seed: u64,
        fee: u16,
        authority: Option<Pubkey>,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        //e fees are calculated in basis points and it signficantly reduces the need for decimals.
        self.config.set_inner(Config {
            seed: seed,
            authority: authority,
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            fee: fee,
            locked: false,
            config_bump: bumps.config,
            lp_bump: bumps.mint_lp,
        });
        Ok(())
    }
}
