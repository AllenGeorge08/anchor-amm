use crate::{error::AmmError, state::Config};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Lock<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config",seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Box<Account<'info, Config>>,
    pub system_program: Program<'info, System>,
}

impl<'info> Lock<'info> {
    pub fn lock(&mut self,seed: u64) -> Result<()> {
        require_eq!(
            self.user.key(),
            self.config.authority.unwrap(),
            AmmError::InvalidAuthority
        );
        self.config.locked = true;
        Ok(())
    }

    pub fn unlock(&mut self,seed: u64) -> Result<()> {
        require_eq!(
            self.user.key(),
            self.config.authority.unwrap(),
            AmmError::InvalidAuthority
        );
        self.config.locked = false;
        Ok(())
    }
}
