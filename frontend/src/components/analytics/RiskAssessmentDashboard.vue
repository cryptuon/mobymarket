<template>
  <div class="space-y-6">
    <!-- Risk Score Overview -->
    <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
      <div class="lg:col-span-1">
        <Card variant="glass">
          <div class="p-6 text-center">
            <div class="relative w-32 h-32 mx-auto mb-4">
              <svg class="w-32 h-32 transform -rotate-90" viewBox="0 0 128 128">
                <circle
                  cx="64"
                  cy="64"
                  r="56"
                  fill="none"
                  stroke="rgba(255,255,255,0.1)"
                  stroke-width="8"
                />
                <circle
                  cx="64"
                  cy="64"
                  r="56"
                  fill="none"
                  :stroke="getRiskScoreColor(riskScore)"
                  stroke-width="8"
                  :stroke-dasharray="`${(riskScore / 100) * 352} 352`"
                  stroke-linecap="round"
                  class="transition-all duration-1000"
                />
              </svg>
              <div class="absolute inset-0 flex items-center justify-center">
                <div class="text-center">
                  <div :class="['text-3xl font-bold', getRiskScoreTextColor(riskScore)]">
                    {{ riskScore }}
                  </div>
                  <div class="text-xs text-white/60">Risk Score</div>
                </div>
              </div>
            </div>
            <div class="space-y-2">
              <div :class="['text-lg font-semibold', getRiskScoreTextColor(riskScore)]">
                {{ getRiskLevel(riskScore) }}
              </div>
              <div class="text-sm text-white/60">Portfolio Risk Level</div>
            </div>
          </div>
        </Card>
      </div>

      <div class="lg:col-span-3">
        <Grid :cols="{ xs: 1, md: 3 }" gap="4">
          <GridItem>
            <MetricCard
              title="Value at Risk (95%)"
              :value="formatCurrency(Math.abs(riskMetrics.var95))"
              :change="riskMetrics.var95Change"
              icon="ExclamationTriangleIcon"
              color="red"
              subtitle="Daily VaR"
            />
          </GridItem>
          <GridItem>
            <MetricCard
              title="Max Drawdown"
              :value="`${riskMetrics.maxDrawdown}%`"
              :change="riskMetrics.drawdownChange"
              icon="TrendingDownIcon"
              :color="riskMetrics.maxDrawdown <= 10 ? 'green' : riskMetrics.maxDrawdown <= 20 ? 'yellow' : 'red'"
              subtitle="Peak to trough"
            />
          </GridItem>
          <GridItem>
            <MetricCard
              title="Beta"
              :value="riskMetrics.beta.toFixed(2)"
              :change="riskMetrics.betaChange"
              icon="ScaleIcon"
              :color="riskMetrics.beta <= 1 ? 'green' : riskMetrics.beta <= 1.5 ? 'yellow' : 'red'"
              subtitle="vs Market"
            />
          </GridItem>
        </Grid>
      </div>
    </div>

    <!-- Risk Breakdown Charts -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <RiskBreakdownChart
          :data="riskBreakdownData"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <VolatilityChart
          :data="volatilityHistory"
          :time-range="timeRange"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Risk Factors & Concentration -->
    <Grid :cols="{ xs: 1, lg: 3 }" gap="6">
      <GridItem>
        <ConcentrationRiskCard
          :data="concentrationData"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <LiquidityRiskCard
          :data="liquidityData"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <CorrelationRiskCard
          :data="correlationRiskData"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Stress Testing -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <StressTestingCard
          :scenarios="stressTestScenarios"
          :results="stressTestResults"
          :loading="loading || stressTestLoading"
          @run-test="runStressTest"
        />
      </GridItem>
      <GridItem>
        <MonteCarloSimulation
          :data="monteCarloData"
          :loading="loading || simulationLoading"
          @run-simulation="runMonteCarloSimulation"
        />
      </GridItem>
    </Grid>

    <!-- Risk Monitoring & Alerts -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <RiskAlertsCard
          :alerts="riskAlerts"
          :loading="loading"
          @dismiss-alert="dismissAlert"
          @configure-alert="configureAlert"
        />
      </GridItem>
      <GridItem>
        <RiskLimitsCard
          :limits="riskLimits"
          :current="currentRiskLevels"
          :loading="loading"
          @update-limit="updateRiskLimit"
        />
      </GridItem>
    </Grid>

    <!-- Portfolio Optimization -->
    <PortfolioOptimizationCard
      :current-allocation="currentAllocation"
      :optimal-allocation="optimalAllocation"
      :optimization-metrics="optimizationMetrics"
      :loading="loading || optimizationLoading"
      @run-optimization="runOptimization"
      @apply-optimization="applyOptimization"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Grid from '@components/ui/Grid.vue'
