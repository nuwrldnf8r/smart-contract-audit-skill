# Solana (Anchor & Native) Vulnerability Catalogue

Solana's account model is fundamentally different from EVM, and most Solana exploits come
from that difference: a program receives a set of accounts as input and **must validate them
itself**. The runtime won't stop you from passing the wrong account. This catalogue is built
around the well-known **Sealevel attack classes** (coral-xyz/sealevel-attacks) plus Anchor
specifics. Use with `methodology.md`.

## Table of contents
1. The account model (read first)
2. Signer authorization
3. Account ownership & type confusion
4. Account data matching / substitution
5. PDA & seed safety
6. CPI safety
7. Arithmetic
8. Account lifecycle (init, close, reinit, rent)
9. Anchor-specific constraints
10. Logic / economic
11. Triage hints

---

## 1. The account model (read first)
- A program instruction takes a list of accounts. The program must verify each one is the
  account it expects — correct **owner**, correct **type**, correct **signer** status,
  correct **address/PDA**, and correct **relationships** between accounts.
- Anchor automates many of these checks via `#[derive(Accounts)]` constraints; **native
  programs must do them all by hand.** Most native-program bugs are a missing check Anchor
  would have done for free.
- "It compiled and the happy path works" tells you nothing about security here — the attack
  is almost always *passing a different account than intended.*

## 2. Signer authorization (the #1 cause of Solana exploits)
- **Missing signer check.** A field that represents an authority must be required to sign
  (`is_signer == true`, or Anchor `Signer<'info>` / `has_one` / `#[account(signer)]`).
  Wormhole lost ~$320M because a pubkey was checked without verifying it had signed.
- **Authority confusion.** Verify the signer is *the specific* authority for the target
  account, not just *any* signer (combine signer check with an ownership/`has_one` link).

## 3. Account ownership & type confusion
- **Owner check.** Confirm each account is owned by the expected program
  (`account.owner == expected_program_id`). A token account must be owned by the SPL Token
  program; a state account by your program. Native code must check explicitly.
- **Type cosplay / discriminator.** Two account types with the same byte layout can be
  swapped. Anchor's 8-byte discriminator prevents this; native code must include and check a
  type tag. Verify the deserialized account is actually the intended type.

## 4. Account data matching / substitution
- **Unvalidated relationships.** If account A should reference account B (e.g.
  `vault.authority == authority.key()`), check it. Failing to match data lets an attacker
  substitute an account whose contents they control (Sealevel "account data matching").
- **Sysvar / known-address spoofing.** Validate sysvars and program IDs against their canonical
  addresses rather than trusting whatever was passed.

## 5. PDA & seed safety
- **Bump canonicalization.** Use the canonical bump (Anchor `bump` / `find_program_address`);
  rejecting non-canonical bumps prevents an attacker using an alternate valid PDA.
- **Seed collisions.** Seeds must uniquely identify the intended account; overlapping seed
  schemes let one PDA stand in for another.
- **PDA authority misuse.** Signing CPIs with a PDA (`invoke_signed`) must use the correct
  seeds; don't expose a PDA-signed call that an attacker can redirect.

## 6. CPI (cross-program invocation) safety
- **Arbitrary CPI.** Verify the target program ID before invoking; don't call whatever program
  the caller supplied. Forwarding a user-controlled program is "arbitrary CPI."
- **Signer forwarding.** Be careful that a user's signature isn't forwarded through a CPI to a
  context where it grants more than intended.

## 7. Arithmetic
- **Overflow.** Release builds don't check overflow by default. Use `checked_*` /
  `saturating_*` and `overflow-checks = true`. Casts (`as u64`) can truncate silently.
- **Rounding/precision** in share, reward, and AMM math — same economic concerns as EVM.

## 8. Account lifecycle
- **Reinitialization.** A previously-initialized account being initialized again can reset
  authority/state. Guard init (Anchor `init` rejects existing accounts; native must check).
- **Account closing / revival.** On close, zero the data, reassign owner, and drain lamports
  correctly. A "closed" account can be revived within the same tx if not handled (closing-
  account attack); use Anchor `close =` or follow the safe close pattern.
- **Rent / lamport checks** where balances gate behavior.

## 9. Anchor-specific constraints (verify they're present and correct)
- `Signer`, `has_one`, `address = `, `owner = `, `seeds`/`bump`, `constraint = `, `init`,
  `init_if_needed` (dangerous — re-init risk), `close = `, `mut`, `realloc`.
- A misused or missing constraint is the typical Anchor finding. `init_if_needed` in
  particular needs a manual re-initialization guard.
- Check `#[account(mut)]` is present everywhere state is written, and absent where it isn't.

## 10. Logic / economic
- Business-logic invariant breaks, oracle manipulation (Pyth/Switchboard staleness &
  confidence interval validation), flash-style amplification via on-chain liquidity, and MEV
  all apply — drive from Phase 1 invariants. Validate oracle `publish_time`/staleness and
  confidence, not just the price.

## 11. Triage hints
- For every account in a `#[derive(Accounts)]` or native instruction, ask: is its **owner**,
  **signer**, **type**, **address/PDA**, and **relationship to other accounts** checked?
- Grep for `AccountInfo` / `UncheckedAccount` / `/// CHECK:` → manual validation required;
  confirm it's actually done.
- Grep `invoke`, `invoke_signed` → CPI target + seed review.
- Grep `init_if_needed` → reinitialization risk.
- Grep `as u64`, `as u32`, raw `+`/`-`/`*` on balances → arithmetic review; check
  `overflow-checks` in `Cargo.toml`.
- Confirm signer checks on every authority; confirm owner checks on every program-owned
  account in native code.
