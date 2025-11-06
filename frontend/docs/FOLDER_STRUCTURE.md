# 📁 Project Folder Structure

This document provides a comprehensive overview of the Moby Market frontend project structure, explaining the purpose and organization of each directory and file.

## 🏗️ Root Directory Structure

```
moby-market/frontend/
├── docs/                     # 📚 Documentation
├── public/                   # 🌐 Static assets
├── src/                      # 💻 Source code
├── package.json              # 📦 Dependencies and scripts
├── README.md                 # 📖 Project overview
├── tailwind.config.ts        # 🎨 TailwindCSS configuration
├── tsconfig.json             # 🔧 TypeScript configuration
└── vite.config.ts            # ⚡ Vite build configuration
```

## 📚 Documentation (`/docs`)

```
docs/
├── README.md                 # Documentation overview
├── ARCHITECTURE.md           # Technical architecture
├── FOLDER_STRUCTURE.md       # This file
├── SETUP.md                  # Development setup
├── DEPLOYMENT.md             # Deployment guide
├── CONTRIBUTING.md           # Contributing guidelines
├── CODE_STANDARDS.md         # Coding standards
├── PR_GUIDELINES.md          # PR guidelines
├── guides/                   # Development guides
│   ├── COMPONENTS.md         # Component development
│   ├── API.md                # API integration
│   ├── STATE.md              # State management
│   ├── STYLING.md            # Styling guide
│   └── TESTING.md            # Testing guide
├── api/                      # API documentation
│   ├── README.md             # API overview
│   ├── AUTH.md               # Authentication API
│   ├── PORTFOLIO.md          # Portfolio API
│   ├── MARKET.md             # Market data API
│   └── TRADING.md            # Trading API
└── components/               # Component documentation
    ├── UI.md                 # UI components
    ├── ANALYTICS.md          # Analytics components
    ├── PORTFOLIO.md          # Portfolio components
    └── TRADING.md            # Trading components
```

## 💻 Source Code (`/src`)

### Main Application Files

```
src/
├── App.vue                   # 🏠 Root Vue component
├── main.ts                   # 🚀 Application entry point
└── env.d.ts                  # 🌍 Environment type definitions
```

### Assets (`/src/assets`)

```
assets/
├── styles/                   # 🎨 Global styles
│   ├── main.css              # Main CSS entry point
│   ├── components.css        # Component-specific styles
│   └── utilities.css         # Utility classes
├── images/                   # 🖼️ Image assets
├── icons/                    # 🎯 Icon assets
└── fonts/                    # 🔤 Custom fonts
```

### Components (`/src/components`)

```
components/
├── ui/                       # 🧱 Base UI components
│   ├── Button.vue            # Button component
│   ├── Card.vue              # Card container
│   ├── Modal.vue             # Modal dialogs
│   ├── Input.vue             # Form inputs
│   ├── Table.vue             # Data tables
│   ├── Chart.vue             # Chart wrapper
│   ├── Loading.vue           # Loading states
│   ├── HeroIcon.vue          # Heroicons wrapper
│   └── Toggle.vue            # Toggle switch
├── layout/                   # 🏗️ Layout components
│   ├── AppHeader.vue         # Application header
│   ├── AppSidebar.vue        # Navigation sidebar
│   ├── AppFooter.vue         # Application footer
│   └── AppLayout.vue         # Main layout wrapper
├── analytics/                # 📊 Analytics components
│   ├── charts/               # Chart components
│   │   ├── PortfolioPerformanceChart.vue
│   │   ├── AssetAllocationChart.vue
│   │   ├── MarketComparisonChart.vue
│   │   ├── TradingVolumeChart.vue
│   │   └── VolatilityAnalysisChart.vue
│   └── cards/                # Analytics cards
│       ├── PnLBreakdownCard.vue
│       ├── TopPerformersCard.vue
│       ├── RiskMetricsCard.vue
│       ├── RecentActivityFeed.vue
│       ├── MarketSentimentCard.vue
│       ├── WhaleActivityCard.vue
│       └── LiquidityPoolCard.vue
├── portfolio/                # 💼 Portfolio components
│   ├── PortfolioOverview.vue # Portfolio dashboard
│   ├── PositionManager.vue   # Position management
│   └── RebalanceWizard.vue   # Rebalancing tool
├── trading/                  # 📈 Trading components
│   ├── TradingInterface.vue  # Main trading UI
│   ├── OrderBook.vue         # Order book display
│   ├── TradingChart.vue      # Price charts
│   ├── OrderForm.vue         # Order placement
│   └── PositionsList.vue     # Active positions
├── wallet/                   # 🔐 Wallet components
│   ├── WalletConnect.vue     # Wallet connection
│   ├── WalletSelector.vue    # Wallet selection
│   └── WalletBalance.vue     # Balance display
├── whale/                    # 🐋 Whale tracking
│   ├── WhaleTracker.vue      # Whale monitoring
│   ├── WhaleAlerts.vue       # Whale alerts
│   └── WhaleAnalytics.vue    # Whale analytics
└── dashboard/                # 🏠 Dashboard components
    ├── DashboardOverview.vue # Main dashboard
    ├── QuickStats.vue        # Quick statistics
    └── RecentActivity.vue    # Recent activity feed
```

