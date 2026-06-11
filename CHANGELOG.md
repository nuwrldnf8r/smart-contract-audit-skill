# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [1.0.0] — 2026-06-11

Initial release.

### Added
- `smart-contract-audit` skill covering **Solidity/EVM**, **CosmWasm (Cosmos)**, and
  **Solana (Anchor & native)**.
- Six-phase attacker-centric methodology (scope → invariants → automated pass → manual review →
  cross-cutting/economic analysis → severity + adversarial self-verification).
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
