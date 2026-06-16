# Protocol Playbook — ERC-4626 / Tokenized Vaults

Use this **in addition to** `methodology.md` and `solidity-vectors.md` when the target is a
tokenized vault (ERC-4626 or an ERC-4626-shaped share/asset accounting system: yield vaults,
lending receipt tokens, LST/LRT wrappers, staking shares). It is a protocol-class checklist on top
of the generic vectors, not a replacement. Tie every candidate back to the core vault invariant:

> **A share is a claim on assets. No action may let one holder's shares appreciate at another's
> expense, mint shares not backed by assets, or extract more assets than the shares redeemed.**

Most vault losses are a way to break that with rounding, donations, or stale prices.

## 1. First-depositor / donation (share-inflation) attack — the signature ERC-4626 bug

The classic: an empty (or near-empty) vault lets the first depositor mint 1 wei-share, then
**donate** assets directly to the vault (a raw `transfer`, not `deposit`), inflating
`totalAssets` while `totalSupply` stays at 1. A later depositor's assets now round to **0 shares**
and are captured by the attacker on redeem.

- **Look for:** `deposit`/`mint` with no virtual-shares offset and no minimum initial deposit /
  dead-shares lock; `convertToShares = assets * totalSupply / totalAssets` with `totalAssets`
  derived from `asset.balanceOf(address(this))` (donatable).
- **Confirm the mitigation is real:** OpenZeppelin v4.9+ uses **virtual shares + a decimals
  offset** (`_decimalsOffset()`); verify the offset is actually nonzero/adequate for the asset's
  value-per-token, or that the vault burns dead shares / enforces a min first deposit / is seeded
  at deployment. A vault that "uses OZ ERC4626" but leaves the offset at 0 is still vulnerable for
  high-value assets. Quantify: with offset `n`, the attacker's donation must exceed
  `~10^n ×` a share to round a victim to zero — show whether that's economical.

## 2. Rounding direction — must always favor the vault, never the user

EIP-4626 mandates the direction; getting it wrong drains the vault a wei at a time, compounding
over many calls.

- **Required directions:** `deposit`/`mint` round shares **down** to the user; `withdraw`/`redeem`
  round assets **down** / shares **up** so the user never gets more than their pro-rata. The four
  `previewX` must match what the action actually does, including fees.
- **Look for:** `mulDiv` without an explicit `Rounding` argument, or rounding that favors the
  caller anywhere in `convertToShares`/`convertToAssets`/`previewX`; a `withdraw` that floors
  shares-burned (lets repeated small withdrawals extract dust from other holders).
- **Confirm:** pick the adversarial direction and show a repeated-call sequence that nets the
  attacker more assets out than in, or leaves `totalAssets/totalSupply` drifting in their favor.

## 3. `totalAssets()` integrity — the price source

Share price is `totalAssets / totalSupply`. Whatever feeds `totalAssets` is the vault's oracle.

- **Look for:** `totalAssets` = `balanceOf(this)` (donatable, see §1); `totalAssets` reading a
  manipulable external position (an AMM LP value, a spot price, another vault's `convertToAssets`);
  unrealized-yield counted before it's claimable; debt/loss not subtracted.
- **Read-only reentrancy:** if `convertToAssets`/`pricePerShare`/`previewRedeem` is consumed by an
  **external** protocol as a price, an attacker can call it mid-redeem-callback while balances are
  half-updated and feed a wrong price to that protocol (SCWE-137, cross-ref `solidity-vectors.md`
  SC08). Flag any external integrator that trusts a vault share price without a manipulation guard.

## 4. Reentrancy on the value-moving paths

- **Look for:** `_deposit`/`_withdraw` performing the asset `transfer`/`transferFrom` **before**
  updating shares/accounting (violates checks-effects-interactions); no `nonReentrant`; the
  underlying being a callback token (ERC777 `tokensReceived`, ERC1155, fee-on-transfer hook).
- **Confirm:** with a callback asset, re-enter `deposit`/`withdraw`/`redeem` during the transfer
  and double-count or double-withdraw.

## 5. Weird-asset assumptions

- **Fee-on-transfer:** crediting `assets` (the argument) instead of the **measured balance delta**
  over-mints shares. Vault must measure `balanceAfter - balanceBefore`.
- **Rebasing assets** silently change `balanceOf(this)`, breaking the share/asset ratio — usually
  unsupported; confirm the vault rejects or special-cases them.
- **Non-18 decimals** + the decimals offset interacting; **reverting/blocklist** assets bricking
  `withdraw` for some holders.

## 6. Slippage, deadlines & integrator footguns

ERC-4626's `deposit/mint/withdraw/redeem` have **no slippage parameters** — share price can move
between quote and execution (sandwich, or a large redeem ahead of you).

- **Look for:** routers/zaps/aggregators calling vault entry points with no `minSharesOut` /
  `maxAssetsIn` wrapper; protocols using `previewRedeem` as a settlement price.
- **Document for integrators:** `convertTo*`/`previewX` are **not** a safe oracle and **may
  revert**; `maxWithdraw`/`maxRedeem` can be below a user's balance when the vault is illiquid.

## 7. Limits, pausing & lifecycle

- **`maxDeposit`/`maxMint`/`maxWithdraw`/`maxRedeem`** consistent with caps and pause state; a
  `deposit` that succeeds past `maxDeposit`, or a pause that blocks `withdraw` but not `deposit`.
- **Zero-share / dust deposits** that grief accounting; **withdrawal when the underlying strategy
  is illiquid** (assets locked downstream) — does it revert cleanly or socialize a loss?
- **Loss reporting / slashing:** if the strategy can lose value, confirm losses are reflected in
  `totalAssets` atomically so the next withdrawer doesn't get a stale, too-high price.

## Quick triage hints
- `_decimalsOffset`, `virtual`, `MINIMUM`, dead shares → first-depositor mitigation present?
- `balanceOf(address(this))` in `totalAssets` → donation/manipulation surface
- `mulDiv(` without `Rounding` → check rounding direction
- `previewDeposit/Mint/Withdraw/Redeem` → must equal the real action incl. fees
- external callers of `convertToAssets`/`pricePerShare` → read-only-reentrancy / oracle misuse
