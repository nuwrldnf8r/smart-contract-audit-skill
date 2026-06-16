# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [1.2.0] — 2026-06-16

### Added
- **Protocol-class playbooks** (progressive-disclosure references, read only when the protocol
  type matches):
  - `references/protocol-erc4626-vault.md` — tokenized/yield vaults: first-depositor/donation
    share inflation, rounding direction, `totalAssets` integrity & read-only reentrancy, weird
    assets, integrator/slippage footguns.
  - `references/protocol-lending.md` — money markets/CDPs: oracle & collateral valuation,
    health-factor math & accrual ordering, liquidation/bad-debt, receipt-token inflation, caps/modes.
  - `references/protocol-amm-dex.md` — AMMs: invariant/swap rounding, first-liquidity
    manipulation, spot-price-as-oracle, reserve desync, flash-swap/hook reentrancy, slippage/MEV,
    and v3/v4 + stableswap specifics.
- Wired into `SKILL.md` and `methodology.md` Phase 0: classify the protocol, then read the
  matching playbook alongside the ecosystem vectors. Other classes (staking, bridge, governance,
  perps, NFT) intentionally have no dedicated playbook yet — they use the cross-cutting analysis.

## [1.1.0] — 2026-06-16

Shift from contract-code audit toward protocol-in-context audit.

### Added
- `references/repo-execution-safety.md` — reviewing a hostile repo safely: no untrusted code
  execution, `.env`/key hygiene, dependency-by-reading, prompt injection from repo text, pinned
  tool versions. Wired into the workflow as a Phase-0 precondition.
- `references/deployment-live-state.md` — auditing deployment configuration and live on-chain
  state: proxy/upgrade authority, timelock *actual* delay, multisig threshold/signer
  concentration, initialize-once, and chain-ID address correctness. New report section.
- Account-abstraction / **EIP-7702** (post-Pectra delegated EOAs) plus ERC-4337 and EIP-1271
  coverage in `solidity-vectors.md`, including the broken `tx.origin` / `msg.sender == tx.origin`
  EOA-check and cross-chain `chainId = 0` authorization replay.
- Report template: an **Assumption Ledger** and a per-finding **Proof / Reproduction** block
  (Foundry PoC / Echidna–Medusa invariant / exploit script / mathematical counterexample).
- `evals/negative/` — safe-but-suspicious fixtures across all three ecosystems with a
  `should_not_flag` ground truth, to grade false-positive / over-claiming resistance directly.

### Changed
- `SKILL.md` workflow now has explicit untrusted-repo and deployment/live-state steps, a louder
  unsupported-ecosystem (Move/Cairo/Soroban/ink!/Clarity) "results are partial" caveat, and
  pointers to the new references.

## [1.0.0] — 2026-06-11

Initial release.

### Added
- `smart-contract-audit` skill covering **Solidity/EVM**, **CosmWasm (Cosmos)**, and
  **Solana (Anchor & native)**.
- Seven-phase attacker-centric methodology, Phases 0–6 (scope → invariants → automated pass →
  manual review → cross-cutting/economic analysis → severity & report → adversarial
  self-verification).
- Per-ecosystem vulnerability catalogues grounded in OWASP Smart Contract Top 10 (2026), the
  OWASP SCWE registry, the Sealevel attack classes, and CosmWasm audit practice.
- Impact × Likelihood severity rubric with calibration examples, and a standard report template.
- Hybrid tooling support: auto-detects and uses Slither / Aderyn / cargo-audit when present,
  falls back to manual review otherwise.
- Reproducible evaluation harness: planted-bug contracts (easy + hard sets) with ground truth,
  plus a trigger-eval query set for description accuracy.
- Installation guide, contributing guide, and MIT license.

### Validation
- 100% detection of planted bugs on both the easy and hard eval sets.
- On the hard (subtle-bug) set, materially better severity calibration than an unguided baseline —
  avoided over-claimed Critical findings whose exploit paths do not hold under scrutiny.
