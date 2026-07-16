# Moby Market Roadmap

> **Where this is going, and what has to be true before real institutional and
> tokenized-RWA flow can settle on it.**

This document is the strategic complement to the low-level phasing in
[`documentation/docs/implementation-roadmap.md`](./documentation/docs/implementation-roadmap.md)
and [`documentation/docs/implementation-order.md`](./documentation/docs/implementation-order.md).
It states the vision, the milestones, and — most importantly — the
**cheapest path to production**: the shortest honest route to settling real
size on Solana mainnet-beta.

Moby Market is **pre-mainnet** and under active development. Nothing below is a
promise of a date; it is a statement of sequencing and of the conditions that
gate each step.

---

## Vision

The two structural shifts reshaping on-chain finance in 2026 — **real-world-asset
(RWA) tokenization** and **institutional DeFi adoption** — both depend on an
execution layer that does not yet exist in open source. Issuing a tokenized
treasury bill, a private-credit tranche, or a corporate balance-sheet position
is now a solved problem. *Moving size in and out of those positions* — without
market impact, without information leakage, and without surrendering custody to
a centralized OTC desk — is not.

Moby Market's north star is to be **the execution leg for institutional DeFi and
tokenized RWAs on Solana**: the intent-native, privacy-preserving, MEV-resistant
infrastructure that tokenized assets and institutional capital need to trade at
size. Concretely, that means:

- **No market impact** — OTC block clearing and TWAP/VWAP shredding keep large
  orders off the public price.
- **No information leakage** — zero-knowledge privacy pools, Pedersen-committed
  amounts, and stealth addresses keep strategy confidential.
- **Provable compliance** — selective disclosure lets an issuer or fund prove
  accreditation, jurisdiction, and audit trail without doxxing the trade.
- **Intent-native execution** — a trader posts *what* they want; a competitive
  solver network figures out *how*, in line with the 2026 shift toward
  intent-based, atomically-composable execution.
- **Open and self-hostable** — MIT-licensed Rust; no closed matcher in front of
  a public protocol.

---

## Milestones

The milestones below are ordered by dependency, not by calendar. Each builds on
the crates that exist today (`moby-types`, `moby-math`, `moby-oracle`,
`moby-trading`, `moby-privacy`, and the supporting `moby-bridge`, `moby-dex`,
`moby-governance` workspace members).

### M1 — Core OTC + execution (foundation)
- Trustless on-chain `OTCEscrow` with partial fills, RFQ, and dark-pool mode.
- TWAP execution (`TWAPOrder`) with anti-detection timing randomness.
- Smart routing across Serum, Raydium, Orca, Phoenix, Lifinity.
- Oracle aggregation (Pyth / Switchboard / Chainlink) with staleness checks.
- **Gate:** deterministic, overflow-checked release builds; devnet coverage of
  the full OTC + TWAP path.

### M2 — Execution-algo completeness
- VWAP execution (`VWAPOrder`) with adaptive market-volume participation.
- Multi-hop and split routing hardened against liquidity shifts mid-flight.
- **Gate:** simulated large-order execution showing modeled slippage within the
  design target (`< 0.1%` on whale-sized orders) under realistic liquidity.

### M3 — Privacy layer
- ZK privacy pools in `moby-privacy` (Groth16 / PLONK / STARKs / Bulletproofs).
- Stealth addresses on both sides; confidential-transfer plumbing.
- Minimum anonymity set and configurable withdrawal delay to defeat timing
  analysis.
- **Gate:** independent ZK-circuit review; proof-generation and on-chain
  verification within Solana compute-budget limits (see below).

### M4 — Intent-native execution + solver network
- `TradingIntent` intake with constraint expression.
- Commit-reveal solver auction with on-chain reputation and MEV protection.
- **Gate:** solver-reliability testing (liveness, correctness, griefing
  resistance) under adversarial conditions.

