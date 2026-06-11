# Severity Rubric

Severity = **Impact × Likelihood**. Assign both, then read severity off the matrix. The
single most important discipline here is *honesty about likelihood* — a real bug that no
attacker can actually reach is not a High. Over-flagging trains the reader to ignore the
report, which gets people hurt.

## Impact — what happens if it's exploited
- **Critical impact:** direct, permanent loss/lock of a material amount of user or protocol
  funds; full protocol takeover; arbitrary mint; bricking core functionality irreversibly.
- **High impact:** theft/loss of funds under specific (but realistic) conditions; partial
  takeover; serious accounting corruption.
- **Medium impact:** limited or temporary fund risk; DoS of a non-core function; griefing
  that costs users but not catastrophically; leak that aids another attack.
- **Low impact:** minor griefing, recoverable misconfiguration, negligible value at risk.

## Likelihood — how plausibly an attacker pulls it off
- **High:** anyone can trigger it permissionlessly, cheaply, with no special timing or
  precondition.
- **Medium:** requires a specific but attainable condition — a particular market state, a
  flash loan (cheap and available, so don't treat capital as a barrier), specific ordering,
  or a non-default but reachable config.
- **Low:** requires a privileged role to be malicious/compromised, an unlikely state, or
  large uneconomical cost; or depends on an assumption that usually holds.

## Severity matrix

| Impact \ Likelihood | High        | Medium   | Low      |
|---------------------|-------------|----------|----------|
| **Critical**        | Critical    | High     | Medium   |
| **High**            | High        | High     | Medium   |
| **Medium**          | Medium      | Medium   | Low      |
| **Low**             | Low         | Low      | Info     |

Additional labels:
- **Informational:** no direct security impact — code quality, style, gas, defense-in-depth,
  best-practice deviations. Still worth reporting, clearly separated from real risk.
- **Gas optimization:** efficiency only.

## Calibration examples
- Missing `onlyOwner` on `setOracle()`, callable by anyone, repointing price to an
  attacker pool → Critical (Critical impact × High likelihood).
- Reentrancy in `withdraw()` allowing repeated drains, no guard → Critical/High.
- Spot-price oracle manipulable via flash loan to borrow under-collateralized → High
  (treat flash-loan capital as available → Medium-High likelihood).
- Rounding favoring the user by 1 wei per op, only meaningful if looped thousands of times
  for cents → Low/Informational unless you can show real extraction.
- Owner can rug via an un-timelocked privileged function → Medium centralization finding
  (High impact × Low likelihood if owner is a reputable multisig; raise it if owner is a
  single EOA). Always disclose centralization risk regardless.
- Floating pragma / outdated compiler → Informational.
- Reentrancy where the naive re-entrant drain actually reverts (e.g. the unwind hits a
  Solidity 0.8 checked-underflow, so an attacker can't redeem more shares than they hold) →
  do NOT report a Critical "drains all funds." Scope it to the real residual risk (often a
  read-only-reentrancy view consumed by an external integrator) and rate that honestly. Always
  walk the *full* re-entrant execution including the unwind before assigning severity.

## Rules
- One root cause = one finding (list all affected locations under it), even if it appears in
  many functions.
- If you cannot construct a concrete exploit path, it is at most Informational — say what
  would need to be true for it to be exploitable.
- Centralization/admin-key risk is legitimately reportable; score it by what the privileged
  actor can do and how trustworthy/decentralized that actor is, and always state it plainly.
