# 🐋 Moby Market Frontend

**Enterprise-Grade Whale Trading Platform** - The future of institutional DeFi trading.

[![Vue.js](https://img.shields.io/badge/Vue.js-3.4-4FC08D?style=flat&logo=vue.js)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat&logo=typescript)](https://www.typescriptlang.org/)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-3.4-06B6D4?style=flat&logo=tailwindcss)](https://tailwindcss.com/)
[![Vite](https://img.shields.io/badge/Vite-5.0-646CFF?style=flat&logo=vite)](https://vitejs.dev/)

## 🎯 **Platform Overview**

Moby Market is a comprehensive whale trading platform that provides institutional-grade tools for tracking, analyzing, and executing large-scale DeFi trades. Built with cutting-edge web technologies and designed for professional traders who move markets.

### **🔥 Key Features**

- **🐋 Whale Intelligence Dashboard** - Real-time whale activity monitoring and analysis
- **📊 Advanced Analytics** - Professional portfolio management and risk assessment
- **⚡ Lightning-Fast Trading** - Optimized swap interface with MEV protection
- **🔗 Multi-Chain Support** - Ethereum, Polygon, Arbitrum, Optimism, Base
- **📱 Mobile-First Design** - Touch-optimized interface for all devices
- **🤖 AI-Powered Insights** - Machine learning recommendations and market analysis

## 🏗️ **Architecture**

### **Technology Stack**

```
Frontend Framework:     Vue.js 3 + Composition API + TypeScript
Styling:               TailwindCSS + SkeletonUI Premium Components
State Management:      Pinia (Vue's official state library)
Build Tool:            Vite (Next-gen build system)
Web3 Integration:      viem + wagmi (Type-safe Ethereum interactions)
Real-time Data:        WebSocket connections with auto-reconnection
UI Components:         100+ custom components with glass morphism design
Testing:               Vitest + Vue Test Utils + Playwright E2E
```

### **Project Structure**

```
frontend/
├── src/
│   ├── components/           # Reusable UI components
│   │   ├── ui/              # Base UI components (Button, Card, etc.)
│   │   ├── trading/         # Trading interface components
│   │   ├── whale/           # Whale intelligence components
│   │   ├── analytics/       # Analytics dashboard components
│   │   └── wallet/          # Wallet connection components
│   ├── stores/              # Pinia state management
│   ├── composables/         # Vue composables for reusable logic
│   ├── services/            # API and external service integrations
│   ├── types/               # TypeScript type definitions
│   ├── utils/               # Utility functions
│   └── assets/              # Static assets and styles
├── public/                  # Public assets
├── docs/                    # Documentation
└── tests/                   # Test files
```

## 🚀 **Quick Start**

### **Prerequisites**

- Node.js 18+
- pnpm (recommended) or npm
- Git

### **Installation**

```bash
# Clone the repository
git clone https://github.com/your-org/moby-market.git
cd moby-market/frontend

# Install dependencies
pnpm install

# Start development server
pnpm dev

# Open http://localhost:5173
```

### **Available Scripts**

```bash
pnpm dev          # Start development server
pnpm build        # Build for production
pnpm preview      # Preview production build
pnpm test         # Run unit tests
pnpm test:e2e     # Run E2E tests
pnpm lint         # Lint code
pnpm typecheck    # Type checking
```

## 🎨 **Component Library**

### **Base UI Components**

| Component | Description | Props | Usage |
|-----------|-------------|-------|-------|
| `Button` | Primary action component | `variant`, `size`, `loading`, `disabled` | Buttons, CTAs, forms |
| `Card` | Container with glass morphism | `variant`, `glow`, `padding` | Content sections |
| `Input` | Form input with validation | `type`, `placeholder`, `error` | Forms, search |
| `Toggle` | Switch component | `modelValue`, `size`, `color` | Settings, filters |
| `Grid` | Responsive grid system | `cols`, `gap`, `responsive` | Layouts |
| `Modal` | Overlay dialog | `show`, `size`, `persistent` | Dialogs, details |

### **Trading Components**

| Component | Description | Features |
|-----------|-------------|----------|
| `SwapInterface` | Desktop trading interface | Real-time quotes, slippage protection, MEV protection |
| `MobileSwapInterface` | Mobile-optimized trading | Bottom sheet UI, touch gestures, simplified flow |
| `LivePriceTicker` | Real-time price feed | Scrolling ticker, price alerts, multi-chain support |
| `WalletConnector` | Multi-wallet integration | MetaMask, WalletConnect, Coinbase Wallet |
| `TransactionStatus` | Trade monitoring | Real-time status, gas tracking, error handling |

### **Whale Intelligence Components**

| Component | Description | Features |
|-----------|-------------|----------|
| `WhaleIntelligenceDashboard` | Main whale tracking hub | Activity feed, metrics, heatmaps |
| `LiveWhaleActivityFeed` | Real-time whale transactions | Filtering, alerts, copy trading prep |
| `WhaleActivityHeatmap` | Time-based activity visualization | 24/7 heatmap, drill-down analysis |
| `TopWhalesCard` | Whale leaderboard | Performance ranking, wallet analysis |
| `TokenFlowAnalysis` | Capital flow visualization | Inflow/outflow tracking, trend analysis |
| `WhaleVolumeChart` | Volume analysis charts | Multi-format charts, zoom, metrics |
| `NetworkDistributionChart` | Cross-chain activity | Donut charts, network comparison |
| `WhaleActivityModal` | Detailed transaction view | Full analysis, market impact, profile |

### **Analytics Components**

| Component | Description | Features |
|-----------|-------------|----------|
| `AnalyticsDashboard` | Main analytics hub | 5 specialized dashboards, AI insights |
| `PortfolioPerformanceChart` | Portfolio tracking | Benchmark comparison, returns analysis |
| `AssetAllocationChart` | Portfolio distribution | Multiple view modes, rebalancing tools |
| `PnLBreakdownCard` | Profit/loss analysis | Category breakdown, performance metrics |
| `TopPerformersCard` | Asset performance ranking | Sortable metrics, detailed analysis |
| `RiskMetricsCard` | Risk assessment | VaR, drawdown, volatility analysis |
| `RecentActivityFeed` | Transaction history | Real-time updates, filtering, export |
| `AIInsightsPanel` | ML-powered recommendations | Confidence scores, risk assessment |

## 📊 **State Management**

### **Pinia Stores**

```typescript
// Core application stores
stores/
├── wallet.ts           # Wallet connection and account state
├── trading.ts          # Trading workflows and order management
├── notifications.ts    # Toast notifications and alerts
├── market.ts          # Market data and price feeds
├── analytics.ts       # Portfolio analytics and performance
└── realtime.ts        # WebSocket connections and live data
```

### **Store Usage Examples**

```typescript
// Trading store usage
import { useTradingStore } from '@/stores/trading'

const trading = useTradingStore()

// Execute a swap
await trading.executeSwap({
  tokenIn: 'ETH',
  tokenOut: 'USDC',
  amount: 10,
  slippage: 0.5
})

// Analytics store usage
import { useAnalyticsStore } from '@/stores/analytics'

const analytics = useAnalyticsStore()

// Get portfolio metrics
const metrics = analytics.portfolioMetrics
const performance = analytics.performanceHistory
```

## 🔗 **Web3 Integration**

### **Supported Wallets**

- **MetaMask** - Browser extension and mobile
- **WalletConnect** - 200+ mobile wallets
- **Coinbase Wallet** - Native integration
- **Brave Wallet** - Built-in browser wallet

### **Supported Networks**

| Network | Chain ID | RPC | Features |
|---------|----------|-----|----------|
| Ethereum | 1 | Infura/Alchemy | Full DEX aggregation |
| Polygon | 137 | Polygon RPC | Low fees, fast finality |
| Arbitrum | 42161 | Arbitrum RPC | L2 scaling, cheap gas |
| Optimism | 10 | Optimism RPC | Optimistic rollups |
| Base | 8453 | Base RPC | Coinbase L2 |

### **Smart Contract Interactions**

```typescript
// Example: Execute a trade
import { useWallet } from '@/composables/useWallet'

const { writeContract } = useWallet()

const swapResult = await writeContract({
  address: UNISWAP_V3_ROUTER,
  abi: UniswapV3RouterABI,
  functionName: 'exactInputSingle',
  args: [swapParams]
})
```

## 📡 **Real-time Data**

### **WebSocket Integration**

```typescript
// Real-time price feeds
import { useRealTimeData } from '@/composables/useRealTimeData'

const {
  livePrices,
  liveWhaleActivity,
  isConnected,
  subscribe,
  unsubscribe
} = useRealTimeData()

// Subscribe to price updates
subscribe('prices', ['ETH/USD', 'BTC/USD'])

// Subscribe to whale activity
subscribe('whale-activity', { minValue: 100000 })
```

### **Data Sources**

- **Price Feeds**: CoinGecko, CoinMarketCap, DEX aggregators
- **Whale Activity**: On-chain analysis, mempool monitoring
- **Market Data**: The Graph, Moralis, custom indexers
- **News & Sentiment**: Multiple news APIs, social sentiment

## 🎨 **Design System**

### **Color Palette**

```css
/* Moby Market Brand Colors */
:root {
  --moby-primary: #0ea5e9;      /* Sky blue */
  --moby-secondary: #8b5cf6;    /* Purple */
  --moby-accent: #06b6d4;       /* Cyan */
  --moby-success: #10b981;      /* Green */
  --moby-warning: #f59e0b;      /* Orange */
  --moby-error: #ef4444;        /* Red */

  /* Glass morphism */
  --glass-bg: rgba(15, 23, 42, 0.8);
  --glass-border: rgba(255, 255, 255, 0.1);
  --glass-blur: blur(20px);
}
```

### **Typography**

```css
/* Font scales */
.text-xs    { font-size: 0.75rem; }    /* 12px */
.text-sm    { font-size: 0.875rem; }   /* 14px */
.text-base  { font-size: 1rem; }       /* 16px */
.text-lg    { font-size: 1.125rem; }   /* 18px */
.text-xl    { font-size: 1.25rem; }    /* 20px */
.text-2xl   { font-size: 1.5rem; }     /* 24px */
.text-3xl   { font-size: 1.875rem; }   /* 30px */
```

### **Component Variants**

```typescript
// Button variants
type ButtonVariant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger'

// Card variants
type CardVariant = 'default' | 'glass' | 'elevated' | 'bordered'

// Size scales
type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl'
```

## 🧪 **Testing Strategy**

### **Unit Tests**

```bash
# Run all unit tests
pnpm test

# Run tests in watch mode
pnpm test:watch

# Generate coverage report
pnpm test:coverage
```

### **E2E Tests**

```bash
# Run E2E tests
pnpm test:e2e

# Run E2E tests in headed mode
pnpm test:e2e:headed

# Run specific test file
pnpm test:e2e tests/trading.spec.ts
```

### **Test Examples**

```typescript
// Component test
import { render, screen } from '@testing-library/vue'
import Button from '@/components/ui/Button.vue'

test('button renders correctly', () => {
  render(Button, {
    props: { variant: 'primary' },
    slots: { default: 'Click me' }
  })

  expect(screen.getByRole('button')).toHaveTextContent('Click me')
})

// Store test
import { setActivePinia, createPinia } from 'pinia'
import { useTradingStore } from '@/stores/trading'

test('trading store executes swap', async () => {
  setActivePinia(createPinia())
  const store = useTradingStore()

  const result = await store.executeSwap({
    tokenIn: 'ETH',
    tokenOut: 'USDC',
    amount: 1
  })

  expect(result.success).toBe(true)
})
```

## 🚀 **Production Deployment**

### **Build Configuration**

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    target: 'esnext',
    outDir: 'dist',
    sourcemap: false,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['vue', 'pinia', 'vue-router'],
          web3: ['viem', 'wagmi', '@wagmi/core'],
          ui: ['@headlessui/vue', '@heroicons/vue']
        }
      }
    }
  },
  define: {
    __VUE_PROD_DEVTOOLS__: false
  }
})
```

### **Environment Variables**

```bash
# .env.production
VITE_APP_ENV=production
VITE_API_BASE_URL=https://api.mobymarket.com
VITE_WS_URL=wss://ws.mobymarket.com
VITE_INFURA_KEY=your_infura_key
VITE_ALCHEMY_KEY=your_alchemy_key
VITE_WALLETCONNECT_PROJECT_ID=your_project_id
```

### **Performance Optimizations**

- **Code Splitting**: Automatic route-based and manual chunk splitting
- **Tree Shaking**: Remove unused code in production builds
- **Asset Optimization**: Image compression, SVG optimization
- **CDN Integration**: Static asset delivery via CDN
- **Caching Strategy**: Service worker for offline functionality
- **Bundle Analysis**: Analyze bundle size and dependencies

## 📈 **Performance Metrics**

### **Core Web Vitals**

- **LCP (Largest Contentful Paint)**: < 2.5s
- **FID (First Input Delay)**: < 100ms
- **CLS (Cumulative Layout Shift)**: < 0.1
- **FCP (First Contentful Paint)**: < 1.8s
- **TTI (Time to Interactive)**: < 3.5s

### **Bundle Size Targets**

- **Initial Bundle**: < 300KB gzipped
- **Vendor Chunks**: < 150KB gzipped
- **Route Chunks**: < 50KB gzipped
- **Total Assets**: < 2MB

## 🔒 **Security**

### **Security Measures**

- **Content Security Policy**: Strict CSP headers
- **HTTPS Everywhere**: Force HTTPS in production
- **Wallet Security**: Secure wallet connections and transaction signing
- **Input Validation**: Client and server-side validation
- **XSS Protection**: Sanitized user inputs and outputs
- **CSRF Protection**: Anti-CSRF tokens for state changes

### **Web3 Security**

```typescript
// Safe contract interactions
import { Address, isAddress } from 'viem'