import GridItem from '@components/ui/GridItem.vue'
import Card from '@components/ui/Card.vue'
import MetricCard from '@components/dashboard/MetricCard.vue'
import RiskBreakdownChart from './charts/RiskBreakdownChart.vue'
import VolatilityChart from './charts/VolatilityChart.vue'
import ConcentrationRiskCard from './cards/ConcentrationRiskCard.vue'
import LiquidityRiskCard from './cards/LiquidityRiskCard.vue'
import CorrelationRiskCard from './cards/CorrelationRiskCard.vue'
import StressTestingCard from './cards/StressTestingCard.vue'
import MonteCarloSimulation from './cards/MonteCarloSimulation.vue'
import RiskAlertsCard from './cards/RiskAlertsCard.vue'
import RiskLimitsCard from './cards/RiskLimitsCard.vue'
import PortfolioOptimizationCard from './cards/PortfolioOptimizationCard.vue'

import { useNotificationStore } from '@/stores/notifications'

interface Props {
  data: any
  timeRange: string
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'data-export': [data: any]
  'metric-drill-down': [metric: string, value: any]
}>()

const notificationStore = useNotificationStore()

const stressTestLoading = ref(false)
const simulationLoading = ref(false)
const optimizationLoading = ref(false)

// Risk metrics
const riskScore = ref(72)

const riskMetrics = ref({
  var95: -45000,
  var95Change: -5.2,
  maxDrawdown: 12.5,
  drawdownChange: 2.1,
  beta: 1.35,
  betaChange: 0.08,
  volatility: 28.5,
  sharpeRatio: 1.85
})

// Risk breakdown data
const riskBreakdownData = ref([
  { category: 'Market Risk', value: 65, color: '#ef4444' },
  { category: 'Concentration Risk', value: 45, color: '#f97316' },
  { category: 'Liquidity Risk', value: 25, color: '#eab308' },
  { category: 'Correlation Risk', value: 55, color: '#8b5cf6' },
  { category: 'Operational Risk', value: 15, color: '#06b6d4' }
])

// Volatility history
const volatilityHistory = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    return {
      timestamp: date.toISOString(),
      portfolio: 0.25 + Math.random() * 0.15,
      market: 0.30 + Math.random() * 0.20,
      realized: 0.22 + Math.random() * 0.18,
      implied: 0.28 + Math.random() * 0.25
    }
  })
})

// Concentration data
const concentrationData = ref({
  hhi: 0.28, // Herfindahl-Hirschman Index
  topPositions: [
    { symbol: 'ETH', allocation: 46.1, limit: 40 },
    { symbol: 'BTC', allocation: 28.8, limit: 35 },
    { symbol: 'USDC', allocation: 12.0, limit: 20 }
  ],
  sectors: [
    { name: 'Layer 1', allocation: 68.5, limit: 60 },
    { name: 'DeFi', allocation: 18.2, limit: 30 },
    { name: 'Stablecoins', allocation: 12.0, limit: 25 }
  ]
})

// Liquidity data
const liquidityData = ref({
  liquidityScore: 78,
  avgDailyVolume: 125000000,
  bidAskSpread: 0.035,
  marketDepth: 85,
  illiquidPositions: [
    { symbol: 'RARE_TOKEN', allocation: 2.3, liquidity: 15 },
    { symbol: 'NEW_DEFI', allocation: 1.8, liquidity: 25 }
  ]
})

