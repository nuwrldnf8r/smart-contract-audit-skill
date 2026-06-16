# Smart Contract Security Audit — <Project Name>

**Auditor:** AI-assisted review (Claude smart-contract-audit skill)
**Date:** <YYYY-MM-DD>
**Scope:** <files / contracts / programs reviewed>
**Commit / snapshot:** <git hash or file checksums>
**Ecosystem(s):** <Solidity | CosmWasm | Solana>
**Tools run:** <e.g. Slither 0.10.x, Aderyn 0.x; or "none available — manual review only" — pin exact versions>

> _Automated, AI-assisted security review — an aid, not a guarantee that the code is free of
> vulnerabilities. Validate findings independently before relying on them._
<!-- Internal use: delete the disclaimer line above if you supply your own. -->

---

## 1. Executive summary
2–4 sentences: what the protocol does, overall security posture, and the headline risks.
Then the findings count table:

| Severity | Count |
|----------|-------|
| Critical | |
| High | |
| Medium | |
| Low | |
| Informational | |

## 2. System overview
The system model from methodology Phase 0: what it does, the actors and their privileges,
where value lives, external dependencies and trust assumptions.

## 3. Invariants reviewed
The key invariants (Phase 1) the audit tested, and whether each held.

### What was verified correct
Briefly record the areas you actively checked and found sound (e.g. "burn gate preserved",
"compliance check runs before value movement, inside `nonReentrant`", "storage layout
upgrade-safe"). This is not filler — it signals coverage, distinguishes "reviewed and fine"
from "not reviewed", and is what lets the reader trust the findings list is complete. A short
table (Area | Assessment) works well.

## 4. Assumption ledger
Every assumption the review relied on. Audits fail when assumptions stay invisible; this turns
them into a reviewable artifact. One row per assumption — make each one falsifiable.

| Assumption | Where relied on | What breaks if false | How verified |
|------------|-----------------|----------------------|--------------|
| e.g. USDC has 6 decimals | collateral accounting | over/under-collateralization | mainnet token address |
| e.g. oracle updates within 1h | borrow/liquidation logic | stale borrow, bad liquidation | feed heartbeat config |
| e.g. admin is a ≥48h timelock | parameter safety | instant rug / brick | on-chain timelock owner check |

Mark each as **Verified**, **Assumed (unverified)**, or **Deployment-time requirement** (source-only
review — the team must confirm at launch).

## 5. Methodology
Phases performed, tools run (and not run, with reason — pin versions), and coverage limitations.
State the ecosystem coverage depth honestly: if the code targets a VM without a dedicated
vulnerability catalogue in this skill (e.g. Move/Sui/Aptos, Cairo/Starknet, Soroban, ink!,
Clarity), say so — the review is then a general logic/economic review, and results are partial.

## 6. Findings
One subsection per finding, ordered by severity. Use this exact structure:

---
### [SEVERITY] <Title>
- **ID:** <SCA-01>
- **Severity:** Critical / High / Medium / Low / Informational  (Impact <…> × Likelihood <…>)
- **Location:** `<file>:<lines>` (list all affected locations for one root cause)
- **Category:** <e.g. SC01 Access Control / Reentrancy / Solana signer check>

**Description.** What the flaw is and why it's a problem.

**Exploit scenario.** The concrete attacker call sequence and what they gain. A finding
without a plausible path is at most Informational — say so if you can't construct one.

**Proof / Reproduction.** _(Expected for every High/Critical; encouraged otherwise.)_ The
strongest evidence you can produce that the bug is real, not just plausible:
- **PoC type:** Foundry test / Echidna–Medusa invariant / minimal exploit script / mathematical
  counterexample / differential test vs. the upstream protocol (for a fork).
- **Minimal call sequence:** <the ordered calls that trigger it>
- **Preconditions:** <state/config the attack needs>
- **Expected (intended) result vs. actual exploitable result:** <the invariant that breaks>
- **Regression test to add:** <the property the fix should make pass>

If you could not build a PoC, say why (not reachable in a sandbox, needs live state, etc.) — do
not imply one exists. Any execution must follow `repo-execution-safety.md` (sandboxed, keyless).

**Recommendation.** Specific, actionable fix referencing the exact code (not "add validation"
but "add `require(msg.sender == owner)` to `setOracle()` at line 142").

**Status.** Open / Acknowledged / Fixed (for re-reviews).
---

## 7. Informational & gas notes
Non-security code quality, best-practice, and gas items, clearly separated from real risk.

## 8. Deployment & live-state
_(Include for any deployed or about-to-be-deployed system; see `deployment-live-state.md`.)_
Proxy/implementation and upgrade authority, ownership/role holders, timelock **actual delay**,
multisig threshold and signer concentration, initialization-once, and whether every configured
external address (oracle, token, router, bridge) is the correct contract **on this chain ID**.
Source-only review: convert each item into a launch checklist and record it in §4.

## 9. Centralization, insider & operational risk
Privileged roles, what each can do, key-management assumptions, and what happens if a
privileged key is compromised or malicious. State this even if the code is "correct" — the
largest 2025–2026 losses came from operational and governance compromise, not just code bugs.

For each privileged power, assess **insider resistance**: assume the holder is hostile or
compromised and the access check passes, then record the worst single action and whether the
power is **bounded** (timelock, cap/rate-limit, price-deviation band, role separation, limits
that apply to admins too, immutability). A table works well:

| Role / power | Worst single action | Bounded? (how) | Residual insider risk |
|--------------|---------------------|----------------|-----------------------|

Unbounded powers over user funds are insider-threat (INS-class) findings — promote them into
§6 (Findings) with a severity, not just a disclosure here. The fix is to bound the power, not
remove the role.

## 10. Verdict
A direct bottom line the reader can act on, per deployable unit if the system has several:
- **GO** — no blocking issues found in scope.
- **GO with conditions** — safe to proceed once specific items are resolved; list them as
  concrete, verifiable gates (e.g. "fix C-1 try/catch", "run storage-layout diff green against
  deployed bytecode", "grant role before pointing at registry").
- **NO-GO** — do not deploy; state the blocking finding(s).

State residual risk carried from prior reviews or left unresolved, and any deployment-ordering
gates, so nothing safety-critical lives only in prose. For a re-review, reconcile against the
prior verdict (what changed, what's still open).

## 11. Appendix
Tool output references (with pinned versions), definitions, and any out-of-scope observations.
