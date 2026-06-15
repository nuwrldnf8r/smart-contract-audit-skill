# Audit Methodology

This is the procedure to follow on every audit, regardless of ecosystem. It is
deliberately attacker-centric: the vulnerability catalogues tell you *what* to look for,
but this file tells you *how to think* so you find the bugs that aren't in any catalogue.

## Table of contents
1. Phase 0 — Scoping and system modeling
2. Phase 1 — Invariant identification
3. Phase 2 — Automated analysis (hybrid)
4. Phase 3 — Manual review (the core)
5. Phase 4 — Cross-cutting / economic analysis
6. Phase 5 — Severity, triage, and reporting
7. Phase 6 — Adversarial self-verification

---

## Phase 0 — Scoping and system modeling

You cannot find a logic bug in a system you don't understand. Before looking for bugs,
reconstruct the designer's mental model.

- **Inventory the code in scope.** List every contract/program/module, record the commit
  hash or file checksums, and note dependencies (libraries, oracles, external protocols).
  An audit must be reproducible against a fixed snapshot.
- **Classify the protocol.** Lending, AMM/DEX, vault/yield, staking, bridge, governance,
  NFT/marketplace, stablecoin, perps — each has characteristic failure modes. Name it.
- **Identify the actors and their powers.** Users, LPs, owner/admin, governance, keepers,
  oracles, relayers. For each privileged role: what can it do, and what happens if its key
  is compromised or malicious? Centralization *is* a finding. Go past "this role is trusted":
  for every privileged power, ask whether it is **bounded** — timelocked, capped/rate-limited,
  split across roles, applied even to admins, or made immutable — or whether a single
  malicious/compromised holder can cause loss in one transaction. Unbounded power over user
  funds is an insider-threat finding even under a "trusted" multisig (see Phase 4, Insider /
  privileged-power abuse).
- **Map external trust.** Every external call, oracle read, and token interaction is a
  trust assumption. Which external contracts must behave honestly for safety to hold?
- **Follow the money.** Where does value enter, sit, and leave? Funds-at-rest and the exact
  functions that move them are the highest-value review targets.

Write a short "system summary" — this becomes the report's overview and forces you to
confront anything you don't actually understand yet.

**Diff / delta-scoped audits.** When the request is to review a specific change (a PR, a
commit, a fork diff) rather than the whole codebase, the diff is the *focus*, not the
boundary. A change is only safe in context:
- Read the changed lines, then read the full functions and contracts they live in — a
  three-line diff can break an invariant established 200 lines away.
- Identify what the change touches: new external calls, new storage, new privileged paths,
  altered math, altered access control. Each gets the full Phase 3 treatment.
- For anything the diff *removes* or *reorders* (a check, a state write, a call ordering),
  ask what previously depended on it.
- If the change is to an upgradeable contract, run the storage-layout check below — appended
  storage and `__gap` edits are the most common way a "small" diff bricks an upgrade.
- State the baseline explicitly (e.g. "delta of commit X against its parent") so the reader
  knows what was and wasn't re-reviewed. Carry forward unresolved findings from prior reviews
  as residual risk rather than silently dropping them.

## Phase 1 — Invariant identification

An invariant is a property that must hold in every reachable state. Most high/critical
findings are a concrete way to break one. Examples:

- Accounting: `sum(userBalances) == totalSupply`; `totalShares` tracks `totalAssets`
  monotonically; a user can never withdraw more than they are entitled to.
- Access: only `owner` can change critical parameters; only the vault can mint shares.
- Solvency: the protocol can always honor withdrawals it has promised; debt ≤ collateral
  value after every state-changing operation (post-op health check).
- Lifecycle: a position can't be liquidated and repaid in the same favorable frame; an
  initializer runs exactly once.

List the invariants explicitly. Then, for each one, ask: *what sequence of calls could
violate this?* That question, repeated across all invariants, is the audit.

## Phase 2 — Automated analysis (hybrid)

Run the tools for the ecosystem if they're installed (see `tooling.md`). They're fast and
catch the shallow, mechanical issues so your manual time goes to the deep ones.

