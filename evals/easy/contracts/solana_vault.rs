use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Vau1t1111111111111111111111111111111111111");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        v.authority = ctx.accounts.authority.key();
        v.balance = amount;
        Ok(())
    }

    // S1 + S2: withdraw authority is NOT required to sign, and no check that
    // the passed `authority` matches vault.authority (missing has_one / signer).
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let v = &mut ctx.accounts.vault;
        // S3: arithmetic underflow possible (no checked_sub), release builds wrap
        v.balance = v.balance - amount;
        let cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token.to_account_info(),
                to: ctx.accounts.dest.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        );
        token::transfer(cpi, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 40)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    // S1: authority is UncheckedAccount, not Signer, and no has_one constraint
    /// CHECK: not validated
    pub authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub vault_token: Account<'info, TokenAccount>,
    // S4: dest token account owner not validated; attacker can pass own account
    #[account(mut)]
    pub dest: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub balance: u64,
}
