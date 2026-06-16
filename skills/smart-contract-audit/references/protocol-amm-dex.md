# Protocol Playbook — AMMs / DEXes

Use this **in addition to** `methodology.md` and `solidity-vectors.md` when the target is an
automated market maker or swap venue: constant-product (Uniswap v2 forks), stableswap
(Curve/Balancer-style), or concentrated-liquidity (Uniswap v3/v4). Protocol-class checklist on top
of the generic vectors, not a replacement. Core invariants to attack:

> **Invariant preservation:** the pool's invariant (`x·y ≥ k` after fees for constant-product; the
> stableswap/weighted invariant otherwise) must never decrease in the pool's disfavor across any
> swap or liquidity op. **LP fairness:** minting/burning LP shares is strictly pro-rata; no one
> mints LP value from nothing or extracts more than their share.

Most AMM losses are rounding that lets the invariant slip, a manipulable spot price used as an
oracle, or first-liquidity share manipulation.

## 1. Invariant & swap rounding — must favor the pool

- **Look for:** `amountOut` not rounded **down** and `amountIn` not rounded **up**; the post-swap
  `k` check missing or computed before fees; fee applied to the wrong side or omitted; precision
  loss in the swap math that, repeated, drains the pool a wei at a time.
- **Confirm:** construct a swap (or batched/looped swaps) that leaves `x·y` below the pre-swap
  value net of fees, extracting value. For stableswap, precision/convergence error in the
  Newton-iteration invariant is the classic compounding bug (cf. Balancer V2 rounding, ~$128M,
  Nov 2025 — see `solidity-vectors.md` SC07).

## 2. First-liquidity / LP-share manipulation

- **Look for:** missing `MINIMUM_LIQUIDITY` lock (Uniswap v2 permanently burns the first 1000
  shares) or equivalent dead-shares; first LP able to mint 1 share then **donate** tokens to skew
  share price so later LPs round to near-zero (the AMM cousin of the ERC-4626 first-depositor bug,
  see `protocol-erc4626-vault.md` §1).
- **Confirm:** first-LP + donation sequence that lets the attacker capture a later LP's deposit.

## 3. Spot price is not an oracle

- **Look for:** any external (or internal) consumer reading `getReserves`/`balanceOf(pool)`/`slot0`
  as a price; that value is flash-loan manipulable within a block (cross-ref `solidity-vectors.md`
  SC03/SC04). For v3, `slot0.sqrtPriceX96` is the *current* (manipulable) tick — only the TWAP
  oracle (`observe`) is sound, and only with an adequate window and cardinality.
- **Confirm:** flash-manipulate reserves/tick, show a dependent contract misprices, then revert the
  manipulation in the same tx.

## 4. Reserve accounting & weird tokens

- **Look for:** `swap`/`mint`/`burn` trusting the `amount` argument instead of the **measured
  balance delta** — **fee-on-transfer** and **rebasing** tokens desync reserves from real balances;
  `sync()`/`skim()` abuse to manipulate reserves; reserves stored as `uint112` overflow assumptions.
- **Confirm:** with a fee-on-transfer token, show reserves drift from balances enabling extraction,
  or that `skim` lets an attacker harvest a donation meant to be shared.

## 5. Reentrancy, flash-swaps & callbacks

- **Look for:** missing reentrancy `lock` on `swap`/`mint`/`burn`; **flash-swap callbacks**
  (`uniswapV2Call`/v3 `swapCallback`/flash-loan callbacks) that let the recipient re-enter before
  reserves/`k` are reconciled; callback tokens (ERC777) on transfer paths; v3/v4 **hooks** running
  attacker code mid-swap (validate hook permissions and that core invariants hold after the hook).
- **Confirm:** re-enter through the callback and complete a swap/mint that violates `k` or
  double-counts.

## 6. Slippage, deadline & MEV

- **Look for:** swap/add/remove-liquidity entry points (or routers over them) without
  `amountOutMin`/`amountInMax` and a `deadline`; sandwichable adds/removes; `permit()`/permit2
  front-running; lack of price-impact bounds on large ops.
- **Confirm:** a sandwich that extracts value from a victim's unprotected swap or liquidity op.

## 7. Concentrated-liquidity & stableswap specifics

- **v3/v4:** tick math rounding at tick crossings, fee-growth accounting per position, position
  accounting on partial burns, and rounding direction on `amount0/amount1` owed; ensure liquidity
  net at ticks can't be corrupted. v4 **hooks** are arbitrary code — treat the hook as untrusted
  and re-check the invariant after every hook callback.
- **Stableswap:** amplification-coefficient (`A`) ramp manipulation, imbalanced-deposit fee
  rounding, and convergence/precision of the invariant solver under extreme imbalance.

## Quick triage hints
- `amountOut`/`amountIn` rounding direction; post-swap `k` check present & after fees → invariant drain
- missing `MINIMUM_LIQUIDITY`/dead shares on first mint → LP-share manipulation
- `getReserves`/`slot0.sqrtPriceX96` read as price → spot-oracle manipulation (use TWAP `observe`)
- `amount` arg vs balance delta; `sync`/`skim` → fee-on-transfer/rebasing reserve desync
- `uniswapV2Call`/`swapCallback`/flash callbacks, v4 hooks, missing `lock` → reentrancy
- router calls without `amountOutMin`/`deadline` → slippage / sandwich
