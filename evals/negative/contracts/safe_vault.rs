// Negative-eval fixture (Solana / Anchor): patterns that LOOK like missing-signer,
// account-substitution, unchecked-account, and unchecked-arithmetic bugs but are SAFE.
// A well-calibrated audit should NOT report these. Minimized for the eval.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Safe1111111111111111111111111111111111111111");

#[program]
pub mod safe_vault {
    use super::*;

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // Arithmetic is checked: underflow returns an error rather than wrapping.
        vault.balance = vault
            .balance
            .checked_sub(amount)
            .ok_or(VaultError::InsufficientFunds)?;

        // The destination is an UncheckedAccount, but it is MANUALLY validated below:
        // it must be a token account, owned by the SPL Token program, whose mint matches the
        // vault mint and whose owner is the (signing) authority. With those checks, the
        // "unchecked" account is fully constrained — not an account-validation bug.
        let dest = &ctx.accounts.dest;
        let data = dest.try_borrow_data()?;
        let parsed = TokenAccount::try_deserialize(&mut &data[..])?;
        require_keys_eq!(*dest.owner, token::ID, VaultError::BadOwner);
        require_keys_eq!(parsed.mint, ctx.accounts.vault.mint, VaultError::BadMint);
        require_keys_eq!(parsed.owner, ctx.accounts.authority.key(), VaultError::BadDest);
        drop(data);

        let seeds: &[&[u8]] = &[b"vault", &[ctx.bumps.vault_authority]];
        let signer = &[seeds];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token.to_account_info(),
                    to: dest.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // authority must sign, and has_one binds it to the vault's recorded authority:
    // no account substitution, signer is enforced.
    #[account(mut, has_one = authority)]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
    /// CHECK: validated manually in the handler (token program owner, mint, and token-owner).
    #[account(mut)]
    pub dest: UncheckedAccount<'info>,
    #[account(mut)]
    pub vault_token: Account<'info, TokenAccount>,
    /// CHECK: PDA used only as CPI signing authority; derived from fixed seeds.
    #[account(seeds = [b"vault"], bump)]
    pub vault_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub balance: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("insufficient funds")]
    InsufficientFunds,
    #[msg("bad token owner")]
    BadOwner,
    #[msg("bad mint")]
    BadMint,
    #[msg("bad destination")]
    BadDest,
}