### M5 — RWA & institutional rails
- Selective-disclosure compliance module: prove accreditation, jurisdiction, and
  KYC status without revealing identity or strategy.
- Optional KYC/travel-rule hooks for regulated issuers and funds.
- Reporting surfaces that satisfy a fund's own audit obligations while keeping
  positions private from competitors.
- **Gate:** compliance-controls review with a regulated design partner; audit
  trail validated end-to-end.

### M6 — Cross-chain settlement
- `CrossChainOrder` via Wormhole / LayerZero graduated out of `moby-trading`
  into a dedicated crate.
- Atomic settlement across Solana, Ethereum, Arbitrum, and Base.
- **Gate:** bridge-failure and timeout handling proven; no fund-loss path on a
  stalled or reverted leg.

---

## Cheapest path to production

**Deployment target: Solana mainnet-beta.** Solana is where the execution layer
belongs — the throughput and sub-second latency profile is what institutional
execution actually needs, the DEX and oracle integrations Moby Market depends on
are native there, and mainnet-beta is the cheapest venue to prove real
settlement without standing up a new chain. Cross-chain reach (M6) is an
*extension* of a working Solana core, not a prerequisite for production.

The cheapest honest path to production is: **ship the smallest surface that a
real desk would actually route size through, on Solana mainnet-beta, and prove
each production-viability gate below before opening it to real flow.** That
smallest surface is M1 + M3's privacy primitives + M5's selective disclosure —
OTC/TWAP execution that is private and compliant. Everything else (VWAP, full
intent solver network, cross-chain) can follow once real flow is settling.

Production-viability requirements — each is a hard gate, not a nice-to-have:

1. **Security audit.** Independent smart-contract audits of every on-chain
   program before any real flow. Economic-security review of escrow, settlement,
   and fee paths. This is the single largest cost and the longest lead time on
   the critical path — schedule it early.
2. **Liquidity & venue partnerships.** Execution quality is only as good as the
   liquidity reachable. Secure OTC counterparties/market makers and confirm the
   DEX venue integrations (Serum, Raydium, Orca, Phoenix, Lifinity) behave under
   real, adversarial mainnet conditions. Without depth, the slippage targets are
   theoretical.
3. **ZK-privacy proof performance.** Prove that privacy is affordable on-chain:
   proof generation is fast enough for a trading workflow, and on-chain
   verification fits inside Solana's per-transaction compute budget
   (`MAX_COMPUTE_UNITS = 1_400_000`). If verification does not fit the budget,
   privacy is a demo, not a product.
4. **Intent-solver reliability.** The solver network must be live, correct, and
   griefing-resistant before intents are the default path. Commit-reveal and
   on-chain reputation must demonstrably reward correct, low-leakage fills and
   punish misbehavior — under load and under adversarial solvers.
5. **Compliance controls.** Selective-disclosure and optional KYC/travel-rule
   hooks must satisfy a real regulated design partner. For institutional and RWA
   flow this is a gating requirement, not a later feature — no compliance story,
   no institutional counterparties.
6. **Monitoring & incident response.** Production-grade observability
   (execution-latency percentiles, realized slippage, solver success rate,
   privacy-pool entropy, cross-chain settlement times), alerting, and the
   emergency-pause / circuit-breaker path exercised in drills before mainnet
   flow. Multi-sig governance over privileged actions is a precondition, not an
   afterthought.

**Sequencing the spend:** audit and compliance have the longest lead times and
gate everything downstream — start them the moment M1 + M3 primitives are
feature-stable. Liquidity partnerships run in parallel because they are
relationship-bound, not code-bound. Proof-performance and solver-reliability
work is engineering the team controls directly and can compress. Monitoring is
cheap to build early and expensive to retrofit — build it alongside M1.

---

## Status

See the active-development notice at the top of the
[README](./README.md). The Rust workspace is buildable; on-chain programs and
supporting crates are evolving; APIs may change between releases. Issues and PRs
are welcome, and this roadmap is expected to change as the gates above are met.
