# Evaluation Harness

A reproducible test set for validating the `smart-contract-audit` skill. Every contract here is
**deliberately vulnerable** — these are planted-bug fixtures for measuring detection and
severity calibration, not reference implementations. Do not deploy any of them.

## Layout

```
evals/
├── easy/                     # classic, well-known bug classes
│   ├── contracts/
│   │   ├── LendingPool.sol         # Solidity: access control, reentrancy, spot oracle,
│   │   │                           #           unchecked transfers, underflow, centralization
│   │   ├── cosmwasm_staking.rs     # CosmWasm: missing auth, unsaved storage, funds/addr validation
│   │   └── solana_vault.rs         # Solana:   missing signer, no has_one, unchecked arithmetic
│   └── ground-truth.json           # the planted bugs each contract is expected to surface
├── hard/                     # subtle bugs an unguided review tends to miss
│   ├── contracts/
│   │   ├── YieldVault.sol          # ERC4626 share inflation, rounding, read-only reentrancy
│   │   ├── cw_rewards.rs           # reply/SubMsg reentrancy, non-functional replay guard, migrate
│   │   └── sol_staking.rs          # account substitution, init_if_needed reinit, unchecked CPI authority
│   └── ground-truth.json
└── trigger-evals.json        # should-trigger / should-not-trigger queries for the description
```

## Ground-truth format

Each `ground-truth.json` lists, per contract, the planted findings the audit should detect:

```json
{
  "evals": [
    {
      "id": 0,
      "name": "solidity-lending-pool",
      "prompt": "Audit prompt a user might send",
      "files": ["contracts/LendingPool.sol"],
      "ground_truth": {
        "P1_oracle_access_control": "setOraclePair has no access control ...",
        "...": "..."
      }
    }
  ]
}
```

`trigger-evals.json` is a flat list of `{ "query": "...", "should_trigger": true|false }` used to
check that the skill's description fires on real audit requests and stays quiet on near-misses
(explaining a concept, choosing an oracle, writing/gas-optimizing a contract, non-contract security).

## Running the evals

These were run with the Claude **skill-creator** harness, which spawns a Claude instance with the
skill against each prompt and (for description tuning) measures trigger rates. The general shape:

1. **Audit quality** — for each contract, run the skill on the prompt, then grade the resulting
   report against `ground_truth` (each planted bug = one expectation; also check no fabricated
   Criticals and reasonable severity calibration).
2. **Triggering** — run the description-optimization loop over `trigger-evals.json`:

   ```bash
   python -m scripts.run_loop \
     --eval-set evals/trigger-evals.json \
     --skill-path skills/smart-contract-audit \
     --model <your-model-id> --max-iterations 5 --verbose
   ```

The harness needs an authenticated `claude` CLI. Detection is graded by comparing report findings
to ground truth; the interesting signal on the **hard** set is severity calibration, not raw
recall — a strong model finds the bugs either way, but disciplined exploit-path verification is
what separates a trustworthy report from an over-flagged one.

## Adding cases

New fixtures are welcome. Add the contract under the relevant set's `contracts/`, append its
planted bugs to that set's `ground-truth.json`, and keep each bug entry specific enough to grade
objectively (name the function and the exact flaw).
