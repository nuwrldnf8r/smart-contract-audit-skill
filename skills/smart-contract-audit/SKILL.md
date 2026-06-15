---
name: smart-contract-audit
description: >-
  Rigorous security audit of smart-contract / on-chain program code in Solidity (EVM),
  CosmWasm (Rust/Cosmos), or Solana (Rust/Anchor & native). Use whenever the user wants their
  actual contract or program code reviewed for vulnerabilities, exploits, or attack vectors —
  e.g. "audit/security-review this contract", "is this safe to deploy?", "can someone drain or
  exploit this?", before a mainnet deploy or audit contest, on a fork or diff, or when they paste
  .sol/.rs code and ask what could go wrong. Trigger even without the word "audit" and for a
  single contract; covers reentrancy, oracle manipulation, access control, flash-loan,
  proxy/upgradeability, PDA/account validation, and economic/logic bugs, and yields a findings
  report with severities. Do NOT trigger when the user only wants to: explain or learn a concept;
  pick between tools/oracles or get design advice; write, refactor, gas-optimize, or translate a
  contract; debug build/test/deploy scripts; or audit non-contract security (web, cloud/IAM,
  infra).
---

# Smart Contract Security Audit

This skill drives a thorough, methodical security audit of smart-contract code across
the three ecosystems the user works in: **Solidity/EVM**, **CosmWasm (Cosmos)**, and
**Solana**. It combines a disciplined manual review methodology with optional static
analysis tooling, a current vulnerability taxonomy, a calibrated severity model, and a
professional report format.

The goal is not to pattern-match a checklist and stop. It is to **understand what the
protocol is trying to do, then reason about how an adversary breaks the gap between
intent and implementation.** Most real losses come from logic and access-control flaws
that no linter flags — so checklists are the floor, not the ceiling.

## Be exhaustive — thoroughness is the product

A security audit is the one task where you must NOT optimize for speed, brevity, or token
economy. The deliverable is completeness. Missing a single Critical finding defeats the
entire exercise, no matter how clean the rest of the report looks. Internalize the following
as hard rules, not suggestions:

- **Read every in-scope file, in full.** Never skim, sample, or assume a file is fine because
  it "looks standard." If a file is long, read it in multiple passes — but read all of it,
  including imports, libraries, interfaces, and test files (tests reveal intended behavior and
  thus invariants). Do not summarize code away before you've understood it line by line.
- **Enumerate and review the entire attack surface — every function, not a representative
  subset.** List every externally-reachable function, every privileged path, every external
  call, every place untrusted input reaches state or math. Then check each one off explicitly.
  Coverage is something you track, not something you eyeball.
- **Do not stop early.** Finding a few issues does not mean you're done — if anything, one bug
  raises the odds of others nearby. Keep going until the whole surface is covered and every
  invariant has been actively attacked. "I found the main bug" is not a stopping condition;
  "I have reviewed everything" is.
- **Use subagents to remove context budget as an excuse.** For anything beyond a small single
  file, parallelize: spawn one subagent per contract/module/file, each instructed to read its
  files completely and report *all* findings (not just the top few), then consolidate and
  de-duplicate. This is the mechanism that lets you be exhaustive without running out of room —
  use it rather than trimming scope to fit. A dedicated subagent re-deriving the top findings
  independently (methodology Phase 6) is also strongly encouraged.
- **When unsure whether something is worth checking, check it.** Prefer false thoroughness over
  false economy. The cost of an extra check is trivial; the cost of a missed exploit is total.
- **Track coverage in the workspace.** Keep a running list of functions/files reviewed and
  invariants tested so nothing silently falls through. Report your coverage honestly, including
  anything you did not get to and why.

Token and time cost are not constraints the user is trying to minimize here. Spend what the
job requires. If you find yourself abbreviating to save space, that is a signal to spawn a
subagent and go deeper, not to cut.

## When you start

Before reading any vulnerability references, establish context. Skipping this is the most
common way audits miss the important bugs.

1. **Identify the ecosystem(s)** from file extensions and imports:
   - `.sol`, `pragma solidity`, OpenZeppelin imports → **Solidity** → read `references/solidity-vectors.md`
   - `.rs` with `cosmwasm_std`, `#[entry_point]`, `cw-storage-plus` → **CosmWasm** → read `references/cosmwasm-vectors.md`
   - `.rs` with `anchor_lang`, `solana_program`, `#[program]`, `Accounts` → **Solana** → read `references/solana-vectors.md`
   - A repo may contain more than one. Audit each with its own reference.
2. **Build a mental model of the system.** What does it do (lending, AMM, vault, bridge,
   staking, governance, NFT)? Where does value live? Who are the privileged actors? What
   are the trust assumptions and external dependencies (oracles, other protocols, tokens)?
   For each privileged actor note not just what it *can* do but whether that power is
   **bounded** (timelock, cap/rate-limit, role split) — an unbounded admin power is an
   insider-threat finding, not just a disclosure (the "INS" lens; see methodology Phase 4).
