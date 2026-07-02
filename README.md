# Moby Market

**[🌐 Site](https://mobymarket.cryptuon.com/) · [📚 Docs](https://docs.cryptuon.com/mobymarket/) · [🔬 Cryptuon Research](https://github.com/cryptuon)**

> **Active development.** MobyMarket is under active development. APIs,
> schemas, and on-chain layouts may change between releases.
> Production use at your own risk. Issues and PRs welcome.

> **Finally, a trading platform built for whales.**

- Marketing site: <https://mobymarket.cryptuon.com/>
- Documentation: <https://docs.cryptuon.com/mobymarket/>
- Source: <https://github.com/cryptuon/mobymarket>

## The Problem

If you're trading millions, DeFi is broken for you:

- **You get front-run** - Every large order risks being sandwiched by MEV bots
- **You move markets** - Large trades cause severe slippage
- **Everyone watches** - Public positions telegraph your strategy
- **No counterparties** - Hard to find institutions to take the other side of size
- **Compliance friction** - Institutional flow needs an audit trail without doxxing the trader
- **Retail-grade tools** - Current DeFi UX feels like a toy compared to TradFi

## The Solution

Moby Market is institutional-grade DeFi infrastructure where you can:

**Trade your real size without moving markets or losing privacy.**

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

The phased rollout is described in detail in
[`documentation/docs/implementation-roadmap.md`](./documentation/docs/implementation-roadmap.md)
and [`documentation/docs/implementation-order.md`](./documentation/docs/implementation-order.md).
Early roadmap, in order:

- **OTC + execution**: OTC marketplace on Solana, TWAP/VWAP execution
- **Privacy**: zero-knowledge privacy pools, stealth trading modes
- **Multi-chain**: Ethereum, Arbitrum, Base support via cross-chain atomic swaps
- **AI-assisted execution**: intent-based trading and execution optimization

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

*Making DeFi safe for whales.*