// Correlation risk data
const correlationRiskData = ref({
  avgCorrelation: 0.67,
  maxCorrelation: 0.92,
  diversificationRatio: 0.78,
  highCorrelationPairs: [
    { pair: 'ETH-UNI', correlation: 0.92 },
    { pair: 'BTC-ETH', correlation: 0.85 },
    { pair: 'AAVE-COMP', correlation: 0.88 }
  ]
})

// Stress test scenarios
const stressTestScenarios = ref([
  { id: '1', name: 'Market Crash (-30%)', severity: 'high' },
  { id: '2', name: 'Bear Market (-50%)', severity: 'extreme' },
  { id: '3', name: 'Flash Crash (-15%)', severity: 'medium' },
  { id: '4', name: 'Correlation Spike', severity: 'high' },
  { id: '5', name: 'Liquidity Crisis', severity: 'extreme' }
])

const stressTestResults = ref([
  { scenario: 'Market Crash (-30%)', portfolioImpact: -285000, recovery: '8 months' },
  { scenario: 'Bear Market (-50%)', portfolioImpact: -485000, recovery: '18 months' },
  { scenario: 'Flash Crash (-15%)', portfolioImpact: -142000, recovery: '2 months' }
])

// Monte Carlo data
const monteCarloData = ref({
  simulations: 10000,
  timeHorizon: 252, // trading days
  confidence: [
    { level: 95, value: -78000 },
    { level: 99, value: -125000 },
    { level: 99.9, value: -180000 }
  ],
  distribution: Array.from({ length: 100 }, (_, i) => ({
    return: -50 + i,
    probability: Math.exp(-Math.pow((i - 50) / 20, 2)) // Normal-ish distribution
  }))
})

// Risk alerts
const riskAlerts = ref([
  {
    id: '1',
    type: 'warning',
    title: 'Concentration Limit Exceeded',
    message: 'ETH allocation (46.1%) exceeds limit of 40%',
    severity: 'medium',
    timestamp: new Date(Date.now() - 1800000).toISOString()
  },
  {
    id: '2',
    type: 'critical',
    title: 'High Correlation Alert',
    message: 'Portfolio correlation increased to 0.92',
    severity: 'high',
    timestamp: new Date(Date.now() - 3600000).toISOString()
  }
])

// Risk limits
const riskLimits = ref({
  maxDrawdown: { limit: 15, current: 12.5 },
  var95: { limit: 50000, current: 45000 },
  beta: { limit: 1.5, current: 1.35 },
  concentration: { limit: 40, current: 46.1 },
  correlation: { limit: 0.8, current: 0.67 }
})

const currentRiskLevels = computed(() => ({
  maxDrawdown: riskMetrics.value.maxDrawdown,
  var95: Math.abs(riskMetrics.value.var95),
  beta: riskMetrics.value.beta,
  concentration: concentrationData.value.topPositions[0].allocation,
  correlation: correlationRiskData.value.avgCorrelation
}))

// Portfolio optimization
const currentAllocation = ref([
  { asset: 'ETH', current: 46.1, value: 461000 },
  { asset: 'BTC', current: 28.8, value: 288000 },
  { asset: 'USDC', current: 12.0, value: 120000 },
  { asset: 'UNI', current: 8.1, value: 81000 },
  { asset: 'AAVE', current: 5.0, value: 50000 }
])

const optimalAllocation = ref([
  { asset: 'ETH', optimal: 35.0, change: -11.1 },
  { asset: 'BTC', optimal: 30.0, change: 1.2 },
  { asset: 'USDC', optimal: 20.0, change: 8.0 },
  { asset: 'UNI', optimal: 10.0, change: 1.9 },
  { asset: 'AAVE', optimal: 5.0, change: 0.0 }
])

