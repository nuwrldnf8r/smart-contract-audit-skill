# Protocol Playbook — Lending / Borrowing Markets

Use this **in addition to** `methodology.md` and `solidity-vectors.md` when the target is a
money market (Aave/Compound-style pools, isolated-pair lending, CDP/stablecoin mints, leverage
vaults). Protocol-class checklist on top of the generic vectors, not a replacement. The core
invariants to attack:

> **Solvency:** after every operation, each account's debt ≤ the liquidation value of its
> collateral, and the protocol can honor all withdrawals it has promised.
> **Conservation:** interest, fees, and liquidation bonuses are accounted to exactly one party;
> no path mints debt or credit from nothing.

Most lending losses are a way to borrow against value that isn't really there (oracle), or to
leave the protocol holding bad debt (liquidation/rounding).

## 1. Oracle & collateral valuation (the #1 lending exploit class)

- **Look for:** spot price from an AMM pool / `getReserves` (flash-loan manipulable); a single
  feed with no fallback; missing staleness/round checks on Chainlink (`updatedAt`,
  `answeredInRound`); **missing L2 sequencer-uptime check** on an L2 deployment (cross-ref
  `solidity-vectors.md` SC03); decimal mismatches between feed and token; using one asset's feed
  for a different (e.g. bridged/look-alike) token.
- **Confirm:** manipulate or stale the price within one tx and **borrow under-collateralized** or
  trigger an **unfair liquidation**. Re-run under the flash-loan assumption (unlimited single-tx
  capital, SC04).
- **LTV vs liquidation threshold:** confirm borrow uses LTV and liquidation uses the (higher)
  threshold, both applied with correct decimals; an off-by-decimals here is instant insolvency.

## 2. Health-factor math & interest accrual ordering

- **Accrue first:** every state-changing entry point (deposit/borrow/repay/withdraw/liquidate)
  must **accrue interest before** reading balances/health, or it acts on stale debt. Look for a
  missing `accrue()` on any path.
- **Post-op health check:** borrow/withdraw must re-check health **after** the state change, not
  before. A check on pre-state lets a user exit insolvent.
- **Index/accumulator drift:** rounding in the interest index that favors borrowers (debt rounds
  down) or lenders (credit rounds up) compounds. Utilization edge cases (100% utilization, empty
  market) and per-second vs per-block accrual mismatches.

## 3. Liquidation — where bad debt is born

- **Look for:** liquidation **incentive/bonus** that exceeds collateral on small/underwater
  positions (liquidator can't be made whole → position left as bad debt); no **close factor** (or
  one that allows liquidating a healthy position); **self-liquidation** to harvest the bonus;
  liquidation that leaves **dust debt** with no collateral (uncollectible); rounding in
  **collateral seizure** that favors the liquidator over the protocol.
- **Ordering/MEV:** liquidations are front-runnable and sandwichable; a borrower can be
  grief-liquidated by pushing price for one block; check for **liquidate-then-repay** in the same
  tx, and whether a borrower can block liquidation (e.g. by making `repay`/transfer revert).
- **Bad-debt handling:** when collateral < debt, is there an explicit socialization / reserve
  draw, or does the last withdrawer eat it (bank-run dynamics)? Silent bad debt is a Critical
  solvency finding even without an active exploit.

## 4. Share/receipt-token inflation (cToken/aToken style)

A lending market's supply-side receipt token is an ERC-4626-shaped share — it inherits the
**first-depositor / donation inflation** bug. See `protocol-erc4626-vault.md` §1–§2; verify the
empty-market exchange rate can't be manipulated by a donation to steal later suppliers, and that
exchange-rate rounding favors the protocol.

## 5. Caps, modes & risk parameters

- **Borrow/supply caps**, **isolation mode**, **e-mode/correlated-asset** modes: confirm each
  enforces its constraint on the actual entry points (a cap that's checked in one path but not
  another is a bypass). Pausing/freezing an asset must not **block `repay`** — if users can't
  repay while they can be liquidated, that's a forced-loss griefing vector.
- **Reserve factor / protocol fee** accounting: skimmed to the right place, not double-counted
  into lender yield.

## 6. Reentrancy, weird tokens, and account-level logic

- **Reentrancy:** deposit/borrow/withdraw with callback collateral (ERC777/fee-on-transfer); and
  **read-only reentrancy** where a price/health view consumed by another market is read mid-update.
- **Weird ERC20s:** fee-on-transfer or rebasing collateral breaks balance accounting (measure
  deltas); non-standard decimals; missing-return tokens (use `SafeERC20`).
- **Collateral toggling:** a user disabling an asset as collateral, or a cross-asset interaction,
  that leaves a shortfall the health check misses.

## Quick triage hints
- AMM `getReserves`/`slot0` in pricing, missing `updatedAt`/`sequencerUptimeFeed` → oracle exploit
- borrow/withdraw health check on pre-state, or missing `accrue` → solvency bypass
- liquidation bonus vs collateral on small positions, dust debt, self-liquidation → bad debt
- exchange-rate / index `mulDiv` rounding direction → drains lenders or borrowers
- pause that blocks `repay` → forced-liquidation griefing
- empty-market exchange rate + donation → receipt-token inflation (see protocol-erc4626-vault.md)