- **Treat tool output as leads, not findings.** Static analyzers have high false-positive
  rates on detectors like reentrancy and "arbitrary call." Confirm each one by hand against
  the actual access control and call path before it goes in the report.
- **Note what the tools can't see.** No analyzer understands the protocol's economic intent,
  cross-contract composition, or oracle trust. Those gaps are your manual scope.
- **Graceful fallback.** If tools aren't available, state that in the report's methodology
  section and proceed with manual review. Never block the audit on missing tooling.

## Phase 3 — Manual review (the core)

This is where the findings that matter come from. Two complementary passes:

**Pass A — Entry-point / data-flow.** For every externally callable function:
- Who is allowed to call it? Is that enforced, and enforced correctly (right modifier,
  right check, no bypass via fallback/delegatecall/CPI)?
- What untrusted input does it accept? Trace each input to where it influences state, math,
  external calls, or addresses. Missing/weak validation is OWASP SC05 and a top loss cause.
- Does it make external calls? If so, is state updated *before* the call
  (checks-effects-interactions)? Can the callee re-enter — including read-only reentrancy
  against view functions other contracts rely on?
- What can fail silently? Unchecked return values, swallowed reverts, low-level calls.

**Pass B — Invariant-driven.** Take each invariant from Phase 1 and actively try to break
it. This catches the business-logic bugs (OWASP SC02), which are now among the costliest
classes and which no linter detects. Look especially at: ordering of operations, rounding
direction, first-depositor/empty-pool edge cases, and state that's read after it's been
made stale.

Go function by function against the ecosystem `*-vectors.md` reference, but keep the
invariants in mind the whole time — the catalogue is the checklist floor, the invariants
are the ceiling.

## Phase 4 — Cross-cutting / economic analysis

These attacks span multiple functions or contracts and are where modern losses concentrate.

- **Oracle & pricing (SC03).** How are prices obtained? Spot price from a DEX pool is
  manipulable; require TWAP with an adequate window, multiple sources, staleness/round
  checks (Chainlink `updatedAt`, `answeredInRound`), and decimal normalization.
- **Flash-loan amplification (SC04).** Assume the attacker has unlimited capital for one
  transaction. Any check that relies on "they couldn't afford to" is broken. Re-examine
  every price read, governance vote weight, and collateral calc under this assumption.
- **Arithmetic & rounding (SC07).** Precision loss, rounding that favors the user,
  share-inflation via donation (ERC4626 and analogues), invariant rounding in stable math.
  Tiny per-operation errors compound when repeated in one tx — Balancer V2 lost ~$128M this
  way in Nov 2025.
- **MEV / transaction ordering.** Front-running, sandwiching, `permit()` nonce griefing,
  missing slippage/deadline protection on swaps and liquidity operations.
- **Composability / integration.** Weird tokens (fee-on-transfer, rebasing, ERC777/ERC1155
  callback hooks, missing-return-value ERC20s), reentrant safe-transfer callbacks, and the
  behavior of every external protocol you depend on under stress.
- **Upgradeability & proxies (SC10).** Uninitialized implementations (a 2025 mass-exploit
  campaign), storage-layout collisions, unauthenticated upgrades, missing
  `_disableInitializers`, init front-running. For any upgradeable contract, verify the
  **storage layout** explicitly: new variables must be *appended* (never inserted between
  existing ones), and a trailing `__gap` must shrink by exactly the number of slots the new
  variables consume *after packing* — an `address`+`uint8` pack into one slot, so the gap
  drops by one, not two. Off-by-one gaps and mis-packing are usually upgrade-safe but signal
  sloppiness; an inserted-not-appended variable is a storage collision that corrupts a live
  slot. Confirm with `solc --storage-layout` (see `tooling.md`) and, before a real upgrade,
  diff the layout against the **deployed** contract's bytecode, not just the new source.
- **Operational & initialization ordering.** Multi-contract systems often only work if
  deployment/config steps happen in a specific order — e.g. a caller must be granted a role
  on a registry *before* it's pointed at that registry, or every call through it reverts and
  the dependent function bricks until the role lands. Trace each cross-contract dependency a
  privileged setter creates and ask: *if these two transactions land out of order, what
  breaks, and is there a window where funds or functionality are stuck?* Surface required
  ordering as an explicit deployment-checklist gate, not buried prose.
