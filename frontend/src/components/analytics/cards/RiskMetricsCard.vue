<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ShieldExclamationIcon" class="w-5 h-5 text-orange-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Risk Metrics</h3>
            <p class="text-xs text-white/60">Portfolio risk assessment</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Risk Level Indicator -->
          <div :class="getRiskLevelClass()" class="px-2 py-1 rounded-lg text-xs font-medium">
            {{ getRiskLevel() }}
          </div>
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Risk Score Gauge -->
      <div class="flex items-center justify-center">
        <div class="relative w-32 h-32">
          <svg class="w-32 h-32 transform -rotate-90" viewBox="0 0 128 128">
            <!-- Background Circle -->
            <circle
              cx="64"
              cy="64"
              r="52"
              fill="none"
              stroke="rgba(255,255,255,0.1)"
              stroke-width="8"
            />

            <!-- Risk Score Arc -->
            <circle
              cx="64"
              cy="64"
              r="52"
              fill="none"
              :stroke="getRiskScoreColor()"
              stroke-width="8"
              :stroke-dasharray="`${(data.riskScore / 100) * 327} 327`"
              stroke-linecap="round"
              class="transition-all duration-1000"
            />
          </svg>

          <!-- Center Content -->
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="text-center">
              <div :class="['text-2xl font-bold', getRiskScoreTextColor()]">
                {{ data.riskScore }}
              </div>
              <div class="text-xs text-white/60">Risk Score</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Key Risk Metrics -->
      <div class="grid grid-cols-2 gap-4">
        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="ExclamationTriangleIcon" class="w-4 h-4 text-red-400" />
            <span class="text-xs text-white/60">Value at Risk (95%)</span>
          </div>
          <div class="text-lg font-bold text-red-400">
            ${{ formatCurrency(Math.abs(data.var95)) }}
          </div>
          <div class="text-xs text-white/60">Daily VaR</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="TrendingDownIcon" class="w-4 h-4 text-orange-400" />
            <span class="text-xs text-white/60">Max Drawdown</span>
          </div>
          <div :class="['text-lg font-bold', getDrawdownColor(data.maxDrawdown)]">
            {{ data.maxDrawdown.toFixed(1) }}%
          </div>
          <div class="text-xs text-white/60">Peak to trough</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-blue-400" />
            <span class="text-xs text-white/60">Volatility</span>
          </div>
          <div :class="['text-lg font-bold', getVolatilityColor(data.volatility)]">
            {{ data.volatility.toFixed(1) }}%
          </div>
          <div class="text-xs text-white/60">Annualized</div>
        </div>

        <div class="bg-slate-800/30 rounded-lg p-3">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="ScaleIcon" class="w-4 h-4 text-purple-400" />
            <span class="text-xs text-white/60">Beta</span>
          </div>
          <div :class="['text-lg font-bold', getBetaColor(data.beta)]">
            {{ data.beta.toFixed(2) }}
          </div>
          <div class="text-xs text-white/60">vs Market</div>
        </div>
      </div>

      <!-- Risk Breakdown -->
      <div class="space-y-3">
        <h4 class="text-sm font-semibold text-white">Risk Components</h4>

        <div class="space-y-2">
          <div
            v-for="component in riskComponents"
            :key="component.name"
            class="flex items-center justify-between"
          >
            <div class="flex items-center space-x-2">
              <div
                :class="component.colorClass"
                class="w-3 h-3 rounded-full"
              ></div>
              <span class="text-sm text-white/70">{{ component.name }}</span>
            </div>

            <div class="flex items-center space-x-2">
              <div class="w-20 h-2 bg-slate-700/50 rounded-full overflow-hidden">
                <div
                  :class="component.colorClass"
                  :style="{ width: `${component.value}%` }"
                  class="h-full transition-all duration-500"
                ></div>
              </div>
              <span class="text-sm font-medium text-white w-8 text-right">
                {{ component.value }}%
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Risk-Adjusted Returns -->
      <div class="grid grid-cols-2 gap-4">
        <div class="text-center">
          <div class="text-xs text-white/60">Sharpe Ratio</div>
          <div :class="['text-xl font-bold', getSharpeColor(data.sharpeRatio)]">
            {{ data.sharpeRatio.toFixed(2) }}
          </div>
          <div class="text-xs text-white/60">Risk-adjusted return</div>
        </div>

        <div class="text-center">
          <div class="text-xs text-white/60">Sortino Ratio</div>
          <div :class="['text-xl font-bold', getSortinoColor(data.sortinoRatio)]">
            {{ data.sortinoRatio.toFixed(2) }}
          </div>
          <div class="text-xs text-white/60">Downside deviation</div>
        </div>
      </div>

      <!-- Risk Alerts -->
      <div v-if="riskAlerts.length > 0" class="space-y-2">
        <h4 class="text-sm font-semibold text-white flex items-center space-x-2">
          <HeroIcon name="BellIcon" class="w-4 h-4 text-yellow-400" />
          <span>Risk Alerts</span>
        </h4>

        <div class="space-y-2">
          <div
            v-for="alert in riskAlerts"
            :key="alert.id"
            :class="getAlertClass(alert.severity)"
            class="p-3 rounded-lg border"
          >
            <div class="flex items-start space-x-2">
              <HeroIcon :name="getAlertIcon(alert.severity)" class="w-4 h-4 mt-0.5 flex-shrink-0" />
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium">{{ alert.title }}</div>
                <div class="text-xs mt-1 opacity-80">{{ alert.message }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Risk Trend -->
      <div class="space-y-3">
        <h4 class="text-sm font-semibold text-white">Risk Trend (7 Days)</h4>

        <div class="flex items-end space-x-1 h-16">
          <div
            v-for="(day, index) in riskTrend"
            :key="index"
            class="flex-1 flex flex-col items-center"
          >
            <div
              :class="getRiskTrendColor(day.risk)"
              :style="{ height: `${(day.risk / 100) * 100}%` }"
              class="w-full rounded-sm min-h-1 transition-all duration-300"
            ></div>
            <div class="text-xs text-white/60 mt-1">
              {{ day.day }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <span>Last updated: {{ formatLastUpdate(data.lastUpdated) }}</span>
        <Button
          variant="ghost"
          size="xs"
          icon-right="ChevronRightIcon"
          @click="$emit('detailed-analysis')"
        >
          Detailed Analysis
        </Button>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface RiskAlert {
  id: string
  title: string
  message: string
  severity: 'low' | 'medium' | 'high'
}

interface RiskData {
  riskScore: number
  var95: number
  maxDrawdown: number
  volatility: number
  beta: number
  sharpeRatio: number
  sortinoRatio: number
  lastUpdated: string
}

interface Props {
  data: RiskData
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'detailed-analysis': []
}>()

// Mock data for risk components and alerts
const riskComponents = computed(() => [
  { name: 'Market Risk', value: 65, colorClass: 'bg-red-400' },
  { name: 'Concentration Risk', value: 45, colorClass: 'bg-orange-400' },
  { name: 'Liquidity Risk', value: 25, colorClass: 'bg-yellow-400' },
  { name: 'Operational Risk', value: 15, colorClass: 'bg-blue-400' }
])

const riskAlerts = computed(() => {
  const alerts: RiskAlert[] = []

  if (props.data.maxDrawdown > 15) {
    alerts.push({
      id: '1',
      title: 'High Drawdown Risk',
      message: 'Current drawdown exceeds 15% threshold',
      severity: 'high'
    })
  }

  if (props.data.volatility > 30) {
    alerts.push({
      id: '2',
      title: 'High Volatility',
      message: 'Portfolio volatility above 30%',
      severity: 'medium'
    })
  }

  if (props.data.beta > 1.5) {
    alerts.push({
      id: '3',
      title: 'High Market Sensitivity',
      message: 'Beta exceeds 1.5x market movements',
      severity: 'medium'
    })
  }

  return alerts
})

const riskTrend = computed(() => {
  const days = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
  return days.map(day => ({
    day: day.slice(0, 1),
    risk: 40 + Math.random() * 40 // Risk score between 40-80
  }))
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(1)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(1)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(1)}K`
  return amount.toFixed(0)
}

function formatLastUpdate(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`
  return date.toLocaleDateString()
}

function getRiskLevel(): string {
  if (props.data.riskScore <= 30) return 'Low Risk'
  if (props.data.riskScore <= 50) return 'Moderate Risk'
  if (props.data.riskScore <= 70) return 'High Risk'
  return 'Very High Risk'
}

function getRiskLevelClass(): string {
  if (props.data.riskScore <= 30) return 'bg-green-500/20 text-green-400'
  if (props.data.riskScore <= 50) return 'bg-yellow-500/20 text-yellow-400'
  if (props.data.riskScore <= 70) return 'bg-orange-500/20 text-orange-400'
  return 'bg-red-500/20 text-red-400'
}

function getRiskScoreColor(): string {
  if (props.data.riskScore <= 30) return '#10b981'
  if (props.data.riskScore <= 50) return '#f59e0b'
  if (props.data.riskScore <= 70) return '#f97316'
  return '#ef4444'
}

function getRiskScoreTextColor(): string {
  if (props.data.riskScore <= 30) return 'text-green-400'
  if (props.data.riskScore <= 50) return 'text-yellow-400'
  if (props.data.riskScore <= 70) return 'text-orange-400'
  return 'text-red-400'
}

function getDrawdownColor(drawdown: number): string {
  if (drawdown <= 10) return 'text-green-400'
  if (drawdown <= 20) return 'text-yellow-400'
  return 'text-red-400'
}

function getVolatilityColor(volatility: number): string {
  if (volatility <= 20) return 'text-green-400'
  if (volatility <= 35) return 'text-yellow-400'
  return 'text-red-400'
}

function getBetaColor(beta: number): string {
  if (beta <= 1) return 'text-green-400'
  if (beta <= 1.5) return 'text-yellow-400'
  return 'text-red-400'
}

function getSharpeColor(sharpe: number): string {
  if (sharpe >= 1.5) return 'text-green-400'
  if (sharpe >= 1) return 'text-yellow-400'
  return 'text-red-400'
}

function getSortinoColor(sortino: number): string {
  if (sortino >= 2) return 'text-green-400'
  if (sortino >= 1.5) return 'text-yellow-400'
  return 'text-red-400'
}

function getAlertClass(severity: string): string {
  switch (severity) {
    case 'high':
      return 'bg-red-500/10 border-red-500/30 text-red-400'
    case 'medium':
      return 'bg-yellow-500/10 border-yellow-500/30 text-yellow-400'
    default:
      return 'bg-blue-500/10 border-blue-500/30 text-blue-400'
  }
}

function getAlertIcon(severity: string): string {
  switch (severity) {
    case 'high':
      return 'ExclamationTriangleIcon'
    case 'medium':
      return 'ExclamationCircleIcon'
    default:
      return 'InformationCircleIcon'
  }
}

function getRiskTrendColor(risk: number): string {
  if (risk <= 30) return 'bg-green-400'
  if (risk <= 50) return 'bg-yellow-400'
  if (risk <= 70) return 'bg-orange-400'
  return 'bg-red-400'
}
</script>

<style scoped>
/* Risk gauge animation */
circle {
  transition: stroke-dasharray 1s ease-out;
}

/* Risk trend bars animation */
.transition-all {
  transition: all 0.3s ease;
}

/* Alert animations */
.border {
  transition: border-color 0.2s ease;
}
</style>