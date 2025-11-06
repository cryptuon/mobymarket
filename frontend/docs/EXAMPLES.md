# 🚀 Code Examples and Usage Patterns

## Overview

This document provides practical examples and usage patterns for implementing common features in the Moby Market frontend application.

## 📋 Table of Contents

- [🏗️ Component Composition Patterns](#️-component-composition-patterns)
- [📊 Analytics Dashboard Examples](#-analytics-dashboard-examples)
- [💼 Portfolio Management Examples](#-portfolio-management-examples)
- [📱 Trading Interface Examples](#-trading-interface-examples)
- [🔄 State Management Examples](#-state-management-examples)
- [🎨 Styling and Theming Examples](#-styling-and-theming-examples)
- [⚡ Performance Optimization Examples](#-performance-optimization-examples)

---

## 🏗️ Component Composition Patterns

### Creating a Dashboard Layout

```vue
<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-900 to-slate-800">
    <!-- Header -->
    <AppHeader />

    <!-- Main Content -->
    <div class="flex">
      <!-- Sidebar -->
      <AppSidebar :collapsed="sidebarCollapsed" />

      <!-- Dashboard Grid -->
      <main class="flex-1 p-6">
        <div class="grid grid-cols-12 gap-6">
          <!-- Portfolio Overview - Spans full width on mobile, 8 cols on desktop -->
          <div class="col-span-12 lg:col-span-8">
            <PortfolioOverview
              :portfolio-id="selectedPortfolioId"
              :show-actions="true"
            />
          </div>

          <!-- Quick Stats Sidebar -->
          <div class="col-span-12 lg:col-span-4 space-y-6">
            <MarketSentimentCard />
            <WhaleActivityCard />
            <TopPerformersCard />
          </div>

          <!-- Charts Row -->
          <div class="col-span-12 lg:col-span-8">
            <PortfolioPerformanceChart
              :portfolio-id="selectedPortfolioId"
              timeframe="1M"
              :height="400"
            />
          </div>

          <div class="col-span-12 lg:col-span-4">
            <AssetAllocationChart
              :portfolio-id="selectedPortfolioId"
              chart-type="pie"
              :interactive="true"
            />
          </div>

          <!-- Activity Feed - Full Width -->
          <div class="col-span-12">
            <RecentActivityFeed
              :limit="10"
              :show-filters="true"
              :portfolio-id="selectedPortfolioId"
            />
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePortfolioStore } from '@/stores/portfolio'

// State
const sidebarCollapsed = ref(false)

// Store
const portfolioStore = usePortfolioStore()

// Computed
const selectedPortfolioId = computed(() => portfolioStore.selectedPortfolio?.id)
</script>
```

### Conditional Component Rendering

```vue
<template>
  <div class="trading-interface">
    <!-- Desktop Layout -->
    <div v-if="!isMobile" class="hidden lg:block">
      <DesktopTradingInterface
        :selected-pair="selectedTradingPair"
        :order-book="orderBookData"
        :chart-data="chartData"
      />
    </div>

    <!-- Mobile Layout -->
    <div v-else class="lg:hidden">
      <MobileTradingInterface
        :selected-pair="selectedTradingPair"
        :simplified-view="true"
      />
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center">
      <LoadingSpinner size="lg" text="Loading trading data..." />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBreakpoints } from '@/composables/useBreakpoints'
import { useTradingStore } from '@/stores/trading'

// Composables
const { isMobile } = useBreakpoints()
const tradingStore = useTradingStore()

// Computed
const selectedTradingPair = computed(() => tradingStore.selectedPair)
const orderBookData = computed(() => tradingStore.orderBook)
const chartData = computed(() => tradingStore.chartData)
const isLoading = computed(() => tradingStore.isLoading)
</script>
```

---

## 📊 Analytics Dashboard Examples

### Real-time Portfolio Analytics

```vue
<template>
  <div class="analytics-dashboard space-y-6">
    <!-- Performance Overview -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <MetricCard
        title="Total Value"
        :value="formatCurrency(totalValue)"
        :change="totalChange"
        icon="CurrencyDollarIcon"
        :loading="loading"
      />

      <MetricCard
        title="24h P&L"
        :value="formatCurrency(dailyPnL)"
        :change="dailyChange"
        icon="TrendingUpIcon"
        :loading="loading"
      />

      <MetricCard
        title="Total Return"
        :value="formatPercentage(totalReturn)"
        :change="returnChange"
        icon="ChartBarIcon"
        :loading="loading"
      />

      <MetricCard
        title="Sharpe Ratio"
        :value="sharpeRatio.toFixed(2)"
        icon="CalculatorIcon"
        :loading="loading"
      />
    </div>

    <!-- Charts Section -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <Card variant="glass">
        <template #header>
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-semibold text-white">Performance Chart</h3>
            <TimeframePicker v-model="selectedTimeframe" />
          </div>
        </template>

        <PortfolioPerformanceChart
          :portfolio-id="portfolioId"
          :timeframe="selectedTimeframe"
          :show-comparison="true"
          :height="300"
        />
      </Card>

      <Card variant="glass">
        <template #header>
          <h3 class="text-lg font-semibold text-white">Asset Allocation</h3>
        </template>

        <AssetAllocationChart
          :portfolio-id="portfolioId"
          chart-type="sunburst"
          :interactive="true"
        />
      </Card>
    </div>

    <!-- Risk Analysis -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <RiskMetricsCard :portfolio-id="portfolioId" />
      <VolatilityAnalysisChart :assets="portfolioAssets" />
      <MarketComparisonChart :portfolio-id="portfolioId" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { usePortfolio } from '@/composables/usePortfolio'
import { useFormatting } from '@/composables/useFormatting'
import { useWebSocket } from '@/composables/useWebSocket'

// Props
interface Props {
  portfolioId: string
}
const props = defineProps<Props>()

// State
const selectedTimeframe = ref<string>('1M')

// Composables
const { portfolio, loading, fetchPortfolio } = usePortfolio(props.portfolioId)
const { formatCurrency, formatPercentage } = useFormatting()

// WebSocket for real-time updates
const { connect, disconnect } = useWebSocket(`/portfolio/${props.portfolioId}`, {
  onMessage: (data) => {
    // Update portfolio data in real-time
    portfolio.value = { ...portfolio.value, ...data }
  }
})

// Computed values
const totalValue = computed(() => portfolio.value?.totalValue || 0)
const dailyPnL = computed(() => portfolio.value?.dailyPnL || 0)
const totalReturn = computed(() => portfolio.value?.totalReturn || 0)
const sharpeRatio = computed(() => portfolio.value?.sharpeRatio || 0)
const portfolioAssets = computed(() => portfolio.value?.positions || [])

const totalChange = computed(() => ({
  value: totalValue.value - (portfolio.value?.previousValue || 0),
  percentage: ((totalValue.value / (portfolio.value?.previousValue || 1)) - 1) * 100
}))

const dailyChange = computed(() => ({
  value: dailyPnL.value,
  percentage: (dailyPnL.value / (totalValue.value || 1)) * 100
}))

// Lifecycle
onMounted(() => {
  fetchPortfolio()
  connect()
})

onUnmounted(() => {
  disconnect()
})
</script>
```

### Custom Chart with Interactive Features

```vue
<template>
  <Card variant="glass" class="min-h-[400px]">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ChartLineIcon" class="w-5 h-5 text-blue-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Custom Trading Chart</h3>
            <p class="text-xs text-white/60">{{ selectedPair.toUpperCase() }}</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Timeframe Selector -->
          <div class="flex rounded-lg bg-white/10 p-1">
            <button
              v-for="timeframe in timeframes"
              :key="timeframe"
              @click="selectedTimeframe = timeframe"
              :class="[
                'px-3 py-1 text-xs font-medium rounded transition-colors',
                selectedTimeframe === timeframe
                  ? 'bg-blue-500 text-white'
                  : 'text-white/70 hover:text-white hover:bg-white/10'
              ]"
            >
              {{ timeframe }}
            </button>
          </div>

          <!-- Chart Type Selector -->
          <Select v-model="chartType" :options="chartTypes" />
        </div>
      </div>
    </template>

    <!-- Chart Container -->
    <div ref="chartContainer" class="w-full h-80 relative">
      <canvas
        ref="chartCanvas"
        @mousemove="handleMouseMove"
        @click="handleChartClick"
        class="absolute inset-0 w-full h-full"
      />

      <!-- Crosshair -->
      <div
        v-if="crosshair.visible"
        class="absolute pointer-events-none"
        :style="{
          left: crosshair.x + 'px',
          top: crosshair.y + 'px'
        }"
      >
        <div class="w-px h-full bg-white/30 -translate-x-px"></div>
        <div class="h-px w-full bg-white/30 -translate-y-px"></div>
      </div>

      <!-- Price Tooltip -->
      <div
        v-if="tooltip.visible"
        class="absolute pointer-events-none bg-black/80 text-white text-xs rounded px-2 py-1 z-10"
        :style="{
          left: tooltip.x + 'px',
          top: tooltip.y + 'px'
        }"
      >
        <div>Price: {{ formatCurrency(tooltip.price) }}</div>
        <div>Time: {{ formatTime(tooltip.time) }}</div>
        <div>Volume: {{ formatVolume(tooltip.volume) }}</div>
      </div>
    </div>

    <!-- Chart Controls -->
    <template #footer>
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <label class="flex items-center space-x-2 text-sm text-white/70">
            <input
              v-model="showVolume"
              type="checkbox"
              class="rounded border-white/20 bg-white/10"
            />
            <span>Show Volume</span>
          </label>

          <label class="flex items-center space-x-2 text-sm text-white/70">
            <input
              v-model="showIndicators"
              type="checkbox"
              class="rounded border-white/20 bg-white/10"
            />
            <span>Technical Indicators</span>
          </label>
        </div>

        <div class="flex items-center space-x-2">
          <Button variant="ghost" size="sm" @click="resetZoom">
            Reset Zoom
          </Button>
          <Button variant="ghost" size="sm" @click="exportChart">
            Export
          </Button>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useChart } from '@/composables/useChart'
import { useFormatting } from '@/composables/useFormatting'

// Props
interface Props {
  selectedPair: string
  data: ChartDataPoint[]
}
const props = defineProps<Props>()

// Chart configuration
const timeframes = ['1m', '5m', '15m', '1h', '4h', '1d']
const chartTypes = [
  { value: 'candlestick', label: 'Candlestick' },
  { value: 'line', label: 'Line' },
  { value: 'area', label: 'Area' }
]

// State
const selectedTimeframe = ref('1h')
const chartType = ref('candlestick')
const showVolume = ref(true)
const showIndicators = ref(false)

const chartContainer = ref<HTMLDivElement>()
const chartCanvas = ref<HTMLCanvasElement>()

const crosshair = ref({
  visible: false,
  x: 0,
  y: 0
})

const tooltip = ref({
  visible: false,
  x: 0,
  y: 0,
  price: 0,
  time: 0,
  volume: 0
})

// Composables
const { formatCurrency, formatTime, formatVolume } = useFormatting()
const { initChart, updateChart, destroyChart } = useChart()

// Event Handlers
const handleMouseMove = (event: MouseEvent) => {
  const rect = chartCanvas.value?.getBoundingClientRect()
  if (!rect) return

  crosshair.value = {
    visible: true,
    x: event.clientX - rect.left,
    y: event.clientY - rect.top
  }

  // Update tooltip with price data at cursor position
  const dataPoint = getDataPointAtPosition(crosshair.value.x, crosshair.value.y)
  if (dataPoint) {
    tooltip.value = {
      visible: true,
      x: crosshair.value.x + 10,
      y: crosshair.value.y - 10,
      price: dataPoint.close,
      time: dataPoint.timestamp,
      volume: dataPoint.volume
    }
  }
}

const handleChartClick = (event: MouseEvent) => {
  // Handle chart interactions (zoom, annotations, etc.)
}

const resetZoom = () => {
  // Reset chart zoom to default
}

const exportChart = () => {
  // Export chart as image
}

// Lifecycle
onMounted(() => {
  if (chartCanvas.value) {
    initChart(chartCanvas.value, {
      type: chartType.value,
      data: props.data,
      showVolume: showVolume.value,
      showIndicators: showIndicators.value
    })
  }
})

onUnmounted(() => {
  destroyChart()
})

// Watchers
watch([chartType, showVolume, showIndicators], () => {
  updateChart({
    type: chartType.value,
    showVolume: showVolume.value,
    showIndicators: showIndicators.value
  })
})

watch(() => props.data, (newData) => {
  updateChart({ data: newData })
})
</script>
```

---

## 💼 Portfolio Management Examples

### Portfolio Creation Wizard

```vue
<template>
  <Modal
    v-model="isOpen"
    title="Create New Portfolio"
    size="lg"
    :persistent="isProcessing"
  >
    <form @submit.prevent="handleSubmit" class="space-y-6">
      <!-- Step Indicator -->
      <div class="flex items-center justify-between mb-8">
        <div
          v-for="(step, index) in steps"
          :key="step.id"
          :class="[
            'flex items-center',
            index < steps.length - 1 ? 'flex-1' : ''
          ]"
        >
          <div
            :class="[
              'w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium',
              currentStep >= index + 1
                ? 'bg-blue-500 text-white'
                : 'bg-white/10 text-white/50'
            ]"
          >
            {{ index + 1 }}
          </div>
          <span class="ml-3 text-sm text-white/70">{{ step.title }}</span>

          <div
            v-if="index < steps.length - 1"
            :class="[
              'flex-1 h-px mx-4',
              currentStep > index + 1
                ? 'bg-blue-500'
                : 'bg-white/20'
            ]"
          />
        </div>
      </div>

      <!-- Step 1: Basic Information -->
      <div v-show="currentStep === 1" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-white/80 mb-2">
            Portfolio Name
          </label>
          <input
            v-model="formData.name"
            type="text"
            required
            placeholder="My Crypto Portfolio"
            class="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-blue-400"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-white/80 mb-2">
            Description (Optional)
          </label>
          <textarea
            v-model="formData.description"
            rows="3"
            placeholder="Describe your investment strategy..."
            class="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-blue-400"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-white/80 mb-2">
            Initial Investment
          </label>
          <div class="relative">
            <input
              v-model.number="formData.initialAmount"
              type="number"
              step="0.01"
              min="0"
              required
              placeholder="0.00"
              class="w-full pl-8 pr-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-blue-400"
            />
            <span class="absolute left-3 top-2.5 text-white/60">$</span>
          </div>
        </div>
      </div>

      <!-- Step 2: Investment Strategy -->
      <div v-show="currentStep === 2" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-white/80 mb-2">
            Investment Strategy
          </label>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <button
              v-for="strategy in investmentStrategies"
              :key="strategy.id"
              type="button"
              @click="formData.strategy = strategy.id"
              :class="[
                'p-4 rounded-lg border-2 text-left transition-colors',
                formData.strategy === strategy.id
                  ? 'border-blue-400 bg-blue-400/10'
                  : 'border-white/20 bg-white/5 hover:border-white/30'
              ]"
            >
              <div class="flex items-center space-x-3">
                <HeroIcon :name="strategy.icon" class="w-6 h-6 text-blue-400" />
                <div>
                  <h4 class="font-medium text-white">{{ strategy.name }}</h4>
                  <p class="text-xs text-white/60">{{ strategy.description }}</p>
                </div>
              </div>
            </button>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-white/80 mb-2">
            Risk Tolerance
          </label>
          <div class="flex space-x-4">
            <label
              v-for="risk in riskLevels"
              :key="risk.value"
              class="flex-1 cursor-pointer"
            >
              <input
                v-model="formData.riskTolerance"
                type="radio"
                :value="risk.value"
                class="sr-only"
              />
              <div
                :class="[
                  'p-3 rounded-lg border text-center transition-colors',
                  formData.riskTolerance === risk.value
                    ? 'border-blue-400 bg-blue-400/10'
                    : 'border-white/20 bg-white/5 hover:border-white/30'
                ]"
              >
                <div :class="`text-${risk.color}-400`">{{ risk.icon }}</div>
                <div class="text-sm font-medium text-white">{{ risk.label }}</div>
              </div>
            </label>
          </div>
        </div>
      </div>

      <!-- Step 3: Asset Allocation -->
      <div v-show="currentStep === 3" class="space-y-4">
        <div class="flex items-center justify-between">
          <h4 class="text-lg font-medium text-white">Asset Allocation</h4>
          <Button
            variant="ghost"
            size="sm"
            @click="suggestAllocation"
            :loading="loadingSuggestion"
          >
            Suggest Allocation
          </Button>
        </div>

        <div class="space-y-3">
          <div
            v-for="(allocation, index) in formData.allocations"
            :key="index"
            class="flex items-center space-x-4"
          >
            <Select
              v-model="allocation.asset"
              :options="availableAssets"
              placeholder="Select asset"
              class="flex-1"
            />

            <div class="w-32">
              <input
                v-model.number="allocation.percentage"
                type="number"
                min="0"
                max="100"
                step="0.1"
                placeholder="0"
                class="w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-white text-center"
              />
            </div>

            <Button
              variant="ghost"
              size="sm"
              @click="removeAllocation(index)"
              :disabled="formData.allocations.length <= 1"
            >
              Remove
            </Button>
          </div>
        </div>

        <div class="flex items-center justify-between">
          <Button
            variant="ghost"
            @click="addAllocation"
            :disabled="formData.allocations.length >= 10"
          >
            Add Asset
          </Button>

          <div class="text-sm text-white/70">
            Total: {{ totalAllocation }}%
            <span
              v-if="totalAllocation !== 100"
              class="text-yellow-400 ml-2"
            >
              (Must equal 100%)
            </span>
          </div>
        </div>

        <!-- Allocation Preview Chart -->
        <div class="mt-6">
          <AssetAllocationChart
            :data="allocationPreview"
            chart-type="pie"
            :height="200"
          />
        </div>
      </div>

      <!-- Step 4: Review -->
      <div v-show="currentStep === 4" class="space-y-6">
        <h4 class="text-lg font-medium text-white">Review Portfolio</h4>

        <div class="bg-white/5 rounded-lg p-4 space-y-3">
          <div class="flex justify-between">
            <span class="text-white/70">Name:</span>
            <span class="text-white">{{ formData.name }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/70">Initial Investment:</span>
            <span class="text-white">{{ formatCurrency(formData.initialAmount) }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/70">Strategy:</span>
            <span class="text-white">{{ getStrategyName(formData.strategy) }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-white/70">Risk Tolerance:</span>
            <span class="text-white">{{ getRiskLabel(formData.riskTolerance) }}</span>
          </div>
        </div>

        <div class="bg-white/5 rounded-lg p-4">
          <h5 class="text-sm font-medium text-white/80 mb-3">Asset Allocation</h5>
          <div class="space-y-2">
            <div
              v-for="allocation in formData.allocations"
              :key="allocation.asset"
              class="flex justify-between text-sm"
            >
              <span class="text-white/70">{{ getAssetName(allocation.asset) }}:</span>
              <span class="text-white">{{ allocation.percentage }}%</span>
            </div>
          </div>
        </div>
      </div>
    </form>

    <!-- Modal Footer -->
    <template #footer>
      <div class="flex justify-between">
        <Button
          v-if="currentStep > 1"
          variant="ghost"
          @click="currentStep--"
          :disabled="isProcessing"
        >
          Previous
        </Button>

        <div class="flex space-x-3 ml-auto">
          <Button
            variant="ghost"
            @click="isOpen = false"
            :disabled="isProcessing"
          >
            Cancel
          </Button>

          <Button
            v-if="currentStep < 4"
            variant="primary"
            @click="nextStep"
            :disabled="!canProceed"
          >
            Next
          </Button>

          <Button
            v-else
            variant="primary"
            type="submit"
            @click="handleSubmit"
            :loading="isProcessing"
            :disabled="!isFormValid"
          >
            Create Portfolio
          </Button>
        </div>
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePortfolioStore } from '@/stores/portfolio'
import { useToast } from '@/composables/useToast'

// Define component interface
interface Props {
  modelValue: boolean
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'created', portfolio: Portfolio): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Local state
const isOpen = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const currentStep = ref(1)
const isProcessing = ref(false)
const loadingSuggestion = ref(false)

// Form data
const formData = ref({
  name: '',
  description: '',
  initialAmount: 0,
  strategy: '',
  riskTolerance: '',
  allocations: [
    { asset: '', percentage: 0 }
  ]
})

// Configuration
const steps = [
  { id: 'basic', title: 'Basic Info' },
  { id: 'strategy', title: 'Strategy' },
  { id: 'allocation', title: 'Allocation' },
  { id: 'review', title: 'Review' }
]

const investmentStrategies = [
  {
    id: 'hodl',
    name: 'HODL',
    description: 'Buy and hold long-term',
    icon: 'LockClosedIcon'
  },
  {
    id: 'dca',
    name: 'Dollar Cost Averaging',
    description: 'Regular automated investments',
    icon: 'ClockIcon'
  },
  {
    id: 'active',
    name: 'Active Trading',
    description: 'Frequent buying and selling',
    icon: 'BoltIcon'
  },
  {
    id: 'yield',
    name: 'Yield Farming',
    description: 'DeFi liquidity provision',
    icon: 'CurrencyDollarIcon'
  }
]

// Rest of the component logic...
</script>
```

---

## 📱 Trading Interface Examples

### Real-time Order Book

```vue
<template>
  <Card variant="glass" class="h-96 flex flex-col">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <h3 class="text-sm font-medium text-white">Order Book</h3>
        <div class="flex items-center space-x-2 text-xs text-white/60">
          <span>{{ selectedPair.toUpperCase() }}</span>
          <div class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></div>
        </div>
      </div>
    </template>

    <!-- Order Book Content -->
    <div class="flex-1 overflow-hidden">
      <!-- Asks (Sell Orders) -->
      <div class="h-1/2 overflow-y-auto">
        <div class="sticky top-0 bg-slate-900/80 backdrop-blur-sm px-4 py-1 text-xs text-white/60 border-b border-white/10">
          <div class="grid grid-cols-3 gap-4">
            <span>Price (USDT)</span>
            <span class="text-right">Size (BTC)</span>
            <span class="text-right">Total</span>
          </div>
        </div>

        <div class="space-y-px">
          <div
            v-for="(order, index) in sortedAsks"
            :key="`ask-${index}`"
            @click="selectPrice(order.price)"
            :class="[
              'relative px-4 py-1 text-xs cursor-pointer hover:bg-white/5 transition-colors',
              order.isNew ? 'animate-pulse' : ''
            ]"
          >
            <!-- Background bar showing depth -->
            <div
              class="absolute inset-0 bg-red-500/10"
              :style="{ width: `${(order.total / maxAskTotal) * 100}%` }"
            />

            <div class="relative grid grid-cols-3 gap-4">
              <span class="text-red-400 font-mono">{{ formatPrice(order.price) }}</span>
              <span class="text-right text-white/80 font-mono">{{ formatSize(order.size) }}</span>
              <span class="text-right text-white/60 font-mono">{{ formatSize(order.total) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Spread -->
      <div class="px-4 py-2 bg-white/5 border-y border-white/10">
        <div class="flex items-center justify-between text-xs">
          <span class="text-white/60">Spread</span>
          <span class="text-white font-mono">{{ formatPrice(spread) }}</span>
          <span class="text-white/60">({{ formatPercentage(spreadPercentage) }})</span>
        </div>
      </div>

      <!-- Bids (Buy Orders) -->
      <div class="h-1/2 overflow-y-auto">
        <div class="space-y-px">
          <div
            v-for="(order, index) in sortedBids"
            :key="`bid-${index}`"
            @click="selectPrice(order.price)"
            :class="[
              'relative px-4 py-1 text-xs cursor-pointer hover:bg-white/5 transition-colors',
              order.isNew ? 'animate-pulse' : ''
            ]"
          >
            <!-- Background bar showing depth -->
            <div
              class="absolute inset-0 bg-green-500/10"
              :style="{ width: `${(order.total / maxBidTotal) * 100}%` }"
            />

            <div class="relative grid grid-cols-3 gap-4">
              <span class="text-green-400 font-mono">{{ formatPrice(order.price) }}</span>
              <span class="text-right text-white/80 font-mono">{{ formatSize(order.size) }}</span>
              <span class="text-right text-white/60 font-mono">{{ formatSize(order.total) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useWebSocket } from '@/composables/useWebSocket'
import { useFormatting } from '@/composables/useFormatting'

// Props
interface Props {
  selectedPair: string
}
const props = defineProps<Props>()

// Emits
interface Emits {
  (e: 'priceSelected', price: number): void
}
const emit = defineEmits<Emits>()

// State
const orderBook = ref<{
  bids: OrderBookEntry[]
  asks: OrderBookEntry[]
}>({
  bids: [],
  asks: []
})

// Composables
const { formatPrice, formatSize, formatPercentage } = useFormatting()

// WebSocket connection
const { connect, disconnect } = useWebSocket(`/orderbook/${props.selectedPair}`, {
  onMessage: (data) => {
    updateOrderBook(data)
  }
})

// Computed
const sortedBids = computed(() =>
  orderBook.value.bids
    .sort((a, b) => b.price - a.price)
    .slice(0, 20)
    .map((bid, index, array) => ({
      ...bid,
      total: array.slice(0, index + 1).reduce((sum, item) => sum + item.size, 0)
    }))
)

const sortedAsks = computed(() =>
  orderBook.value.asks
    .sort((a, b) => a.price - b.price)
    .slice(0, 20)
    .reverse()
    .map((ask, index, array) => ({
      ...ask,
      total: array.slice(index).reduce((sum, item) => sum + item.size, 0)
    }))
)

const maxBidTotal = computed(() =>
  Math.max(...sortedBids.value.map(bid => bid.total))
)

const maxAskTotal = computed(() =>
  Math.max(...sortedAsks.value.map(ask => ask.total))
)

const spread = computed(() => {
  const lowestAsk = Math.min(...orderBook.value.asks.map(a => a.price))
  const highestBid = Math.max(...orderBook.value.bids.map(b => b.price))
  return lowestAsk - highestBid
})

const spreadPercentage = computed(() => {
  const midPrice = (
    Math.min(...orderBook.value.asks.map(a => a.price)) +
    Math.max(...orderBook.value.bids.map(b => b.price))
  ) / 2
  return (spread.value / midPrice) * 100
})

// Methods
const updateOrderBook = (data: any) => {
  // Mark new orders for animation
  const newOrders = new Set()

  // Update bids
  if (data.bids) {
    data.bids.forEach((bid: any) => {
      const existingIndex = orderBook.value.bids.findIndex(b => b.price === bid.price)
      if (existingIndex >= 0) {
        if (bid.size === 0) {
          orderBook.value.bids.splice(existingIndex, 1)
        } else {
          orderBook.value.bids[existingIndex] = { ...bid, isNew: true }
          newOrders.add(`bid-${bid.price}`)
        }
      } else if (bid.size > 0) {
        orderBook.value.bids.push({ ...bid, isNew: true })
        newOrders.add(`bid-${bid.price}`)
      }
    })
  }

  // Update asks
  if (data.asks) {
    data.asks.forEach((ask: any) => {
      const existingIndex = orderBook.value.asks.findIndex(a => a.price === ask.price)
      if (existingIndex >= 0) {
        if (ask.size === 0) {
          orderBook.value.asks.splice(existingIndex, 1)
        } else {
          orderBook.value.asks[existingIndex] = { ...ask, isNew: true }
          newOrders.add(`ask-${ask.price}`)
        }
      } else if (ask.size > 0) {
        orderBook.value.asks.push({ ...ask, isNew: true })
        newOrders.add(`ask-${ask.price}`)
      }
    })
  }

  // Clear new order flags after animation
  setTimeout(() => {
    orderBook.value.bids.forEach(bid => bid.isNew = false)
    orderBook.value.asks.forEach(ask => ask.isNew = false)
  }, 1000)
}

const selectPrice = (price: number) => {
  emit('priceSelected', price)
}

// Lifecycle
onMounted(() => {
  connect()
})

onUnmounted(() => {
  disconnect()
})
</script>
```

---

## 🔄 State Management Examples

### Pinia Store with Persistence

```typescript
// stores/portfolio.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Portfolio, Position, Transaction } from '@/types'
import { portfolioService } from '@/services/api/portfolio'

export const usePortfolioStore = defineStore('portfolio', () => {
  // State
  const portfolios = ref<Portfolio[]>([])
  const selectedPortfolio = ref<Portfolio | null>(null)
  const positions = ref<Position[]>([])
  const recentTransactions = ref<Transaction[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const totalValue = computed(() => {
    return positions.value.reduce((total, position) => {
      return total + (position.quantity * position.currentPrice)
    }, 0)
  })

  const totalPnL = computed(() => {
    return positions.value.reduce((total, position) => {
      const costBasis = position.quantity * position.averagePrice
      const currentValue = position.quantity * position.currentPrice
      return total + (currentValue - costBasis)
    }, 0)
  })

  const totalPnLPercentage = computed(() => {
    const totalCostBasis = positions.value.reduce((total, position) => {
      return total + (position.quantity * position.averagePrice)
    }, 0)

    return totalCostBasis > 0 ? (totalPnL.value / totalCostBasis) * 100 : 0
  })

  const topPerformers = computed(() => {
    return positions.value
      .map(position => ({
        ...position,
        pnlPercentage: ((position.currentPrice - position.averagePrice) / position.averagePrice) * 100
      }))
      .sort((a, b) => b.pnlPercentage - a.pnlPercentage)
      .slice(0, 5)
  })

  const assetAllocation = computed(() => {
    const total = totalValue.value
    return positions.value.map(position => ({
      symbol: position.symbol,
      value: position.quantity * position.currentPrice,
      percentage: total > 0 ? ((position.quantity * position.currentPrice) / total) * 100 : 0,
      color: getAssetColor(position.symbol)
    }))
  })

  // Actions
  const fetchPortfolios = async () => {
    try {
      loading.value = true
      error.value = null
      portfolios.value = await portfolioService.getPortfolios()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch portfolios'
      throw err
    } finally {
      loading.value = false
    }
  }

  const createPortfolio = async (portfolioData: Omit<Portfolio, 'id' | 'createdAt' | 'updatedAt'>) => {
    try {
      loading.value = true
      error.value = null

      const newPortfolio = await portfolioService.createPortfolio(portfolioData)
      portfolios.value.push(newPortfolio)

      // Auto-select the new portfolio
      selectedPortfolio.value = newPortfolio

      return newPortfolio
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create portfolio'
      throw err
    } finally {
      loading.value = false
    }
  }

  const selectPortfolio = async (portfolioId: string) => {
    try {
      loading.value = true
      error.value = null

      const portfolio = portfolios.value.find(p => p.id === portfolioId)
      if (!portfolio) {
        throw new Error('Portfolio not found')
      }

      selectedPortfolio.value = portfolio

      // Fetch associated data
      await Promise.all([
        fetchPositions(portfolioId),
        fetchRecentTransactions(portfolioId)
      ])
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to select portfolio'
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchPositions = async (portfolioId: string) => {
    positions.value = await portfolioService.getPositions(portfolioId)
  }

  const fetchRecentTransactions = async (portfolioId: string, limit = 50) => {
    recentTransactions.value = await portfolioService.getTransactions(portfolioId, { limit })
  }

  const addPosition = async (portfolioId: string, positionData: Omit<Position, 'id' | 'portfolioId'>) => {
    try {
      const newPosition = await portfolioService.addPosition(portfolioId, positionData)
      positions.value.push(newPosition)
      return newPosition
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to add position'
      throw err
    }
  }

  const updatePosition = async (positionId: string, updates: Partial<Position>) => {
    try {
      const updatedPosition = await portfolioService.updatePosition(positionId, updates)
      const index = positions.value.findIndex(p => p.id === positionId)
      if (index >= 0) {
        positions.value[index] = updatedPosition
      }
      return updatedPosition
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update position'
      throw err
    }
  }

  const removePosition = async (positionId: string) => {
    try {
      await portfolioService.removePosition(positionId)
      const index = positions.value.findIndex(p => p.id === positionId)
      if (index >= 0) {
        positions.value.splice(index, 1)
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to remove position'
      throw err
    }
  }

  const rebalancePortfolio = async (portfolioId: string, targetAllocations: any[]) => {
    try {
      loading.value = true
      const result = await portfolioService.rebalancePortfolio(portfolioId, targetAllocations)

      // Refresh positions and transactions
      await Promise.all([
        fetchPositions(portfolioId),
        fetchRecentTransactions(portfolioId)
      ])

      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to rebalance portfolio'
      throw err
    } finally {
      loading.value = false
    }
  }

  // Utility functions
  const getAssetColor = (symbol: string): string => {
    const colors = {
      'BTC': '#f7931a',
      'ETH': '#627eea',
      'ADA': '#0033ad',
      'DOT': '#e6007a',
      'SOL': '#66d9ef'
    }
    return colors[symbol] || '#8b5cf6'
  }

  const getPortfolioById = (id: string) => {
    return portfolios.value.find(p => p.id === id)
  }

  const getPositionBySymbol = (symbol: string) => {
    return positions.value.find(p => p.symbol === symbol)
  }

  // Reset state
  const $reset = () => {
    portfolios.value = []
    selectedPortfolio.value = null
    positions.value = []
    recentTransactions.value = []
    loading.value = false
    error.value = null
  }

  return {
    // State
    portfolios,
    selectedPortfolio,
    positions,
    recentTransactions,
    loading,
    error,

    // Getters
    totalValue,
    totalPnL,
    totalPnLPercentage,
    topPerformers,
    assetAllocation,

    // Actions
    fetchPortfolios,
    createPortfolio,
    selectPortfolio,
    fetchPositions,
    fetchRecentTransactions,
    addPosition,
    updatePosition,
    removePosition,
    rebalancePortfolio,

    // Utilities
    getPortfolioById,
    getPositionBySymbol,
    $reset
  }
}, {
  // Persistence configuration
  persist: {
    key: 'moby-market-portfolio',
    storage: localStorage,
    paths: ['selectedPortfolio', 'portfolios']
  }
})
```

---

## ⚡ Performance Optimization Examples

### Lazy Loading and Code Splitting

```typescript
// router/index.ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Dashboard',
      component: () => import('@/views/DashboardView.vue'),
      meta: {
        title: 'Dashboard',
        requiresAuth: true
      }
    },
    {
      path: '/portfolio',
      name: 'Portfolio',
      component: () => import('@/views/PortfolioView.vue'),
      meta: {
        title: 'Portfolio Management',
        requiresAuth: true
      },
      children: [
        {
          path: '',
          name: 'PortfolioOverview',
          component: () => import('@/components/portfolio/PortfolioOverview.vue')
        },
        {
          path: 'positions',
          name: 'PositionManager',
          component: () => import('@/components/portfolio/PositionManager.vue')
        },
        {
          path: 'rebalance',
          name: 'RebalanceWizard',
          component: () => import('@/components/portfolio/RebalanceWizard.vue')
        }
      ]
    },
    {
      path: '/trading',
      name: 'Trading',
      // Chunk name for better debugging
      component: () => import(/* webpackChunkName: "trading" */ '@/views/TradingView.vue'),
      meta: {
        title: 'Trading Interface',
        requiresAuth: true
      }
    },
    {
      path: '/analytics',
      name: 'Analytics',
      component: () => import(/* webpackChunkName: "analytics" */ '@/views/AnalyticsView.vue'),
      meta: {
        title: 'Analytics Dashboard',
        requiresAuth: true
      }
    }
  ]
})

export default router
```

### Virtual Scrolling for Large Lists

```vue
<template>
  <div class="virtual-list-container" :style="{ height: containerHeight + 'px' }">
    <!-- Virtual scrollbar track -->
    <div class="virtual-scrollbar" :style="{ height: totalHeight + 'px' }"></div>

    <!-- Visible items -->
    <div
      class="virtual-items"
      :style="{ transform: `translateY(${offsetY}px)` }"
    >
      <div
        v-for="item in visibleItems"
        :key="item.id"
        class="virtual-item"
        :style="{ height: itemHeight + 'px' }"
      >
        <slot :item="item" :index="item.index">
          <!-- Default item rendering -->
          <div class="p-4 border-b border-white/10 text-white">
            {{ item.symbol }} - {{ formatCurrency(item.price) }}
          </div>
        </slot>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

// Props
interface Props {
  items: any[]
  itemHeight?: number
  containerHeight?: number
  buffer?: number
}

const props = withDefaults(defineProps<Props>(), {
  itemHeight: 60,
  containerHeight: 400,
  buffer: 5
})

// State
const scrollTop = ref(0)
const containerRef = ref<HTMLElement>()

// Computed
const totalHeight = computed(() => props.items.length * props.itemHeight)

const visibleStartIndex = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / props.itemHeight) - props.buffer)
)

const visibleEndIndex = computed(() =>
  Math.min(
    props.items.length - 1,
    Math.ceil((scrollTop.value + props.containerHeight) / props.itemHeight) + props.buffer
  )
)

const visibleItems = computed(() => {
  return props.items.slice(visibleStartIndex.value, visibleEndIndex.value + 1).map((item, index) => ({
    ...item,
    index: visibleStartIndex.value + index
  }))
})

const offsetY = computed(() => visibleStartIndex.value * props.itemHeight)

// Event handlers
const handleScroll = (event: Event) => {
  scrollTop.value = (event.target as HTMLElement).scrollTop
}

// Lifecycle
onMounted(() => {
  containerRef.value?.addEventListener('scroll', handleScroll, { passive: true })
})

onUnmounted(() => {
  containerRef.value?.removeEventListener('scroll', handleScroll)
})
</script>

<style scoped>
.virtual-list-container {
  overflow-y: auto;
  position: relative;
}

.virtual-scrollbar {
  position: absolute;
  top: 0;
  left: 0;
  width: 1px;
  pointer-events: none;
  visibility: hidden;
}

.virtual-items {
  position: relative;
}

.virtual-item {
  position: absolute;
  width: 100%;
  box-sizing: border-box;
}
</style>
```

### Memoized Expensive Computations

```vue
<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue'
import { useMemoize } from '@/composables/useMemoize'

// Props
interface Props {
  portfolioData: Portfolio
  marketData: MarketData[]
  timeframe: string
}
const props = defineProps<Props>()

// Memoized expensive calculations
const memoizedPortfolioMetrics = useMemoize(
  (portfolio: Portfolio, market: MarketData[], timeframe: string) => {
    // Expensive calculation that should only run when inputs change
    return {
      sharpeRatio: calculateSharpeRatio(portfolio, market),
      volatility: calculateVolatility(portfolio, timeframe),
      beta: calculateBeta(portfolio, market),
      maxDrawdown: calculateMaxDrawdown(portfolio, timeframe),
      correlations: calculateCorrelations(portfolio, market)
    }
  },
  // Cache key generator
  (portfolio, market, timeframe) =>
    `${portfolio.id}-${market.length}-${timeframe}-${portfolio.updatedAt}`
)

// Computed values using memoized functions
const portfolioMetrics = computed(() =>
  memoizedPortfolioMetrics.value(props.portfolioData, props.marketData, props.timeframe)
)

// Memoized chart data processing
const memoizedChartData = useMemoize(
  (positions: Position[], priceHistory: PriceHistory[]) => {
    return positions.map(position => ({
      ...position,
      historicalValues: priceHistory
        .filter(price => price.symbol === position.symbol)
        .map(price => ({
          timestamp: price.timestamp,
          value: position.quantity * price.price
        }))
    }))
  },
  (positions, priceHistory) =>
    `${positions.map(p => `${p.id}-${p.quantity}`).join(',')}-${priceHistory.length}`
)

const chartData = computed(() =>
  memoizedChartData.value(props.portfolioData.positions, props.marketData.priceHistory)
)

// Helper functions for expensive calculations
function calculateSharpeRatio(portfolio: Portfolio, market: MarketData[]): number {
  // Complex calculation implementation
  const returns = calculateReturns(portfolio)
  const riskFreeRate = getRiskFreeRate(market)
  const excessReturns = returns.map(r => r - riskFreeRate)
  const avgExcessReturn = excessReturns.reduce((sum, r) => sum + r, 0) / excessReturns.length
  const volatility = calculateStandardDeviation(excessReturns)

  return volatility > 0 ? avgExcessReturn / volatility : 0
}

function calculateVolatility(portfolio: Portfolio, timeframe: string): number {
  // Implementation for volatility calculation
  const returns = getPortfolioReturns(portfolio, timeframe)
  return calculateStandardDeviation(returns)
}

function calculateBeta(portfolio: Portfolio, market: MarketData[]): number {
  // Implementation for beta calculation
  const portfolioReturns = getPortfolioReturns(portfolio)
  const marketReturns = getMarketReturns(market)

  const covariance = calculateCovariance(portfolioReturns, marketReturns)
  const marketVariance = calculateVariance(marketReturns)

  return marketVariance > 0 ? covariance / marketVariance : 1
}
</script>
```

### Debounced Search Implementation

```vue
<template>
  <div class="search-container">
    <div class="relative">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search assets, portfolios, transactions..."
        class="w-full pl-10 pr-4 py-3 bg-white/10 border border-white/20 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-blue-400"
        @focus="showResults = true"
        @blur="handleBlur"
      />

      <HeroIcon
        name="MagnifyingGlassIcon"
        class="absolute left-3 top-3.5 w-4 h-4 text-white/40"
      />

      <!-- Loading spinner -->
      <LoadingSpinner
        v-if="isSearching"
        size="sm"
        class="absolute right-3 top-3.5"
      />
    </div>

    <!-- Search results -->
    <div
      v-if="showResults && (searchResults.length > 0 || searchQuery.length > 0)"
      class="absolute top-full left-0 right-0 mt-2 bg-slate-800/90 backdrop-blur-sm border border-white/20 rounded-lg shadow-2xl z-50 max-h-96 overflow-y-auto"
    >
      <!-- No results -->
      <div
        v-if="searchResults.length === 0 && searchQuery.length > 0 && !isSearching"
        class="px-4 py-6 text-center text-white/60"
      >
        <HeroIcon name="ExclamationCircleIcon" class="w-8 h-8 mx-auto mb-2 text-white/40" />
        <p>No results found for "{{ searchQuery }}"</p>
        <p class="text-xs mt-1">Try adjusting your search terms</p>
      </div>

      <!-- Search results -->
      <div v-else class="py-2">
        <!-- Group by category -->
        <div
          v-for="group in groupedResults"
          :key="group.category"
        >
          <div class="px-4 py-2 text-xs font-medium text-white/40 uppercase tracking-wider border-t border-white/10 first:border-t-0">
            {{ group.category }}
          </div>

          <button
            v-for="item in group.items"
            :key="item.id"
            @click="selectResult(item)"
            class="w-full px-4 py-3 text-left hover:bg-white/10 focus:bg-white/10 focus:outline-none transition-colors flex items-center space-x-3"
          >
            <!-- Icon based on type -->
            <div class="flex-shrink-0">
              <img
                v-if="item.type === 'asset' && item.icon"
                :src="item.icon"
                :alt="item.symbol"
                class="w-6 h-6 rounded-full"
              />
              <HeroIcon
                v-else
                :name="getIconForType(item.type)"
                class="w-5 h-5 text-white/60"
              />
            </div>

            <!-- Content -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <span class="text-white font-medium truncate">{{ item.title }}</span>
                <span
                  v-if="item.value"
                  class="text-sm text-white/60 ml-2"
                >
                  {{ item.value }}
                </span>
              </div>
              <p v-if="item.subtitle" class="text-xs text-white/50 truncate">
                {{ item.subtitle }}
              </p>
            </div>

            <!-- Badge -->
            <div
              v-if="item.badge"
              :class="[
                'px-2 py-1 rounded-full text-xs font-medium',
                getBadgeClass(item.badge.type)
              ]"
            >
              {{ item.badge.text }}
            </div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useDebounceFn } from '@/composables/useDebounce'
import { useSearch } from '@/composables/useSearch'

// Search state
const searchQuery = ref('')
const showResults = ref(false)
const isSearching = ref(false)
const searchResults = ref<SearchResult[]>([])

// Composables
const { search } = useSearch()

// Debounced search function
const debouncedSearch = useDebounceFn(async (query: string) => {
  if (query.trim().length < 2) {
    searchResults.value = []
    isSearching.value = false
    return
  }

  try {
    isSearching.value = true
    const results = await search(query)
    searchResults.value = results
  } catch (error) {
    console.error('Search error:', error)
    searchResults.value = []
  } finally {
    isSearching.value = false
  }
}, 300) // 300ms debounce delay

// Watcher for search query
watch(searchQuery, (newQuery) => {
  if (newQuery.trim().length === 0) {
    searchResults.value = []
    isSearching.value = false
    return
  }

  debouncedSearch(newQuery)
})

// Computed
const groupedResults = computed(() => {
  const groups = new Map()

  searchResults.value.forEach(result => {
    if (!groups.has(result.category)) {
      groups.set(result.category, {
        category: result.category,
        items: []
      })
    }
    groups.get(result.category).items.push(result)
  })

  return Array.from(groups.values())
})

// Methods
const selectResult = (item: SearchResult) => {
  showResults.value = false

  // Navigate based on item type
  switch (item.type) {
    case 'asset':
      navigateToAsset(item.id)
      break
    case 'portfolio':
      navigateToPortfolio(item.id)
      break
    case 'transaction':
      navigateToTransaction(item.id)
      break
  }
}

const handleBlur = () => {
  // Delay hiding results to allow click events
  setTimeout(() => {
    showResults.value = false
  }, 200)
}

const getIconForType = (type: string): string => {
  const icons = {
    asset: 'CurrencyDollarIcon',
    portfolio: 'BriefcaseIcon',
    transaction: 'ArrowRightLeftIcon',
    strategy: 'ChartBarIcon'
  }
  return icons[type] || 'DocumentIcon'
}

const getBadgeClass = (type: string): string => {
  const classes = {
    success: 'bg-green-500/20 text-green-400',
    warning: 'bg-yellow-500/20 text-yellow-400',
    error: 'bg-red-500/20 text-red-400',
    info: 'bg-blue-500/20 text-blue-400'
  }
  return classes[type] || classes.info
}
</script>
```

This completes the comprehensive component documentation and examples! The documentation covers all major component patterns, state management, performance optimization, and real-world usage examples for building the Moby Market frontend.