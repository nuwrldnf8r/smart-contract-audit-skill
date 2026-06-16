# Deployment & Live-State Audit

Source code is only half of what holds funds. The other half is **how the code is configured
once deployed** — who owns the proxy, what the timelock delay actually is, which addresses the
contracts are wired to, whether the initializer ran exactly once. A contract that is flawless in
`src/` can still be a rug or a brick because of its deployment state. Many real incidents live in
exactly this gap.

Run this phase whenever the audit covers a **deployed or about-to-be-deployed** system (not a
pure source-only review). It complements the code review; it does not replace it. Findings here
get their own report section (see the report template's deployment section) and feed the
**centralization & operational risk** section.

If you only have source (no addresses yet), turn each item below into a **deployment-time
requirement** the team must verify before/at launch, and record them in the Assumption Ledger.

## What to check

### Proxy & upgrade authority

- **Implementation vs proxy.** For every proxy, identify the implementation address and confirm
  it is the audited code (not an older/different implementation). Verify on-chain bytecode
  matches the reviewed source where possible.
- **Upgrade authority.** Who can upgrade? ProxyAdmin owner (Transparent), UUPS `_authorizeUpgrade`
  gate (UUPS), beacon owner (Beacon). Confirm it is a timelock/multisig, not a single EOA.
- **Uninitialized implementation.** Confirm `_disableInitializers()` ran in the implementation
  constructor and the logic contract cannot be initialized directly (the 2025 mass-exploit class).
- **Storage layout.** For an upgrade, confirm the new layout is append-only vs the live one — no
  reordered/removed/retyped slots that would corrupt existing state.

### Ownership, roles & timelocks

- **Role inventory.** Enumerate every privileged role (`owner`, `DEFAULT_ADMIN_ROLE`, pauser,
  minter, guardian, keeper) and its current holder on-chain. Compare to what the docs claim.
- **Timelock reality.** If a timelock guards admin actions, read its **actual `delay` value** —
  a timelock set to 0 (or seconds) is theater. Confirm the timelock's proposer/executor/canceller
  roles, and that the admin of the protocol is genuinely the timelock, not an EOA that merely
  *can* route through it.
- **Multisig health.** Threshold (`m`-of-`n`), signer count, and **signer concentration** — are
  signers independent parties or one person's wallets? Are any signers stale/rotated? A 2-of-3
  where one party holds two keys is a 1-of-2.
- **Ownership handoff.** Was ownership actually transferred from the deployer EOA to the
  timelock/multisig, or is the deployer still owner? Prefer two-step (`Ownable2Step`) and confirm
  the acceptance happened.

### Initialization & wiring order

- **Initialized exactly once.** Confirm each initializer was called once, by the intended party,
  and cannot be re-called or front-run. Check the init transaction, not just the code guard.
- **Roles granted before wiring.** Were privileged roles set, and renounce/cleanup done, before
  the contracts were connected and funded? Roles granted *after* funds flow in are a window.
- **No leftover setup powers.** Deployer-only bootstrap functions (set implementation, set oracle,
  grant role) that remain callable after launch are latent takeover paths.

### Address & chain-config correctness

- **External addresses match the chain.** Oracle feeds, tokens, routers, bridges, and dependency
  protocols — confirm each configured address is the correct contract **on this chain ID** (a
  feed/token address valid on mainnet may be a different or non-existent contract on an L2 or a
  fork). Same address ≠ same contract across chains.
- **Canonical vs bridged tokens.** Confirm the configured token is the canonical one and not a
  bridged/look-alike with different decimals or behavior.
- **Chain assumptions.** Verify decimals, the expected chain ID, and any block-time/finality
  assumptions baked into config (heartbeats, deadlines, TWAP windows) suit the target chain.

### Operational readiness

- **Emergency runbooks exist.** Is there a pause path, an upgrade path, an asset-rescue path — and
  documented procedures and authority for using them? Capability without a runbook fails under
  pressure.
- **Pause/guardian scope.** What exactly can the pauser/guardian do, and can that power itself be
  abused (e.g. pause-to-grief, or a "rescue" function that is an unrestricted drain)?
- **Monitoring.** Are privileged actions, upgrades, and large flows monitored/alerted? Note the
  absence as operational risk.

## How to verify (when addresses are available)

- Read storage slots directly: ERC-1967 implementation slot
  (`0x360894...bbc`), admin slot (`0xb53127...103`), beacon slot. Use `eth_getStorageAt` /
  `cast storage` / a block explorer's read-proxy view.
- Read owner/role getters, timelock `getMinDelay()`, and multisig threshold/owners on-chain.
- Cross-check every configured external address against the official deployment list for that
  chain ID. Do **not** call untrusted endpoints (see `repo-execution-safety.md`).

## Reporting

- Put concrete live-config findings (e.g. "ProxyAdmin owned by deployer EOA `0x…`, no timelock")
  in the deployment section, severity-scored like any other finding — instant-upgrade-to-malicious
  is typically High/Critical operational risk.
- Where you only have source, list each item as a **launch checklist** the team must satisfy, and
  record the relied-upon configuration in the **Assumption Ledger** (e.g. "assumes admin is a
  ≥48h timelock" → what breaks if false: instant rug/brick).
