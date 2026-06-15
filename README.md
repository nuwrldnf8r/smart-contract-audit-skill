# smart-contract-audit

> A **Claude skill** (Claude Code / Claude.ai / Cowork) for auditing smart-contract and on-chain
> program code — Solidity/Vyper, Solana, and CosmWasm — for vulnerabilities and exploits.

A rigorous, multi-ecosystem **security audit skill** for [Claude](https://claude.com) — built
to review real smart-contract and on-chain program code for vulnerabilities, exploits, and
attack vectors across three ecosystems:

- **Solidity & Vyper / EVM** (including L2, cross-chain/bridge, and account-abstraction concerns)
- **CosmWasm** (Rust / Cosmos)
- **Solana** (Rust / Anchor & native)

It is attacker-centric by design. Rather than pattern-matching a checklist, it builds a model
of what the protocol is trying to do, defines the invariants that must hold, and then reasons
about how an adversary breaks the gap between intent and implementation — which is where most
real losses come from. It runs static analyzers when they're available (Slither, Aderyn,
cargo-audit) and falls back to thorough manual review when they aren't.

## What it produces

A severity-rated findings report. Each finding includes its location, a concrete exploit
scenario (the actual sequence of calls an attacker makes and what they gain), and a specific,
actionable remediation. Severities are scored Impact × Likelihood, with an explicit discipline
against over-claiming. The report also carries:

- An **insider-resistance ("INS") analysis** — for each privileged role, it assumes the role
  check *passes* and the holder is hostile or compromised, then asks what one transaction can
  extract or brick and whether that power is **bounded** (timelock, cap/rate-limit, deviation
  band, role separation). An unbounded power over user funds is reported as a finding even under
  a "trusted" multisig — the dominant 2025–2026 loss vector.
- A clear **deployment verdict** — **GO / GO-with-conditions / NO-GO**, with any required
  deployment-ordering gates and residual risk stated explicitly rather than buried in prose.

## How it works

The skill follows a seven-phase methodology (Phases 0–6):

0. **Scope & system model** — what the protocol does, who holds privilege, where value lives.
1. **Invariants** — the properties that must always hold; most high-severity findings break one.
2. **Automated pass** — Slither / Aderyn / cargo-audit if installed (leads to verify, not findings).
3. **Manual review** — function-by-function against an ecosystem vector catalogue + the invariants.
4. **Cross-cutting analysis** — oracle manipulation (incl. L2 sequencer-uptime), flash-loan
   amplification, rounding/precision, MEV/ordering, composability, upgradeability, cross-chain/
   bridge messaging, account abstraction, governance, and insider/privileged-power abuse.
5. **Severity & report** — score each finding Impact × Likelihood and write it up with an exploit
   path and a specific fix.
6. **Adversarial self-verification** — re-derive each finding's exploit path before it ships;
   remove anything that can't be substantiated.

The vulnerability catalogues are grounded in the OWASP Smart Contract Top 10 (2026), the OWASP
SCWE weakness registry, the Sealevel attack classes (Solana), and CosmWasm audit practice.

## Repository layout

```
.
├── .claude-plugin/                # Claude Code plugin + marketplace manifests
│   ├── plugin.json
│   └── marketplace.json
├── skills/
│   └── smart-contract-audit/      # the skill itself
│       ├── SKILL.md               # entry point: workflow + routing
│       ├── references/            # methodology, per-ecosystem vectors, severity, tooling
│       └── assets/                # the audit report template
├── evals/                         # reproducible evaluation harness
│   ├── easy/                      # classic planted-bug contracts + ground truth
│   ├── hard/                      # subtle planted-bug contracts + ground truth
│   └── trigger-evals.json         # should-trigger / should-not-trigger queries
└── docs/
    └── install.md                 # full setup, including the static-analysis tools
```

## Installation

### As a Claude Code plugin (recommended)

This repo is also a Claude Code plugin marketplace, so you can install it in two commands:

```text
/plugin marketplace add nuwrldnf8r/smart-contract-audit-skill
/plugin install smart-contract-audit@smart-contract-tools
```

The skill then activates automatically when you ask Claude to audit or security-review on-chain
code.

### As a plain skill

Copy `skills/smart-contract-audit/` into your Claude skills directory (Claude Code, or the
Cowork/desktop skills folder). The skill activates automatically when you ask Claude to audit
or security-review on-chain code — you don't invoke it by name.

### The analysis tools (optional but recommended)

The skill auto-detects these on your `PATH` and uses them in its automated pass; if they're
absent it does manual review only. See [`docs/install.md`](docs/install.md) for the full guide.

```bash
# Slither (Solidity) — via an isolated venv to avoid PEP 668 issues
python3 -m venv ~/.slither && ~/.slither/bin/pip install slither-analyzer solc-select
export PATH="$HOME/.slither/bin:$PATH"        # add to your shell rc
solc-select install 0.8.24 && solc-select use 0.8.24

# Aderyn (Solidity, Rust-based)
curl -L https://raw.githubusercontent.com/Cyfrin/up/main/install | bash && cyfrinup

# cargo-audit (CosmWasm / Solana dependency advisories)
cargo install cargo-audit
```

## Usage

In Claude, point it at the code and describe the job:

> "Audit this Solidity vault before I deploy to mainnet — it holds user deposits."

> "Security-review our Anchor staking program in `./programs/` and flag any account-validation
> or signer issues."

> "Can someone drain this CosmWasm contract?"

Give it the **actual code** (a folder, repo, or pasted files), say what the protocol does and
what holds value, and note the scope (whole repo vs. a diff) — the system model is sharper with
that context, and a sharper model finds more.

## Evaluation

The `evals/` directory contains a reproducible test harness used to validate the skill: planted-
bug contracts across all three ecosystems (an "easy" classic-bug set and a "hard" subtle-bug
set), with documented ground truth, plus a trigger-eval set for description accuracy. See
[`evals/README.md`](evals/README.md) for how the sets are organized and how to run them with the
[skill-creator](https://docs.claude.com) harness.

In testing, the skill detected 100% of planted bugs on both sets, and on the subtle set it
materially outperformed an unguided baseline on **severity calibration** — avoiding over-claimed
"Critical" findings whose exploit paths don't actually hold (e.g. a reentrancy that reverts on a
checked-underflow during unwind).

## Contributing

Issues and PRs welcome — new vulnerability classes, additional ecosystems, harder eval cases, or
methodology improvements. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Disclaimer

This is an automated, AI-assisted security aid. It raises the bar and surfaces many issues, but
it is provided "as is" (see [`LICENSE`](LICENSE)) with no warranty, and a clean review is not a
guarantee that code is free of vulnerabilities. Validate findings independently and use your own
judgment before relying on any result for production code.

## License

[MIT](LICENSE) © 2026 Gavin Marshall
