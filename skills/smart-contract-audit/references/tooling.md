# Tooling (Hybrid Static/Dynamic Analysis)

The audit is **hybrid**: run automated tools when they're installed to clear shallow issues
fast, but never depend on them and never report their raw output as findings. Tools produce
leads; manual review confirms or rejects each one. If a tool isn't available, note it in the
report's methodology section and proceed with manual review — do not block.

Always check availability first (e.g. `command -v slither`, `cargo --version`). Run tools in
the sandbox/workspace, never against untrusted code on a sensitive host beyond the sandbox.

## Solidity / EVM

**Slither** (Trail of Bits) — fast AST/static analysis, ~90+ detectors. Primary tool.
Install it once in the environment where audits run (Python 3.8+ required); the audit then
picks it up off PATH automatically.
```
# Preferred: isolated venv (cleanest on modern PEP 668 "externally-managed" Python).
# This matches docs/install.md and the README quickstart.
python3 -m venv ~/.slither
~/.slither/bin/pip install slither-analyzer solc-select
export PATH="$HOME/.slither/bin:$PATH"   # add to your shell rc so `slither` resolves
solc-select install 0.8.24 && solc-select use 0.8.24

# Equivalent alternatives (pick one): pipx
# (`pipx install slither-analyzer && pipx install solc-select` — one package per call),
# `brew install slither-analyzer`, or plain pip
# (`pip3 install slither-analyzer solc-select`; add --break-system-packages if needed).

slither --version               # confirm it's on PATH
slither . --json slither-out.json   # run: whole project, or `slither path/to/Contract.sol`
```
Note: a sandboxed/locked-down environment (e.g. a restricted CI box or the Cowork sandbox)
may block PyPI — if install fails with a proxy/403 error, you can't install here; run the
audit somewhere with network access, or proceed with manual review (the skill falls back
gracefully). High signal-but-noisy detectors to confirm manually: `reentrancy-*`,
`arbitrary-send`, `unchecked-*`, `uninitialized-*`, `tx-origin`, `suicidal`, `delegatecall-loop`.

**Aderyn** (Cyfrin) — Rust-based, fast, low false-positive, clean markdown output. Good
complement to Slither on larger codebases.
```
curl -L https://raw.githubusercontent.com/Cyfrin/up/main/install | bash   # installs cyfrinup
cyfrinup                                                                    # then installs aderyn
aderyn .            # produces report.md
```

**Other (use when warranted, often need setup):**
- `solc --hashes`/`--storage-layout` for proxy storage-collision checks.
- Mythril (symbolic execution) for deeper path bugs; slower.
- Echidna / Medusa (property fuzzing) and Foundry `forge test` invariant tests — best for
  confirming an invariant can be broken; write a property from your Phase 1 invariants.
- Semgrep with smart-contract rulesets for custom patterns.

Workflow: run Slither (and Aderyn if available) → triage output → manually confirm each
plausible hit against access control and call path → fold confirmed ones into findings.

## CosmWasm / Cosmos (Rust)

No single dominant security scanner; rely on Rust tooling plus manual review.
```
cargo audit          # RUSTSEC advisories on dependencies  (cargo install cargo-audit)
cargo clippy --all-targets -- -W clippy::all   # lints, some correctness
cargo test           # run the contract's own tests / multi-test
```
Check `Cargo.toml` for `overflow-checks = true`. Use `cargo-geiger` to flag `unsafe`. Manual
review against `cosmwasm-vectors.md` is the main event here.

## Solana (Rust / Anchor)

```
cargo audit
cargo clippy --all-targets
anchor test           # if an Anchor project (runs against a local validator)
```
- **X-Ray / Sec3 / L3X** and similar Solana static analyzers exist but often aren't
  installed; don't assume them.
- The highest-value automated step is `cargo audit` + clippy for dependency/overflow issues;
  everything account-validation-related is manual against `solana-vectors.md`.
- Check `Cargo.toml`/release profile for overflow checks.

## Interpreting and reporting tool use
- In the report methodology section, list which tools ran (and versions) and which didn't and
  why. This makes the audit reproducible and honest about coverage.
- Every tool finding that survives into the report must have been manually confirmed with an
  exploit path. Discard or mark as Informational anything you can't substantiate.
- A clean tool run is **not** evidence of safety — state this explicitly.
