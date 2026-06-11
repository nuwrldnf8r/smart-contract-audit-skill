# CosmWasm / Cosmos Vulnerability Catalogue

CosmWasm contracts are Rust compiled to Wasm, running on Cosmos-SDK chains. Many EVM
intuitions transfer (access control, arithmetic, oracle, logic bugs), but the execution
model is different in ways that create CosmWasm-specific pitfalls. Use this with
`methodology.md`. Most of these are drawn from the patterns Oak Security / jcsec document
in the CosmWasm Security Spotlight and audit checklists.

## Table of contents
1. The CosmWasm execution model (read first)
2. Access control & authorization
3. Address & input validation
4. State / storage handling
5. Arithmetic, rounding, and overflow
6. Message passing, replies, and reentrancy
7. Funds handling
8. Migration / upgrade
9. Oracle, logic & economic
10. Chain-level / module concerns
11. Triage hints

---

## 1. The execution model (read first)
- Contracts expose `instantiate`, `execute`, `query`, and optionally `migrate`, `reply`,
  `sudo`, and IBC entry points. Each `execute` returns a `Response` carrying **messages**
  that are dispatched *after* the current execution completes (actor model).
- Because outbound messages execute later, the classic single-call reentrancy is rarer — but
  **multi-message and reply-based reentrancy** exist and are easy to get wrong.
- `query` is read-only but can be called cross-contract; stale or manipulable query results
  feed logic bugs.
- Gas metering differs between the Wasm VM and the SDK; unmetered logic in
  BeginBlock/EndBlock/hooks can enable DoS.

## 2. Access control & authorization
- **Missing/incorrect auth on `execute` branches.** Every privileged `ExecuteMsg` variant
  must check `info.sender` against stored admin/owner/role. A forgotten check on one match
  arm is the most common CosmWasm high-severity bug.
- **Admin stored vs. enforced.** Confirm the check actually compares to the right stored
  address and rejects otherwise.
- **`sudo` entry points** are privileged (chain-level) — ensure logic there can't be reached
  from `execute`.
- **Migration admin.** Whoever holds the contract's migrate admin can replace the code
  entirely — treat as full control; flag if it's a single key.

## 3. Address & input validation
- **Unvalidated addresses.** Any address from a message must go through
  `deps.api.addr_validate` before storage/use. Storing a raw user string lets attackers
  inject malformed or foreign-prefix addresses (CosmWasm Spotlight #3).
- **Trusting `Addr` from untrusted sources.** `Addr` is only "validated" by convention;
  don't construct it from unchecked input.
- **Numeric/parameter bounds**, denom validation, and message-field validation as in any
  contract.

## 4. State / storage handling
- **Unsaved storage changes.** A frequent bug: mutating a struct read from storage but
  never calling `.save()` (or saving the wrong key), so the change silently doesn't persist
  (CosmWasm Spotlight #1). Verify every state mutation is actually written back.
- **Map key collisions / wrong key**, default-value assumptions on missing keys, and
  unbounded iteration over `Map` (state bloat / gas DoS).
- **`Item`/`Map` load `.may_load` vs `.load`** — unhandled `None` or panics on missing state.

## 5. Arithmetic, rounding, and overflow
- Rust release builds **wrap on overflow by default** unless `overflow-checks = true` or
  checked math is used. Prefer `Uint128`/`checked_*`/`Decimal` and explicit error handling.
- Rounding direction in share/reward/swap math (CosmWasm Spotlight #4) — round in the
  protocol's favor, never the user's; watch first-depositor and empty-pool cases.
- `Decimal` precision and casts between `Uint128`/`u128`/`Uint256`.

## 6. Message passing, replies, and reentrancy
- **SubMsg + `reply` reentrancy.** State assumed unchanged between dispatching a SubMsg and
  its reply can be violated if the called contract calls back. Apply checks-effects-
  interactions: finalize state before emitting messages.
- **Reply handling.** Validate `reply.id`, handle both success and error variants, and don't
  trust reply data blindly. Missing error handling can leave partial state.
- **Message ordering / atomicity.** A returned message that fails reverts the whole tx unless
  wrapped in a `SubMsg` with `reply_on` — understand which failures roll back and which don't.

## 7. Funds handling
- **`info.funds` validation.** Check the attached funds (denom and amount) match expectations;
  don't assume a single coin or correct denom.
- **Bank send correctness**, and ensuring the contract can't be tricked into sending more
  than owed or to an attacker-controlled recipient.

## 8. Migration / upgrade
- **`migrate` entry point auth & logic.** Ensure storage layout compatibility across versions
  and that migration validates the new state. An unguarded or buggy migrate can brick or hijack
  the contract.
- **Version checks** (`cw2::set_contract_version` / `get_contract_version`) to prevent
  downgrade or wrong-contract migration.

## 9. Oracle, logic & economic
- Same economic classes as EVM: oracle manipulation (spot vs TWAP, single source, staleness),
  flash-loan-style amplification where the chain/protocol allows it, business-logic invariant
  breaks, MEV/ordering. Drive these from Phase 1 invariants.

## 10. Chain-level / module concerns (appchain context)
- **Unmetered computation in BeginBlock/EndBlock/hooks** → infinite loop / DoS.
- **Mispriced state operations** enabling cheap state bloat.
- **IBC handlers** (`ibc_packet_receive`, ack, timeout): validate packet source
  channel/port, handle timeouts and acks, and never trust packet contents without checks —
  cross-chain message authenticity is critical.

## 11. Triage hints
- Search every `ExecuteMsg` arm for a matching `info.sender` auth check.
- Grep for `.update(` / `.save(` and confirm each mutated value is persisted.
- Grep for `addr_validate` — its *absence* near address inputs is a flag.
- Grep `SubMsg`, `reply`, `reply_on` → reentrancy/atomicity review.
- Grep `info.funds`, `BankMsg::Send` → funds validation.
- Check `Cargo.toml` for `overflow-checks` and use of `checked_*` math.
- Grep `migrate`, `set_contract_version` → upgrade safety.
