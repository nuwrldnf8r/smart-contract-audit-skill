# Solidity / EVM Vulnerability Catalogue

Use this alongside `methodology.md`. It is organized around the **OWASP Smart Contract
Top 10 (2026)** with the deeper **OWASP SCWE** weakness IDs folded in, plus integration
and token-specific issues. For each class: what it is, how to spot it, and how to confirm.
Don't just match patterns — tie each candidate finding back to a broken invariant and a
concrete exploit path.

## Table of contents
- SC01 Access Control
- SC02 Business Logic
- SC03 Price Oracle Manipulation
- SC04 Flash-Loan-Facilitated Attacks
- SC05 Input Validation
- SC06 Unchecked External Calls
- SC07 Arithmetic Errors (rounding & precision)
- SC08 Reentrancy
- SC09 Integer Overflow / Underflow
- SC10 Proxy & Upgradeability
- Cross-cutting: signatures, randomness, DoS, MEV
- Token integration & "weird ERC20s"
- Quick grep/triage hints

---

## SC01 — Access Control Vulnerabilities (largest loss category)
Unauthorized callers reaching privileged functions or critical state.
- **Look for:** state-changing/admin functions missing modifiers; `onlyOwner` declared but
  not applied; `tx.origin` used for auth (SCWE-018); role checks that can be bypassed via
  fallback, `delegatecall`, or a public initializer; `selfdestruct`/upgrade/withdraw paths
  without auth (SCWE-038/049/050); single-step ownership transfer (SCWE-139 — prefer
  `Ownable2Step`); single EOA admin / no multisig (SCWE-129/155, centralization risk).
- **Confirm:** trace the call graph to the sensitive sink; verify *no* path reaches it
  without the intended check. Check that modifiers are actually on the externally-reachable
  function, not just an internal helper.
- **Insider resistance (INS lens).** Enforced access control is necessary but not sufficient.
  Separately assume each privileged role is hostile/compromised *and the check passes*, then
  ask what one transaction can extract or brick (confiscate fees, repoint the oracle, mint,
  pause-and-extract, "rescue"-drain). For each such power check whether it is **bounded** —
  timelock/delay, hard cap or rate-limit, price-deviation band, role separation (setter ≠
  upgrader, proposer ≠ executor), limits that apply to admins too, immutability of the worst
  params. An unbounded power over user funds is an INS-class finding even under a reputable
  multisig; the fix is to *bound* it, not to delete the role. See methodology Phase 4.

## SC02 — Business Logic Vulnerabilities
Design-level flaws that break intended economic/functional rules even when low-level checks
pass. Now among the costliest classes and invisible to linters.
- **Look for:** incorrect accounting (deposit/withdraw/reward math), reward double-claim,
  ordering assumptions, missing post-operation health/solvency check (SCWE-125), state read
  after it's been made stale, liquidation logic that can be gamed, fee/discount edge cases,
  first-depositor manipulation.
- **Confirm:** drive it from the Phase 1 invariants. Construct the call sequence that leaves
  the system in a state the designer assumed impossible (e.g. shares minted with no assets).

## SC03 — Price Oracle Manipulation
- **Look for:** spot price read directly from an AMM pair/`getReserves`/`balanceOf` of a pool
  (SCWE-112 low-liquidity spot); single oracle source (SCWE-029); missing staleness/round
  validation on Chainlink (`updatedAt == 0`, `answeredInRound < roundId`) (SCWE-086);
  insufficient TWAP window or single observation (SCWE-113); decimal mismatch in price math
  (SCWE-088); admin-writable oracle without delay (SCWE-130); unvalidated min/max bands
  (SCWE-085).
- **Confirm:** show that manipulating the source within one block/tx (often via flash loan)
  moves the protocol's reference price enough to borrow under-collateralized, trigger unfair
  liquidation, or misprice a swap.

## SC04 — Flash-Loan-Facilitated Attacks
Not a bug in itself — an amplifier. Assume unlimited single-tx capital.
- **Look for:** any safety check that implicitly assumes the attacker lacks capital; price
  reads, governance vote weight, or collateral valuations taken at a single point in a tx;
  flash-loan-fueled governance (SCWE-101).