### Composables (`/src/composables`)

```
composables/
├── useAuth.ts                # 🔐 Authentication logic
├── useApi.ts                 # 🌐 API integration
├── useWebSocket.ts           # 📡 WebSocket connections
├── useNotifications.ts       # 🔔 Notification system
├── useTheme.ts               # 🎨 Theme management
├── useLocalStorage.ts        # 💾 Local storage utilities
├── useClipboard.ts           # 📋 Clipboard operations
├── useModal.ts               # 🪟 Modal management
├── useTable.ts               # 📊 Table utilities
└── useChart.ts               # 📈 Chart utilities
```

### Router (`/src/router`)

```
router/
├── index.ts                  # 🗺️ Main router configuration
├── routes.ts                 # 📍 Route definitions
├── guards.ts                 # 🛡️ Navigation guards
└── middleware.ts             # 🔄 Route middleware
```

### Services (`/src/services`)

```
services/
├── api/                      # 🌐 API services
│   ├── index.ts              # API exports
│   ├── base.ts               # Base API client
│   ├── auth.ts               # Authentication API
│   ├── portfolio.ts          # Portfolio API
│   ├── market.ts             # Market data API
│   └── trading.ts            # Trading API
├── websocket/                # 📡 WebSocket services
│   ├── index.ts              # WebSocket exports
│   ├── connection.ts         # Connection management
│   ├── handlers.ts           # Message handlers
│   └── types.ts              # WebSocket types
├── storage/                  # 💾 Storage services
│   ├── localStorage.ts       # Local storage
│   ├── sessionStorage.ts     # Session storage
│   └── indexedDB.ts          # IndexedDB operations
└── utils/                    # 🛠️ Utility services
    ├── formatters.ts         # Data formatters
    ├── validators.ts         # Data validation
    ├── crypto.ts             # Cryptographic utilities
    └── date.ts               # Date utilities
```

### Stores (`/src/stores`)

```
stores/
├── index.ts                  # 🏪 Store exports
├── auth.ts                   # 🔐 Authentication store
├── portfolio.ts              # 💼 Portfolio store
├── market.ts                 # 📊 Market data store
├── trading.ts                # 📈 Trading store
├── notifications.ts          # 🔔 Notifications store
├── theme.ts                  # 🎨 Theme store
├── wallet.ts                 # 🔐 Wallet store
└── ui.ts                     # 🖥️ UI state store
```

### Types (`/src/types`)

```
types/
├── index.ts                  # 📋 Type exports
├── api.ts                    # 🌐 API types
├── auth.ts                   # 🔐 Authentication types
├── portfolio.ts              # 💼 Portfolio types
├── market.ts                 # 📊 Market data types
├── trading.ts                # 📈 Trading types
├── wallet.ts                 # 🔐 Wallet types
├── common.ts                 # 🔄 Common types
└── vue.ts                    # ⚡ Vue-specific types
```

