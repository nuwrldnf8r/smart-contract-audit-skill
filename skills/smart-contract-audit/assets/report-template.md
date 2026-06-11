# Smart Contract Security Audit — <Project Name>

**Auditor:** AI-assisted review (Claude smart-contract-audit skill)
**Date:** <YYYY-MM-DD>
**Scope:** <files / contracts / programs reviewed>
**Commit / snapshot:** <git hash or file checksums>
**Ecosystem(s):** <Solidity | CosmWasm | Solana>
**Tools run:** <e.g. Slither 0.10.x, Aderyn; or "none available — manual review only">

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

## 4. Methodology
Phases performed, tools run (and not run, with reason), and coverage limitations.

## 5. Findings
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

**Recommendation.** Specific, actionable fix referencing the exact code (not "add validation"
but "add `require(msg.sender == owner)` to `setOracle()` at line 142").

**Status.** Open / Acknowledged / Fixed (for re-reviews).
---

## 6. Informational & gas notes
Non-security code quality, best-practice, and gas items, clearly separated from real risk.

## 7. Centralization & operational risk
Privileged roles, what each can do, key-management assumptions, and what happens if a
privileged key is compromised or malicious. State this even if the code is "correct" — the
largest 2025–2026 losses came from operational and governance compromise, not just code bugs.

## 8. Appendix
Tool output references, definitions, and any out-of-scope observations.