- **Confirm:** re-run the oracle/logic/arithmetic findings under the "attacker borrowed 100M
  for this tx" assumption and show the magnified drain.

## SC05 — Lack of Input Validation
- **Look for:** unchecked addresses (zero-address on critical params, SCWE-143); amounts not
  bounded; array-length/decoding without length checks (SCWE-122/154); missing slippage
  (SCWE-090) and deadline (SCWE-141) on swaps/liquidity; unvalidated constructor params
  (SCWE-145); cross-chain inputs trusted without proof.
- **Confirm:** find an input value that reaches core logic and corrupts state or steals funds.

## SC06 — Unchecked External Calls
- **Look for:** low-level `.call`/`.delegatecall`/`.send` whose return value is ignored
  (SCWE-048); swallowed reverts / `try/catch` that hides failure (SCWE-121/146); missing
  return-data length validation (SCWE-120); call to non-contract address (SCWE-134);
  `extcodesize`-based existence checks that are bypassable (SCWE-144).
- **Confirm:** show a failed external call leaving the contract in an inconsistent state that
  benefits the attacker.

## SC07 — Arithmetic Errors (rounding & precision)
- **Look for:** division before multiplication; rounding that favors the user; inconsistent
  rounding direction in financial math (SCWE-124); ERC4626 share inflation via donation
  (SCWE-135); precision loss in share/interest/AMM invariant math repeated within one tx.
- **Confirm:** compute the per-operation error and show it compounds (loop or batched swap)
  into meaningful value extraction. (cf. Balancer V2, ~$128M, Nov 2025.)

## SC08 — Reentrancy
- **Look for:** external call before state update (violates checks-effects-interactions,
  SCWE-102/046); missing `nonReentrant`; **read-only reentrancy** where a view function
  returns stale state mid-transition and other contracts trust it (SCWE-137); ERC721/1155
  `safeTransfer`/`onERC*Received` callback reentrancy (SCWE-138); ERC777 `tokensReceived`
  hooks (SCWE-104); cross-function and cross-contract reentrancy.
- **Confirm:** identify the re-entrant path and the state that's exploitable while stale
  (repeated withdrawal, double-mint, manipulated view consumed by a dependent protocol).

## SC09 — Integer Overflow / Underflow
- **Look for:** `unchecked { }` blocks; inline assembly arithmetic; casts/downcasts that
  truncate (SCWE-041/080); pre-0.8.0 code without SafeMath. (0.8+ reverts by default, so
  focus on `unchecked` and assembly.)
- **Confirm:** show an input that wraps a value and breaks an invariant (e.g. underflowed
  balance → huge number).

## SC10 — Proxy & Upgradeability
- **Look for:** uninitialized implementation / missing `_disableInitializers()` in
  constructor (SCWE-092 — uninitialized ERC1967 proxies were an automated mass-exploit
  campaign in 2025); storage-layout collision on upgrade (SCWE-099/150); unauthenticated
  upgrade or beacon upgrade (SCWE-118); init front-running (SCWE-098); `selfdestruct` in
  implementation (SCWE-117); shared proxy-admin/logic-owner key (SCWE-119).
- **Storage-layout discipline (upgrade-safety, even without an attacker).** New state in an
  upgradeable contract must be *appended* after existing vars, and any trailing `__gap` must
  shrink by exactly the number of slots the additions consume **after packing** (e.g. an
  `address` + a `uint8` share one 32-byte slot, so the gap drops by 1, not 2). Inserting a
  variable between existing ones, or reordering, is a true storage collision that silently
  corrupts a live slot on upgrade. Off-by-one `__gap` or mis-packing is usually still
  upgrade-safe but orphans a slot and signals the layout wasn't checked — flag it. Verify with
  `solc --storage-layout` and diff against the **deployed** bytecode's layout before upgrading,
  not just the new source.
- **Confirm:** show an attacker initializing or upgrading to seize control, or a storage
  collision corrupting a critical slot after upgrade. For a `__gap`/packing nit with no
  collision, scope it as Informational (latent upgrade-hygiene), not a theft finding.

