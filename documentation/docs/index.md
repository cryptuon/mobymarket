# MobyMarket

> **Finally, a trading platform built for whales.**

MobyMarket is an institutional-grade DeFi trading platform designed for traders moving size that would otherwise move markets. It combines an OTC marketplace, sophisticated execution algorithms (TWAP / VWAP), and zero-knowledge privacy features on top of Solana, with a roadmap toward multi-chain support.

## What problem does it solve?

If you are trading millions on public DeFi venues today, you face a stack of structural problems:

- **MEV / front-running** — every large order risks being sandwiched.
- **Market impact** — large trades cause severe slippage.
- **Strategy leakage** — public order flow telegraphs your positions.
- **Counterparty discovery** — finding institutions to take the other side of size is hard.
- **Compliance** — institutional flow needs an audit trail without doxxing the trader.

MobyMarket addresses each of these as a first-class concern rather than a workaround.

## Core capabilities

### Find real counterparties

- **OTC Marketplace** — connect directly with institutions and market makers.
- **Request-for-Quote (RFQ)** — solicit competitive quotes from multiple counterparties.
- **Dark pools** — trade without revealing size until matched.

See: [OTC Trading Guide](otc-trading.md), [Market Maker Guide](market-maker-guide.md).

### Trade without moving markets

- **TWAP** — spread execution over hours or days with intelligent timing.
- **VWAP** — execute against natural volume profiles.
- **Smart routing** — split across venues automatically.

See: [Execution Algorithms](execution-algorithms.md), [Whale Trading Guide](whale-trading-guide.md).

### Keep your strategy private

- **Zero-knowledge proofs** — trade without revealing amounts or timing.
- **Stealth mode** — order flow that looks like noise to observers.
- **Privacy pools** — mix with other whales for stronger anonymity sets.

See: [Privacy Features](privacy-features.md), [Privacy Trading Guide](privacy-trading.md).

### Stay compliant

- **Selective disclosure** — prove compliance without revealing details.
- **Jurisdiction proofs** — demonstrate accreditation without revealing location.
- **Audit trail** — full compliance reporting when required.

## Who is this for?

- **Traders** — start with the [Whale Trading Guide](whale-trading-guide.md) and the [Privacy Trading Guide](privacy-trading.md).
- **Institutions** — see [Institutional Onboarding](institutional-setup.md) and the [API Reference](api-reference.md).
- **Market makers** — see the [Market Maker Guide](market-maker-guide.md).
- **Developers** — start with the [Developer Guide](developer-guide.md), then the [Technical Architecture](architecture.md).

## How the system is built

MobyMarket is structured as a layered set of Solana programs and supporting libraries:

- **Main controller** — orchestrates operations.
- **OTC settlement** — peer-to-peer trade execution.
- **Execution algorithms** — TWAP / VWAP / intent processing.
- **Privacy program** — ZK-proof verification and confidential transfers.
- **Oracles** — price feeds and market data aggregation.

For implementation detail see the [Technical Architecture](architecture.md) and the [Solana Programs Specification](solana-programs.md).

## Project status

The codebase under this repository is the in-progress implementation of the MobyMarket whale trading infrastructure. The phased rollout (OTC, then privacy, then multi-chain, then AI-assisted execution) is described in the [Implementation Roadmap](implementation-roadmap.md) and the [Implementation Order](implementation-order.md).

## Contact

- Institutions: `institutions@moby-market.com`
- Security: `security@moby-market.com`
- Support: `support@moby-market.com`
