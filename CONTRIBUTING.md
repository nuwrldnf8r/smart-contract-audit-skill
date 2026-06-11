# Contributing

Thanks for your interest in improving the `smart-contract-audit` skill. Contributions that make
the skill catch more real bugs — or catch them with better-calibrated severity — are especially
welcome.

## Ways to contribute

- **New vulnerability classes.** Add entries to the relevant `skills/smart-contract-audit/references/*-vectors.md`.
  Keep the format consistent: what the flaw is, how to spot it, and how to confirm exploitability.
  Tie each to a broken invariant where possible, and cite the OWASP SCWE / Sealevel / CosmWasm
  reference if there is one.
- **Harder eval cases.** Add a planted-bug fixture under `evals/easy/` or `evals/hard/` and record
  its bugs in that set's `ground-truth.json`. The most valuable additions are subtle bugs an
  unguided review misses. Mark fixtures clearly as intentionally vulnerable.
- **Methodology / severity improvements.** Refinements to `references/methodology.md` or
  `references/severity-rubric.md` — especially anything that reduces false positives or sharpens
  exploit-path reasoning.
- **Additional ecosystems.** e.g. Move (Sui/Aptos), ink!/Substrate. Add a `references/<eco>-vectors.md`
  and wire the routing into `SKILL.md`.

## Guidelines

- **Prove exploitability.** A vector or finding should come with a concrete attack path, not just a
  pattern. The skill's whole value is avoiding noise.
- **Keep severities honest.** Follow the Impact × Likelihood rubric; don't default to Critical.
- **Mind the length budget.** `SKILL.md`'s frontmatter `description` must stay under 1024 characters,
  and the body works best under ~500 lines. Push detail into `references/` files (progressive
  disclosure) rather than bloating the entry point.
- **Don't include real malicious code.** Eval fixtures should isolate a vulnerability for testing,
  not provide a working exploit kit.

## Process

1. Open an issue describing the change (or the bug class / eval gap you want to address).
2. Make the change on a branch; if it affects detection, add or update an eval fixture so the
   improvement is measurable.
3. Open a PR. Note what you changed and, for skill-behavior changes, how you validated it (which
   eval cases, before/after).