function validateAddress(address: string): Address {
  if (!isAddress(address)) {
    throw new Error('Invalid Ethereum address')
  }
  return address as Address
}

// Transaction simulation before execution
const simulation = await publicClient.simulateContract({
  address: contractAddress,
  abi: contractABI,
  functionName: 'swap',
  args: [params]
})

if (!simulation.result) {
  throw new Error('Transaction simulation failed')
}
```

## 📚 **API Documentation**

### **REST API Endpoints**

```typescript
// Portfolio API
GET    /api/v1/portfolio              # Get portfolio overview
GET    /api/v1/portfolio/positions    # Get current positions
GET    /api/v1/portfolio/history      # Get transaction history
POST   /api/v1/portfolio/rebalance    # Execute rebalancing

// Analytics API
GET    /api/v1/analytics/performance  # Performance metrics
GET    /api/v1/analytics/risk         # Risk assessment
GET    /api/v1/analytics/attribution  # Return attribution

// Whale Intelligence API
GET    /api/v1/whales/activity        # Recent whale activity
GET    /api/v1/whales/leaderboard     # Top whales by volume
GET    /api/v1/whales/flows           # Token flow analysis
```

### **WebSocket Events**

```typescript
// Price updates
{
  event: 'price_update',
  data: {
    symbol: 'ETH/USD',
    price: 3250.75,
    change24h: 2.3,
    timestamp: '2024-01-15T10:30:00Z'
  }
}