## Cross-cutting

**Signatures & crypto.** ECDSA malleability (SCWE-054); missing nonce/domain separator →
replay (SCWE-055/105/147); `ecrecover` returning `address(0)` not handled; cross-chain
replay from missing chain ID (SCWE-107); hash collisions with packed variable-length args
(SCWE-074). Prefer OZ ECDSA + EIP-712.

**Randomness.** `block.timestamp`/`blockhash`/`block.prevrandao` for high-value randomness
is predictable/manipulable (SCWE-024/084/153). Require a VRF.

**DoS / griefing.** Unbounded loops over attacker-growable structures (SCWE-109/148); push
payments that revert; gas-limited calls; unbounded withdrawal queue (SCWE-126); locked ether
with no withdrawal path (SCWE-140). Also: a call into an external dependency (compliance
registry, allowlist, oracle) on a core path (e.g. inside `_update`/transfer) that can
**revert** instead of returning a status, and isn't wrapped in `try/catch`, bricks that path
for affected accounts. Distinguish *fail-closed for theft* (a revert blocks the action — safe
against loss) from *availability* (the same revert is an unintended DoS): if the dependency's
interface says "a revert MUST be treated as not-approved," the caller must `try/catch` and map
a revert to the not-approved branch, and the two layers of a system should handle it
consistently. Rate these as availability findings, usually Low/Medium, not theft.

**MEV / ordering.** Sandwichable swaps, missing slippage/deadline, `permit()` front-running
nonce DoS, commit-reveal absence on sensitive flows.

## Token integration & "weird ERC20s"
A huge share of integration bugs come from assuming all tokens behave like a textbook ERC20:
- Missing return values (USDT) — use `SafeERC20`.
- Fee-on-transfer (SCWE-110) — measure balance delta, don't trust the amount argument.
- Rebasing (SCWE-111) — stored balances drift.
- ERC777/1155 callback hooks — reentrancy vector on transfer.
- Approval race / double-spend (SCWE-103); blocklists; non-18 decimals; pausable tokens.
Always ask: *what breaks if this token is one of the weird ones?*

## Dead code, missing wiring & unreachable state
Not every defect is a named attack class. Watch for functions whose dependency is never
initialized and has no setter (so they can never work), privileged setters that were added
without access control, dead/unreachable branches, and state that's declared but never used.
These are correctness/latent-risk findings — usually Informational, but raise the severity if
the "missing wiring," once added, would itself be exploitable (e.g. an unguarded oracle setter).

## Worked example (calibration anchor)
Keep findings at roughly this length and concreteness — enough to prove the path, no filler:

> **[CRITICAL] Unprotected `setOwner` allows ownership takeover**
> - Severity: Critical (Impact Critical × Likelihood High)
> - Location: `Vault.sol:13`
> - Category: SC01 Access Control (SCWE-038/049)
>
> **Description.** `setOwner(address)` has no access control; any account can set itself owner.
> **Exploit scenario.** Attacker calls `setOwner(attacker)`, then `adminWithdraw(attacker,
> address(this).balance)` to drain all ETH. Single tx, permissionless.
> **Recommendation.** Add `require(msg.sender == owner)` to `setOwner` (line 13); prefer
> `Ownable2Step` to avoid transferring ownership to a wrong/zero address.

## Quick grep/triage hints
Use these to locate surface fast, then review manually — never report from grep alone:
- `tx.origin` → auth bug candidate
- `delegatecall`, `.call{value:` → external-call / proxy review
- `unchecked`, `assembly` → arithmetic review
- `block.timestamp`, `blockhash`, `prevrandao` → randomness/time
- `initialize`, `initializer`, `_disableInitializers` → proxy init
- `getReserves`, `slot0`, `balanceOf(` in pricing → oracle manipulation
- `onERC721Received`, `tokensReceived`, `safeTransfer` → callback reentrancy
- `pragma solidity ^` (floating, SCWE-060) and old versions (SCWE-061)