const optimizationMetrics = ref({
  expectedReturn: 12.5,
  volatility: 22.3,
  sharpeRatio: 2.15,
  improvement: {
    return: 1.8,
    risk: -18.5,
    sharpe: 0.3
  }
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `$${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `$${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `$${(amount / 1e3).toFixed(2)}K`
  return `$${amount.toFixed(2)}`
}

function getRiskScoreColor(score: number): string {
  if (score <= 30) return '#10b981' // green
  if (score <= 50) return '#f59e0b' // yellow
  if (score <= 70) return '#f97316' // orange
  return '#ef4444' // red
}

function getRiskScoreTextColor(score: number): string {
  if (score <= 30) return 'text-green-400'
  if (score <= 50) return 'text-yellow-400'
  if (score <= 70) return 'text-orange-400'
  return 'text-red-400'
}

function getRiskLevel(score: number): string {
  if (score <= 30) return 'Low Risk'
  if (score <= 50) return 'Moderate Risk'
  if (score <= 70) return 'High Risk'
  return 'Very High Risk'
}

function getTimeRangeDays(range: string): number {
  switch (range) {
    case '24h': return 1
    case '7d': return 7
    case '30d': return 30
    case '90d': return 90
    case '1y': return 365
    default: return 30
  }
}

async function runStressTest(scenarioId: string) {
  stressTestLoading.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 2000))

    const scenario = stressTestScenarios.value.find(s => s.id === scenarioId)
    if (scenario) {
      const impact = Math.random() * -500000 - 50000
      const recoveryMonths = Math.floor(Math.random() * 12) + 3

      stressTestResults.value.push({
        scenario: scenario.name,
        portfolioImpact: impact,
        recovery: `${recoveryMonths} months`
      })

      notificationStore.notifySystem(
        'Stress Test Complete',
        `${scenario.name} analysis finished`,
        'success'
      )
    }
  } finally {
    stressTestLoading.value = false
  }
}

async function runMonteCarloSimulation() {
  simulationLoading.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 3000))

    // Update simulation results
    monteCarloData.value.confidence = [
      { level: 95, value: -75000 - Math.random() * 10000 },
      { level: 99, value: -120000 - Math.random() * 15000 },
      { level: 99.9, value: -175000 - Math.random() * 20000 }
    ]

    notificationStore.notifySystem(
      'Monte Carlo Complete',
      '10,000 simulations finished',
      'success'
    )
  } finally {
    simulationLoading.value = false
  }
}

async function runOptimization() {
  optimizationLoading.value = true
  try {
    await new Promise(resolve => setTimeout(resolve, 2500))

    // Update optimization results
    optimizationMetrics.value = {
      expectedReturn: 12.5 + Math.random() * 2,
      volatility: 22.3 - Math.random() * 3,
      sharpeRatio: 2.15 + Math.random() * 0.3,
      improvement: {
        return: 1.8 + Math.random() * 0.5,
        risk: -18.5 - Math.random() * 5,
        sharpe: 0.3 + Math.random() * 0.1
      }
    }

    notificationStore.notifySystem(
      'Optimization Complete',
      'Portfolio optimization analysis finished',
      'success'
    )
  } finally {
    optimizationLoading.value = false
  }
}

function dismissAlert(alertId: string) {
  const index = riskAlerts.value.findIndex(alert => alert.id === alertId)
  if (index !== -1) {
    riskAlerts.value.splice(index, 1)
  }
}

function configureAlert(type: string) {
  notificationStore.notifySystem(
    'Alert Configuration',
    `Setting up ${type} alert...`,
    'info'
  )
}

function updateRiskLimit(metric: string, newLimit: number) {
  if (riskLimits.value[metric as keyof typeof riskLimits.value]) {
    (riskLimits.value[metric as keyof typeof riskLimits.value] as any).limit = newLimit
    notificationStore.notifySystem(
      'Risk Limit Updated',
      `${metric} limit updated to ${newLimit}`,
      'success'
    )
  }
}

function applyOptimization() {
  notificationStore.notifySystem(
    'Applying Optimization',
    'Rebalancing portfolio to optimal allocation...',
    'info'
  )
}
</script>

<style scoped>
/* Custom gradient for risk score */
.risk-score-gradient {
  background: conic-gradient(
    from 0deg,
    #10b981 0deg 108deg,
    #f59e0b 108deg 180deg,
    #f97316 180deg 252deg,
    #ef4444 252deg 360deg
  );
}
</style>