// Whale activity
{
  event: 'whale_activity',
  data: {
    id: 'whale_123',
    type: 'buy',
    token: 'ETH',
    amount: 1000,
    usd_value: 3250000,
    address: '0x1234...5678',
    timestamp: '2024-01-15T10:30:00Z'
  }
}
```

## 🤝 **Contributing**

### **Development Workflow**

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Commit** changes: `git commit -m 'Add amazing feature'`
4. **Push** to branch: `git push origin feature/amazing-feature`
5. **Open** a Pull Request

### **Code Standards**

- **TypeScript**: Strict mode enabled, no `any` types
- **ESLint**: Vue/TypeScript recommended rules
- **Prettier**: Consistent code formatting
- **Conventional Commits**: Semantic commit messages
- **Testing**: Unit tests for all components and stores

### **PR Requirements**

- ✅ All tests passing
- ✅ No TypeScript errors
- ✅ ESLint/Prettier compliance
- ✅ Component documentation
- ✅ Performance impact assessment

## 📝 **Changelog**

### **v2.0.0** (Current)
- ✨ Complete whale intelligence dashboard
- ✨ Advanced analytics with AI insights
- ✨ Multi-chain trading support
- ✨ Mobile-optimized interface
- ✨ Real-time WebSocket integration
- 🎨 Glass morphism design system
- ⚡ Performance optimizations
- 🔒 Enhanced security measures

### **v1.0.0** (Foundation)
- 🎉 Initial platform launch
- 🔗 Basic Web3 integration
- 💱 Simple swap interface
- 📊 Basic portfolio tracking

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 **Acknowledgments**

- **Vue.js Team** - For the amazing framework
- **Tailwind Labs** - For the utility-first CSS framework
- **Ethereum Foundation** - For Web3 standards and tooling
- **DeFi Community** - For inspiration and innovation
- **Open Source Contributors** - For making this possible

---

<div align="center">

**Built with ❤️ by the Moby Market Team**

[Website](https://mobymarket.com) • [Documentation](https://docs.mobymarket.com) • [Discord](https://discord.gg/mobymarket) • [Twitter](https://twitter.com/mobymarket)

</div>