3. **Map the attack surface:** every externally callable entry point, every place value
   moves, every privileged function, every external call, every place user input reaches
   state or math.
4. **Define the invariants** that must always hold (e.g. "total shares ≤ total assets",
   "only the owner can pause", "you can never withdraw more than you deposited"). Most
   high-severity findings are a way to violate one of these. Write them down explicitly —
   they drive the whole review.

Then read `references/methodology.md` for the full review procedure, and the relevant
ecosystem reference(s). Read `references/severity-rubric.md` before assigning severities,
and use `assets/report-template.md` for the deliverable.

## The audit workflow

Follow `references/methodology.md` in full. In brief:

1. **Scope & context** — establish the system model and invariants (above). Record commit
   hash / file list so the audit is reproducible. If the request is a **diff/PR/fork** review,
   treat the change as the focus but read the full functions and contracts it touches — a
   small diff can break an invariant defined far away, and appended storage can brick an
   upgrade (see methodology Phase 0, "Diff / delta-scoped audits").
2. **Automated pass (hybrid)** — run static analyzers if available (Slither/Aderyn for
   Solidity, `cargo audit`/clippy for Rust). Treat their output as leads to verify, not
   findings. See `references/tooling.md`. If tools aren't installed, say so and proceed
   with manual review — do not block.
3. **Manual review** — go function by function against the ecosystem vector reference AND
   reason from the invariants. This is where the real findings come from. Trace untrusted
   input to sensitive sinks; trace value flows; question every external call and every
   privileged path.
4. **Cross-cutting analysis** — economic/logic attacks (flash-loan-amplified, oracle
   manipulation, rounding/precision, MEV/ordering), composability and integration risk
   (weird ERC20s, callback tokens, upgrade/proxy risk), and governance.
5. **Severity & triage** — score each finding with `references/severity-rubric.md`
   (Impact × Likelihood). Be honest about likelihood; don't inflate.
6. **Report** — write findings using `assets/report-template.md`: each with severity,
   location, description, a concrete exploit scenario, and a specific remediation. Also record
   what you verified as *correct* (signals coverage) and end with a clear **verdict**
   (GO / GO with conditions / NO-GO), listing any deployment-ordering gates and residual risk.
7. **Verify before delivering** — re-read each finding adversarially: is it actually
   reachable and exploitable given the real access control and call paths? Remove or
   downgrade anything you can't substantiate. False positives destroy trust in the report.
   Include a final self-check pass; for a large/high-stakes codebase, use a subagent to
   independently re-derive the top findings.

## Reference files

Read these as needed (progressive disclosure — don't load all of them unless the codebase
spans all ecosystems):

- `references/methodology.md` — the full step-by-step review procedure, how to think like
  an attacker, and the cross-cutting (logic/economic) analysis that catches the bugs
  linters miss. **Read this on every audit.**
- `references/solidity-vectors.md` — EVM/Solidity vulnerability catalogue, mapped to OWASP
  SC Top 10 (2026) and the OWASP SCWE registry, with detection guidance.
- `references/cosmwasm-vectors.md` — CosmWasm/Cosmos-specific vulnerability catalogue.
- `references/solana-vectors.md` — Solana (Anchor & native) vulnerability catalogue,
  built around the Sealevel attack classes.
- `references/severity-rubric.md` — Impact × Likelihood scoring, with calibration examples.
- `references/tooling.md` — how to run and interpret static/dynamic analysis tools per
  ecosystem, and how to fall back gracefully when they're absent.
- `assets/report-template.md` — the standard audit report format. Copy and fill it.

## Operating principles

- **Intent first, patterns second.** A checklist that doesn't understand the protocol will
  miss the business-logic bug that drains it. Spend real effort on the system model.
- **Prove exploitability.** A finding without a plausible attack path is noise. For each
  one, write the concrete sequence of calls an attacker makes and what they gain.
- **Calibrate severity honestly.** Reserve Critical/High for issues with real impact and
  real reachability. Over-flagging trains the reader to ignore you.
- **Insider resistance, not just access control.** Don't stop at "is the role check enforced."
  Assume each privileged role is hostile/compromised and the check passes, then ask what one
  transaction can extract or brick — and whether that power is bounded. Unbounded power over
  user funds is an insider (INS) finding even under a trusted multisig; the fix is to bound it.
- **No fabricated certainty.** If something is suspicious but you can't confirm it, say so
  and flag it for manual follow-up rather than asserting a vuln that isn't there.
- **Be specific in remediation.** "Add access control" is weak. "Add `onlyOwner` to
  `setOracle()` (line 142); it currently lets any caller repoint the price source" is useful.