- **Insider / privileged-power abuse (the "INS" lens).** Distinct from "is access control
  enforced" (Phase 3) — here you *assume* the role check passes and the holder is hostile or
  compromised, then ask what they can extract or brick. A contract can be fully role-correct
  and still fail this. For each privileged function, classify the worst single action (confiscate
  fees, repoint the price oracle, mint, pause-and-extract, drain via an "admin rescue" path) and
  check what *bounds* it: timelock/delay giving users an exit, hard caps or rate-limits on the
  amount/parameter, deviation bands on prices, role separation (proposer ≠ executor, setter ≠
  upgrader), limits that apply to admins too, and immutability of the most dangerous params.
  An unbounded power over user funds is an insider finding regardless of how reputable the
  holder is — score it by the worst action and how much the bounds (or their absence) constrain
  it. (E.g. an unbounded `PRICE_SETTER` that lets one role set a rigged price to drain an LP, or
  a fee controller that can confiscate accrued user fees, are INS-class findings whose fix is to
  *bound* the power, not to remove the role.)
- **Governance.** Flash-loan-amplified voting, timelock bypass, weak quorum, malicious
  proposal execution with unbounded gas.
- **DoS / griefing.** Unbounded loops over attacker-growable arrays, gas-limited external
  calls, state bloat, locked funds with no withdrawal path.

Remember the 2025–2026 trend: the biggest losses increasingly chain *small* bugs together,
or come from operational/governance failures (compromised multisig signers, social
engineering of a security council) rather than a single code bug. Where the contract grants
sweeping admin power, flag the operational risk explicitly even if the code is "correct." This
is exactly why the insider/INS lens above is not optional: the dominant loss vector now is a
privileged actor turning hostile, so an unbounded admin power is a live threat, not a
theoretical one.

**Maintaining threat-intel priors.** The dated incident references throughout this skill (the
2025 uninitialized-proxy mass-exploit campaign, Balancer V2's ~$128M rounding loss in Nov 2025,
the signer-compromise trend) are *detection priors*, not background reading: each is welded to a
specific check. Keep new intel in that shape — a one-line, dated prior attached to the vector it
should make you test — and refresh it periodically. Do **not** grow a standalone "recent hacks"
catalogue: decoupled from a check it ages into noise and bloats context. Live/dynamic intel
gathering (pulling current advisories or incidents for the protocol class) is a separate concern
and should not live in this skill's always-loaded surface.

## Phase 5 — Severity, triage, and reporting

- Score every finding with `severity-rubric.md` (Impact × Likelihood). Be honest about
  likelihood — a real-but-unreachable issue is Informational, not High.
- De-duplicate: collapse the same root cause appearing in many places into one finding with
  multiple locations.
- Write each finding into `assets/report-template.md`. A finding is not done until it has a
  **concrete exploit scenario** (the actual call sequence and attacker gain) and a
  **specific, actionable fix**.

## Phase 6 — Adversarial self-verification

Before delivering, re-read the report as a skeptic trying to discredit it:

- For each finding, re-derive the exploit path. Is the function actually reachable by the
  attacker, given the real modifiers and call graph? If you can't substantiate it, downgrade
  to Informational or cut it. **False positives are worse than silence** — they erode the
  reader's trust in every other finding.
- Check for missed surface: did every entry point, every external call, and every privileged
  function get reviewed? Did you test each Phase 1 invariant?
- Spawn a subagent to independently re-derive the top findings from the code (without seeing
  your writeup) and reconcile differences when the codebase is non-trivial — a useful rule of
  thumb is roughly >500 LoC, code holding meaningful value, or any time you've flagged a
  Critical. For a tiny, low-stakes snippet this is overkill; a careful self-re-read suffices.
- State residual risk and any coverage limitations plainly (what was and wasn't reviewed, and
  any findings that need manual follow-up), so the reader knows exactly what the audit covered.
