# Installation Guide

## 1. Install the skill

### Option A — Claude Code plugin (recommended)

This repo ships a Claude Code plugin marketplace, so the fastest install is:

```text
/plugin marketplace add nuwrldnf8r/smart-contract-audit-skill
/plugin install smart-contract-audit@smart-contract-tools
```

`/plugin marketplace add` registers the marketplace defined in `.claude-plugin/marketplace.json`;
`/plugin install` then pulls in the `smart-contract-audit` plugin (whose skill lives under
`skills/`).

### Option B — Copy the skill directory

Copy the skill directory into your Claude skills location:

- **Claude Code:** place `skills/smart-contract-audit/` where your other skills live (e.g.
  `~/.claude/skills/smart-contract-audit/`).
- **Cowork / desktop:** use the "Save skill" flow with a packaged `.skill` (zip the skill
  directory and rename to `smart-contract-audit.skill`), or drop the folder into your skills
  directory.

The skill activates automatically when you ask Claude to audit or security-review on-chain code.

## 2. Install the static-analysis tools (optional, recommended)

The skill probes your `PATH` for these and uses them in its automated pass. If none are present
it proceeds with manual review — nothing breaks, you just lose the fast first sweep.

### Slither (Solidity)

Requires Python 3.8+. Modern OSes mark the system Python "externally managed" (PEP 668), so a
dedicated virtual environment is the cleanest install:

```bash
python3 -m venv ~/.slither
~/.slither/bin/pip install slither-analyzer solc-select
```

Put the binaries on your `PATH` (so `slither` resolves without activating the venv):

```bash
# zsh (macOS default)
echo 'export PATH="$HOME/.slither/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
# bash
echo 'export PATH="$HOME/.slither/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc
```

Pick a compiler version matching your contracts and verify:

```bash
solc-select install 0.8.24 && solc-select use 0.8.24
slither --version
```

Alternatives: `brew install slither-analyzer` (macOS/Homebrew handles isolation), or — only if
you accept writing into your user environment — `pip3 install --user --break-system-packages
slither-analyzer`.

### Aderyn (Solidity, Rust-based)

No Python involved; installs as a prebuilt binary via Cyfrin's installer:

```bash
curl -L https://raw.githubusercontent.com/Cyfrin/up/main/install | bash
cyfrinup
aderyn --version
```

Or, with a Rust toolchain: `cargo install aderyn`.

### cargo-audit (CosmWasm / Solana)

For Rust dependency advisories:

```bash
cargo install cargo-audit
cargo audit            # run from a crate root
```

`cargo clippy` (ships with Rust) is also used for lints. Check `Cargo.toml` for
`overflow-checks = true`.

## 3. Verify

From a project root, confirm the tools resolve:

```bash
slither --version
aderyn --version
cargo audit --version
```

Any that print a version will be used automatically on your next audit. Missing ones are simply
skipped.

## Notes on restricted environments

Sandboxed environments (locked-down CI, some hosted notebooks) may block PyPI/crates.io. If an
install fails with a proxy or `403` error, install the tools on a machine with network access, or
run the audit in manual-review mode — the skill handles the absence gracefully.
