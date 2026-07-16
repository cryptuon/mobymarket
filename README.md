# Moby Market

**[🌐 Site](https://mobymarket.cryptuon.com/) · [📚 Docs](https://docs.cryptuon.com/mobymarket/) · [🔬 Cryptuon Research](https://github.com/cryptuon)**

> **Active development.** MobyMarket is under active development. APIs,
> schemas, and on-chain layouts may change between releases.
> Production use at your own risk. Issues and PRs welcome.

> **The execution leg for institutional DeFi and tokenized RWAs on Solana.**

- Marketing site: <https://mobymarket.cryptuon.com/>
- Documentation: <https://docs.cryptuon.com/mobymarket/>
- Source: <https://github.com/cryptuon/mobymarket>
- Roadmap & production path: [`ROADMAP.md`](./ROADMAP.md)

## Why this matters in 2026

The two biggest structural shifts in on-chain finance — **real-world-asset (RWA)
tokenization** and **institutional DeFi adoption** — both hit the same wall: the
*execution* wall. Tokenizing a treasury, a credit fund, or a corporate balance
sheet is now well-understood. Moving size in and out of those positions without
front-running, market impact, or information leakage is not. Public AMMs
broadcast every order. CEX OTC desks reintroduce counterparty risk and custody
trust. Intent frameworks solve routing but rarely solve privacy or block-sized
OTC clearing.

Moby Market is the missing execution leg. It is open-source Rust infrastructure
that lets institutions and tokenized-RWA issuers trade whale-sized size on
Solana with the three properties that matter to a treasurer, a fund desk, or a
compliance officer:

- **No market impact** — TWAP/VWAP shredding and OTC block clearing keep large
  orders off the public price.
- **No information leakage** — zero-knowledge privacy pools and stealth addresses
  keep strategy confidential while selective disclosure keeps compliance provable.
- **Intent-native** — a trader posts *what* they want; a solver network competes
  on *how* to fill it, aligned with the 2026 move toward intent-based,
  atomically-composable execution.

If tokenized RWAs and institutional capital are the flow, Moby Market is the
plumbing that moves that flow without leaving a wake.

## The Problem

Institutional and RWA-sized trades break every assumption retail DeFi is built on:

- **You get front-run** - Every large order risks being sandwiched by MEV bots
- **You move markets** - Large trades cause severe slippage on public pools
- **Everyone watches** - Public positions telegraph strategy to competitors
- **No counterparties** - Hard to find institutions to take the other side of size
- **Compliance friction** - Institutional and RWA flow needs an audit trail without doxxing the trader
- **Custodial OTC** - CEX OTC desks solve impact but reintroduce counterparty and custody trust
- **Retail-grade tools** - Current DeFi UX feels like a toy compared to TradFi

## The Solution

Moby Market is open-source, institutional-grade execution infrastructure where
institutions and tokenized-RWA issuers can:

**Trade real size without moving markets, without leaking strategy, and without
giving up custody.**

## How We Solve It

### Find Real Counterparties
*"I need to sell $50M SOL but can't find buyers"*

- **OTC Marketplace**: Connect directly with institutions and market makers
- **Request-for-Quote**: Get competitive quotes from multiple counterparties
- **Dark Pools**: Trade without revealing your size until matched
- **One-Click**: Post your order, get quotes, execute — that simple

### Trade Without Moving Markets
*"My $10M trade just moved SOL price 5%"*

- **TWAP**: Spread your order over hours/days with smart timing
- **VWAP**: Execute based on natural market volume patterns
- **Smart Routing**: Split across multiple venues automatically
- **Set & Forget**: "Sell $10M over 4 hours" — done

### Keep Your Strategy Private
*"Everyone front-runs me because they see my trades coming"*

- **Zero-Knowledge Proofs**: Trade without revealing amounts or timing
- **Stealth Mode**: Your trades look like noise to observers
- **Privacy Pools**: Mix with other whales for maximum anonymity
- **Professional Grade**: Privacy that actually works at institutional scale

### Stay Compliant
*"I need regulatory compliance but can't sacrifice privacy"*

- **Selective Disclosure**: Prove compliance without revealing details
- **Jurisdiction Proofs**: Show you're accredited without revealing location
- **Audit Trail**: Full compliance reporting when needed
- **Regulatory Ready**: Designed for institutional compliance requirements

## Illustrative Examples

These are scenarios the platform is designed to support; numbers are
targets the architecture is built around, not measured production results.

### Example 1: Pension Fund Selling $100M SOL
*"We need to liquidate without crashing the price"*

```
1. Click "TWAP Order"
2. Enter: Sell $100M SOL over 48 hours
3. System automatically:
   - Splits into hundreds of randomised chunks
   - Routes across multiple venues
   - Executes when liquidity is deep
4. Goal: sub-1% realised slippage vs double-digit slippage on naive AMM swaps
```

### Example 2: Hedge Fund Private Position Building
*"We're accumulating ETH but can't let competitors know"*

```
1. Enable "Stealth Mode"
2. Set target: Buy $50M ETH over 1 week
3. Orders are:
   - Mixed with other traders
   - Split across multiple addresses
   - Timing randomised to avoid detection
4. Goal: position built with minimal alpha leakage
```

### Example 3: Institution Cross-Chain Rebalancing
*"Move $200M from Ethereum to Solana efficiently"*

```
1. Post RFQ: ETH → SOL, $200M size
2. Get quotes from competing market makers
3. Accept best quote
4. Settlement happens atomically across chains
5. Goal: professional execution with institutional pricing
```

### Example 4: Tokenized-RWA Issuer Rebalancing a Portfolio
*"We hold tokenized T-bills and private credit and need to rotate size on-chain"*

```
1. Post a TradingIntent: rotate $75M from token A into token B
2. System chooses the cheapest path per leg:
   - OTC block clearing where a counterparty exists
   - TWAP/VWAP shredding where it must touch venues
3. Amounts stay Pedersen-committed; a selective-disclosure proof
   satisfies the fund's own compliance and audit obligations
4. Goal: rebalance a tokenized book without printing the trade
   to the rest of the market
```

## Design Targets

These are the design goals of the architecture, not measured production
metrics. The system is pre-mainnet.

### Save on Slippage
- **Typical retail DeFi**: 5–15% slippage on large orders
- **Moby Market target**: <0.1% slippage on whale-sized orders
- **Mechanism**: TWAP/VWAP shredding, smart routing, OTC clearing

### Professional Speed
- **Order Confirmation target**: <400ms
- **Settlement**: minutes vs hours on TradFi
- **Always On**: 24/7/365

### Institutional Security
- **Multi-signature controls** on privileged actions
- **Formal verification** targeted for critical paths
- **Regulatory-ready** compliance modules
- **Emergency Controls**: pause mechanisms for incident response

### Deep Liquidity (design target)
- **Cross-Chain**: access liquidity across major chains
- **OTC Network**: direct access to whale counterparties
- **Aggregated** across Serum, Raydium, Orca, Phoenix, Lifinity, and custom AMMs

## Repository Layout

```
.
├── Cargo.toml          # Rust workspace root
├── libs/               # Rust crates
│   ├── moby-math       # Fixed-point math, Pedersen commitments
│   ├── moby-types      # Shared types (TradingIntent, OTCEscrow, ...)
│   ├── moby-oracle     # Pyth / Switchboard / Chainlink aggregation
│   ├── moby-trading    # TWAP, VWAP, smart routing, cross-chain primitives
│   └── moby-privacy    # Groth16/PLONK/STARK/Bulletproofs, stealth addresses
├── platform/           # Higher-level platform crate + examples
├── frontend/           # Web frontend (separate package)
├── documentation/      # MkDocs Material site (docs.cryptuon.com/mobymarket)
├── docs/               # Long-form design docs (Markdown)
└── scripts/            # build.sh, test.sh
```

## Quickstart

Prerequisites:

- Rust (toolchain pinned via `rust-toolchain.toml`)
- Solana CLI (optional, required for on-chain deploys)
- Anchor CLI 0.29 (optional, required for `anchor build`/`anchor test`)

Clone and build the Rust workspace:

```bash
git clone https://github.com/cryptuon/mobymarket.git
cd mobymarket

# Full build + lint + test
./scripts/build.sh

# Or just the workspace
cargo build --workspace --release

# Run the test suite
./scripts/test.sh
```

Frontend (separate Node package):

```bash
cd frontend
npm install
npm run dev
```

## Documentation

Full documentation lives at <https://docs.cryptuon.com/mobymarket/>.

### For Traders
- **[Whale Trading Guide](./docs/whale-trading-guide.md)** — How to trade your size without moving markets
- **[Finding Counterparties](./docs/otc-trading.md)** — Access the institutional OTC network
- **[Privacy Trading](./docs/privacy-trading.md)** — Keep your strategy confidential

### For Institutions
- **[Institutional Onboarding](./docs/institutional-setup.md)** — White-glove setup process
- **[API Reference](./docs/api-reference.md)** — Connect your existing systems
- **Support**: contact@cryptuon.com

### For Market Makers
- **[Market Maker Guide](./docs/market-maker-guide.md)** — Earn fees providing liquidity
- **[Technical Integration](./docs/developer-guide.md)** — Build your own tools

### Upcoming Docs
The following guides are planned and will land as the corresponding subsystems
mature:

- Compliance Guide (selective disclosure, jurisdiction proofs)
- Risk Management (institutional risk controls)
- RFQ System (deep dive on the request-for-quote flow)

## Roadmap

The vision, milestones, and — importantly — the **cheapest path to production**
live in [`ROADMAP.md`](./ROADMAP.md). Read that first if you want to understand
where this is going and what has to be true before real institutional or RWA
flow can settle on it.

The lower-level phasing is described in detail in
[`documentation/docs/implementation-roadmap.md`](./documentation/docs/implementation-roadmap.md)
and [`documentation/docs/implementation-order.md`](./documentation/docs/implementation-order.md).
Early roadmap, in order:

- **OTC + execution**: OTC marketplace on Solana, TWAP/VWAP execution
- **Privacy**: zero-knowledge privacy pools, stealth trading modes
- **Intent-native execution**: intent-based trading and a competitive solver network
- **RWA & institutional rails**: selective-disclosure compliance controls for tokenized-RWA and institutional flow
- **Multi-chain**: Ethereum, Arbitrum, Base support via cross-chain atomic swaps

Dates are intentionally not pinned in the README; the roadmap docs above
track current sequencing.

## Status

MobyMarket is **pre-mainnet** and under active development. The Rust
workspace is buildable, the on-chain programs and supporting crates are
evolving, and APIs may change. See the active-development notice at the
top of this README.

## Contact

- **Institutions**: contact@cryptuon.com
- **Security**: contact@cryptuon.com
- **Support**: contact@cryptuon.com

## License

MIT — see [LICENSE_HEADER.txt](./LICENSE_HEADER.txt) and per-crate
metadata.

---

## Part of Cryptuon Research

`mobymarket` is one of [20 open-source blockchain-infrastructure projects](https://www.cryptuon.com/projects) from **[Cryptuon Research](https://www.cryptuon.com)** — blockchain theory, shipped as protocols.

**Related projects:** [Mentat](https://mentat.cryptuon.com/) · [PolyBot](https://polybot.cryptuon.com/) · [dgbit](https://dgbit.cryptuon.com/)

Docs: [docs.cryptuon.com/mobymarket](https://docs.cryptuon.com/mobymarket/) · Contact: [contact@cryptuon.com](mailto:contact@cryptuon.com)

---

*The execution leg for institutional DeFi and tokenized RWAs.*