## 🌐 Public Assets (`/public`)

```
public/
├── index.html                # 🏠 HTML template
├── favicon.ico               # 🎯 Site favicon
├── manifest.json             # 📱 PWA manifest
├── robots.txt                # 🤖 SEO robots file
├── tokens/                   # 🪙 Token icons
│   ├── eth.svg
│   ├── btc.svg
│   ├── uni.svg
│   └── ...
└── images/                   # 🖼️ Static images
    ├── logo.svg
    ├── hero-bg.jpg
    └── ...
```

## 📦 Configuration Files

### Package.json
- **Dependencies**: Vue 3, TailwindCSS, TypeScript, Vite
- **Scripts**: Development, build, test, and deployment commands
- **Metadata**: Project information and version

### TailwindCSS Config (`tailwind.config.ts`)
- **Custom colors**: Moby Market brand colors
- **Typography**: Custom font configurations
- **Utilities**: Custom utility classes
- **Plugins**: Additional TailwindCSS plugins

### TypeScript Config (`tsconfig.json`)
- **Compiler options**: Strict type checking
- **Path mapping**: Import path aliases
- **Build targets**: ES2020 compatibility

### Vite Config (`vite.config.ts`)
- **Build optimization**: Production optimizations
- **Development server**: Hot reload configuration
- **Plugin configuration**: Vue, TypeScript plugins
- **Path aliases**: Simplified import paths

## 🗂️ File Naming Conventions

### Components
- **PascalCase**: `ComponentName.vue`
- **Descriptive names**: Clear purpose indication
- **Consistent suffixes**: `.vue` for components

### Composables
- **camelCase with "use" prefix**: `useFeatureName.ts`
- **Single responsibility**: One composable per file
- **Clear naming**: Indicates functionality

### Stores
- **camelCase**: `featureName.ts`
- **Domain-based**: Organized by feature area
- **Consistent patterns**: Standard store structure

### Types
- **camelCase**: `featureName.ts`
- **Interface naming**: PascalCase interfaces
- **Type exports**: Centralized type definitions

### Services
- **camelCase**: `serviceName.ts`
- **API services**: Clear service boundaries
- **Utility functions**: Organized by purpose

## 📋 Import Path Aliases

The project uses path aliases for cleaner imports:

```typescript
// Instead of: import Button from '../../../components/ui/Button.vue'
import Button from '@components/ui/Button.vue'

// Available aliases:
// @/ -> src/
// @components/ -> src/components/
// @composables/ -> src/composables/
// @stores/ -> src/stores/
// @services/ -> src/services/
// @types/ -> src/types/
// @assets/ -> src/assets/
```

## 🧹 Organization Principles

### 1. **Feature-First Organization**
Components are organized by feature/domain rather than technical type.

### 2. **Clear Separation of Concerns**
- **Components**: UI logic only
- **Composables**: Reusable business logic
- **Stores**: Global state management
- **Services**: External integrations

### 3. **Consistent Naming**
All files follow consistent naming conventions for easy navigation.

### 4. **Scalable Structure**
The structure supports easy addition of new features without reorganization.

### 5. **Documentation Co-location**
Related documentation is placed near the relevant code.

## 🔍 Finding Files

### Quick Reference

| Looking for... | Check... |
|----------------|----------|
| UI components | `src/components/ui/` |
| Feature components | `src/components/{feature}/` |
| Business logic | `src/composables/` |
| API calls | `src/services/api/` |
| Global state | `src/stores/` |
| Type definitions | `src/types/` |
| Routing | `src/router/` |
| Styles | `src/assets/styles/` |
| Static assets | `public/` |
| Documentation | `docs/` |

---

This folder structure is designed to be intuitive, scalable, and maintainable as the Moby Market platform grows. Each directory has a clear purpose, and the organization supports both individual feature development and cross-cutting concerns.