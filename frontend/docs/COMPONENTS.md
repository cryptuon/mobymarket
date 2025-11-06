# 🧩 Component Library Documentation

## Overview

This document provides comprehensive documentation for all components in the Moby Market frontend application. Components are organized into logical groups and follow consistent patterns for maintainability and reusability.

## 📋 Table of Contents

- [📊 Analytics Components](#-analytics-components)
  - [Charts](#charts)
  - [Cards](#cards)
  - [Feeds](#feeds)
- [💼 Portfolio Components](#-portfolio-components)
- [📱 Trading Components](#-trading-components)
- [🏗️ Base Components](#️-base-components)
- [🎨 UI Components](#-ui-components)
- [🔄 Layout Components](#-layout-components)

---

## 📊 Analytics Components

### Charts

Analytics charts provide visual representations of market data, portfolio performance, and trading metrics.

#### PortfolioPerformanceChart

**Purpose**: Displays portfolio performance over time with multiple timeframes and comparison metrics.

**Location**: `src/components/analytics/charts/PortfolioPerformanceChart.vue`

**Props**:
```typescript
interface Props {
  portfolioId?: string
  timeframe?: '1D' | '1W' | '1M' | '3M' | '1Y' | 'ALL'
  showComparison?: boolean
  height?: number
}
```

**Usage Example**:
```vue
<template>
  <PortfolioPerformanceChart
    :portfolio-id="selectedPortfolioId"
    timeframe="1M"
    :show-comparison="true"
    :height="400"
  />
</template>
```

**Features**:
- Interactive time series chart with zoom and pan
- Performance comparison with benchmarks
- Profit/Loss visualization with color coding
- Real-time updates via WebSocket
- Responsive design for mobile and desktop

#### AssetAllocationChart

**Purpose**: Visualizes portfolio asset allocation with interactive pie and treemap charts.

**Location**: `src/components/analytics/charts/AssetAllocationChart.vue`

**Props**:
```typescript
interface Props {
  portfolioId?: string
  chartType?: 'pie' | 'treemap' | 'sunburst'
  showPercentages?: boolean
  interactive?: boolean
}
```

**Usage Example**:
```vue
<template>
  <AssetAllocationChart
    :portfolio-id="currentPortfolio.id"
    chart-type="sunburst"
    :show-percentages="true"
    :interactive="true"
  />
</template>
```

#### MarketComparisonChart

**Purpose**: Compares portfolio performance against market benchmarks and indices.

**Location**: `src/components/analytics/charts/MarketComparisonChart.vue`

**Features**:
- S&P 500, Bitcoin, Ethereum benchmark comparisons
- Correlation analysis and risk-adjusted returns
- Market beta and alpha calculations
- Historical performance overlays

#### TradingVolumeChart

**Purpose**: Displays trading volume patterns and trends over time.

**Location**: `src/components/analytics/charts/TradingVolumeChart.vue`

**Features**:
- Volume bars with price overlay
- Moving averages and trend indicators
- Buy/sell volume breakdown
- Whale activity highlights

#### VolatilityAnalysisChart

**Purpose**: Analyzes and visualizes asset volatility patterns.

**Location**: `src/components/analytics/charts/VolatilityAnalysisChart.vue`

**Features**:
- Historical volatility trends
- Implied volatility curves
- Risk metrics visualization
- Volatility surface for options

### Cards

Analytics cards provide quick insights and key metrics in a compact format.

#### PnLBreakdownCard

**Purpose**: Shows detailed profit and loss breakdown with categorization.

**Location**: `src/components/analytics/cards/PnLBreakdownCard.vue`

**Props**:
```typescript
interface Props {
  timeframe?: string
  showDetails?: boolean
  portfolioId?: string
}
```

**Usage Example**:
```vue
<template>
  <PnLBreakdownCard
    timeframe="1M"
    :show-details="true"
    :portfolio-id="activePortfolio"
  />
</template>
```

#### TopPerformersCard

**Purpose**: Displays top performing assets with performance metrics.

**Location**: `src/components/analytics/cards/TopPerformersCard.vue`

**Features**:
- Top gainers and losers
- Performance percentages
- Asset thumbnails and symbols
- Quick action buttons

#### RiskMetricsCard

**Purpose**: Shows key risk metrics and exposure analysis.

**Location**: `src/components/analytics/cards/RiskMetricsCard.vue`

**Metrics Displayed**:
- Value at Risk (VaR)
- Sharpe Ratio
- Maximum Drawdown
- Beta coefficient
- Standard deviation

#### MarketSentimentCard

**Purpose**: Displays market sentiment indicators and social metrics.

**Location**: `src/components/analytics/cards/MarketSentimentCard.vue`

**Features**:
- Fear & Greed Index
- Social sentiment analysis
- News sentiment scoring
- Community metrics

#### WhaleActivityCard

**Purpose**: Shows large transaction alerts and whale movements.

**Location**: `src/components/analytics/cards/WhaleActivityCard.vue`

**Features**:
- Real-time whale alerts
- Transaction value thresholds
- Whale wallet tracking
- Market impact analysis

#### LiquidityPoolCard

**Purpose**: Displays DeFi liquidity pool information and yields.

**Location**: `src/components/analytics/cards/LiquidityPoolCard.vue**

**Features**:
- Pool TVL and volume
- Yield farming opportunities
- Impermanent loss calculations
- Pool composition analysis

### Feeds

#### RecentActivityFeed

**Purpose**: Shows chronological list of recent portfolio activities and transactions.

**Location**: `src/components/analytics/feeds/RecentActivityFeed.vue`

**Props**:
```typescript
interface Props {
  limit?: number
  showFilters?: boolean
  portfolioId?: string
  activityTypes?: ActivityType[]
}
```

**Features**:
- Real-time activity updates
- Transaction filtering and search
- Activity type categorization
- Pagination and infinite scroll

---

## 💼 Portfolio Components

Portfolio components handle portfolio management, position tracking, and rebalancing workflows.

#### PortfolioOverview

**Purpose**: Main portfolio dashboard with summary metrics and quick actions.

**Location**: `src/components/portfolio/PortfolioOverview.vue`

**Props**:
```typescript
interface Props {
  portfolioId: string
  showActions?: boolean
  compact?: boolean
}
```

**Usage Example**:
```vue
<template>
  <PortfolioOverview
    :portfolio-id="selectedPortfolio.id"
    :show-actions="true"
    :compact="false"
  />
</template>
```

**Features**:
- Total portfolio value and PnL
- Asset allocation overview
- Performance metrics
- Quick action buttons
- Real-time price updates

#### PositionManager

**Purpose**: Detailed position management with editing and analysis capabilities.

**Location**: `src/components/portfolio/PositionManager.vue**

**Features**:
- Position list with sorting and filtering
- Position details and analytics
- Buy/sell/close position actions
- Stop loss and take profit settings
- Position sizing calculator

#### RebalanceWizard

**Purpose**: Guided portfolio rebalancing with optimization suggestions.

**Location**: `src/components/portfolio/RebalanceWizard.vue`

**Features**:
- Target allocation setting
- Rebalancing cost analysis
- Tax impact calculations
- Automated rebalancing schedules
- Risk assessment and warnings

---

## 📱 Trading Components

Trading components provide interfaces for order management, strategy execution, and market analysis.

### Desktop Trading Interface

**Location**: `src/components/trading/desktop/`

- **OrderBook.vue**: Real-time order book with depth visualization
- **TradingChart.vue**: Advanced trading charts with indicators
- **OrderForm.vue**: Order entry form with validation
- **PositionPanel.vue**: Open positions and orders management
- **MarketDepth.vue**: Market depth and liquidity analysis

### Mobile Trading Interface

**Location**: `src/components/trading/mobile/`

- **MobileOrderBook.vue**: Optimized order book for mobile
- **MobileTradingChart.vue**: Touch-optimized trading charts
- **MobileOrderForm.vue**: Mobile-friendly order entry
- **QuickTradePanel.vue**: One-tap trading for mobile users

---

## 🏗️ Base Components

Base components provide fundamental UI elements used throughout the application.

#### Card

**Purpose**: Flexible card container with glass morphism styling.

**Location**: `src/components/base/Card.vue`

**Props**:
```typescript
interface Props {
  variant?: 'default' | 'glass' | 'solid' | 'outline'
  padding?: 'sm' | 'md' | 'lg' | 'xl'
  interactive?: boolean
  loading?: boolean
}
```

**Usage Example**:
```vue
<template>
  <Card variant="glass" padding="lg" :interactive="true">
    <template #header>
      <h3>Card Title</h3>
    </template>
    <p>Card content goes here</p>
    <template #footer>
      <button>Action</button>
    </template>
  </Card>
</template>
```

#### Modal

**Purpose**: Accessible modal dialogs with backdrop and focus management.

**Location**: `src/components/base/Modal.vue`

**Props**:
```typescript
interface Props {
  modelValue: boolean
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  persistent?: boolean
  showClose?: boolean
  title?: string
}
```

#### Button

**Purpose**: Consistent button component with multiple variants and states.

**Location**: `src/components/base/Button.vue`

**Props**:
```typescript
interface Props {
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'ghost'
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  loading?: boolean
  disabled?: boolean
  icon?: string
  iconPosition?: 'left' | 'right'
}
```

---

## 🎨 UI Components

UI components provide specialized interface elements and interactive widgets.

#### LoadingSpinner

**Purpose**: Consistent loading indicators with multiple styles.

**Location**: `src/components/ui/LoadingSpinner.vue`

**Props**:
```typescript
interface Props {
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  variant?: 'spinner' | 'dots' | 'pulse' | 'bars'
  color?: string
  text?: string
}
```

#### Toast

**Purpose**: Toast notifications for user feedback and alerts.

**Location**: `src/components/ui/Toast.vue`

**Usage with Composable**:
```typescript
import { useToast } from '@/composables/useToast'

const { showToast } = useToast()

// Success toast
showToast('Portfolio updated successfully!', 'success')

// Error toast
showToast('Failed to execute trade', 'error', { duration: 5000 })
```

#### DataTable

**Purpose**: Feature-rich data table with sorting, filtering, and pagination.

**Location**: `src/components/ui/DataTable.vue`

**Props**:
```typescript
interface Props {
  columns: TableColumn[]
  data: any[]
  sortable?: boolean
  filterable?: boolean
  paginated?: boolean
  selectable?: boolean
  loading?: boolean
}
```

---

## 🔄 Layout Components

Layout components provide application structure and navigation.

#### AppHeader

**Purpose**: Main application header with navigation and user menu.

**Location**: `src/components/layout/AppHeader.vue`

**Features**:
- Logo and brand identity
- Main navigation menu
- User account dropdown
- Notifications bell
- Theme switcher

#### AppSidebar

**Purpose**: Collapsible sidebar navigation for desktop.

**Location**: `src/components/layout/AppSidebar.vue`

**Features**:
- Hierarchical navigation menu
- Active route highlighting
- Collapse/expand functionality
- Search within navigation
- Quick access shortcuts

#### AppFooter

**Purpose**: Application footer with links and information.

**Location**: `src/components/layout/AppFooter.vue`

**Features**:
- Company information and links
- Social media links
- Legal and privacy links
- Version information
- Status indicators

---

## 🚀 Component Development Guidelines

### Naming Conventions

1. **PascalCase** for component names: `PortfolioOverview.vue`
2. **Descriptive names** that indicate purpose: `TradingVolumeChart.vue`
3. **Consistent suffixes** for component types:
   - Charts: `*Chart.vue`
   - Cards: `*Card.vue`
   - Forms: `*Form.vue`
   - Modals: `*Modal.vue`

### Component Structure

```vue
<template>
  <!-- Component template -->
</template>

<script setup lang="ts">
// Imports
// Props interface
// Composables
// Reactive state
// Computed properties
// Methods
// Lifecycle hooks
</script>

<style scoped>
/* Component-specific styles */
</style>
```

### Props Best Practices

1. **Define TypeScript interfaces** for all props
2. **Provide default values** where appropriate
3. **Use descriptive prop names** that indicate their purpose
4. **Validate props** when necessary
5. **Keep props minimal** and focused

### Emitting Events

```typescript
// Define emits
const emit = defineEmits<{
  'update:modelValue': [value: any]
  'change': [event: ChangeEvent]
  'submit': [data: FormData]
}>()

// Emit events
emit('update:modelValue', newValue)
emit('change', changeEvent)
emit('submit', formData)
```

### Composables Integration

```typescript
// Use composables for shared logic
const { portfolio, loading, error } = usePortfolio(portfolioId)
const { formatCurrency, formatPercentage } = useFormatting()
const { showToast } = useToast()
```

### Styling Guidelines

1. **Use TailwindCSS** utility classes for styling
2. **Follow glass morphism** design system
3. **Ensure responsive design** for all screen sizes
4. **Use CSS variables** for theme consistency
5. **Minimize custom CSS** in favor of utilities

### Testing Components

```typescript
// Component test structure
import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'
import PortfolioOverview from '@/components/portfolio/PortfolioOverview.vue'

describe('PortfolioOverview', () => {
  it('renders portfolio data correctly', () => {
    const wrapper = mount(PortfolioOverview, {
      props: {
        portfolioId: 'test-portfolio-id'
      }
    })

    expect(wrapper.find('[data-testid="portfolio-value"]').exists()).toBe(true)
  })
})
```

### Documentation Requirements

1. **JSDoc comments** for all public methods
2. **Type definitions** for all props and emits
3. **Usage examples** in component documentation
4. **Storybook stories** for visual testing
5. **Test coverage** for critical functionality

---

## 📚 Additional Resources

- [Vue.js 3 Documentation](https://vuejs.org/)
- [TypeScript Guide](https://www.typescriptlang.org/docs/)
- [TailwindCSS Documentation](https://tailwindcss.com/docs)
- [Pinia State Management](https://pinia.vuejs.org/)
- [Vitest Testing Framework](https://vitest.dev/)
- [Storybook for Vue](https://storybook.js.org/docs/vue/get-started/introduction)

For questions or contributions, please refer to our [Contributing Guidelines](./CONTRIBUTING.md).