use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Stake1111111111111111111111111111111111111");

#[program]
pub mod staking {
    use super::*;

    // Subtle G2: init_if_needed used without a guard, so an attacker can re-call
    // initialize on an existing pool and reset reward_rate / authority.
    pub fn initialize(ctx: Context<Initialize>, reward_rate: u64) -> Result<()> {
        let p = &mut ctx.accounts.pool;
        p.authority = ctx.accounts.authority.key();
        p.reward_rate = reward_rate;
        p.total = 0;
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.total = pool.total.checked_add(amount).unwrap();
        // Subtle G1: `user_stake` is deserialized but its `owner` field is never checked
        // against ctx.accounts.user. An attacker can pass someone else's UserStake account
        // (same type, valid discriminator) and credit their own deposit to it — or pass a
        // look-alike account. The relationship user_stake.owner == user is never enforced.
        let us = &mut ctx.accounts.user_stake;
        us.amount = us.amount.checked_add(amount).unwrap();
        let cpi = CpiContext::new(ctx.accounts.token_program.to_account_info(), Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        });
        token::transfer(cpi, amount)?;
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let us = &mut ctx.accounts.user_stake;
        us.amount = us.amount.checked_sub(amount).unwrap();
        // Subtle G3: pool_vault authority for the CPI is `ctx.accounts.authority`, an
        // UncheckedAccount, rather than a program PDA signer. Combined with G1 (no owner
        // binding on user_stake), an attacker can unstake against any UserStake.
        let cpi = CpiContext::new(ctx.accounts.token_program.to_account_info(), Transfer {
            from: ctx.accounts.pool_vault.to_account_info(),
            to: ctx.accounts.user_token.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        });
        token::transfer(cpi, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init_if_needed, payer = authority, space = 8 + 72)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_stake: Account<'info, UserStake>,   // no has_one = user
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_stake: Account<'info, UserStake>,
    /// CHECK: used as CPI signer authority, not validated
    pub authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Pool { pub authority: Pubkey, pub reward_rate: u64, pub total: u64 }
#[account]
pub struct UserStake { pub owner: Pubkey, pub amount: u64 }
