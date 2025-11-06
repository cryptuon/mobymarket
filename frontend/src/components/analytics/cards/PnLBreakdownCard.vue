<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="CurrencyDollarIcon" class="w-5 h-5 text-green-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">P&L Breakdown</h3>
            <p class="text-xs text-white/60">Profit and loss analysis</p>
          </div>
        </div>

        <Button
          @click="$emit('drill-down', data)"
          variant="ghost"
          size="sm"
          icon-right="ArrowTopRightOnSquareIcon"
        >
          Details
        </Button>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Summary Cards -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="bg-green-500/10 border border-green-500/30 rounded-lg p-4">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="TrendingUpIcon" class="w-4 h-4 text-green-400" />
            <span class="text-xs text-green-400 font-medium">Realized P&L</span>
          </div>
          <div class="text-2xl font-bold text-white">${{ formatCurrency(data.realized) }}</div>
          <div class="text-xs text-green-400/80 mt-1">Locked-in gains</div>
        </div>

        <div class="bg-blue-500/10 border border-blue-500/30 rounded-lg p-4">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-blue-400" />
            <span class="text-xs text-blue-400 font-medium">Unrealized P&L</span>
          </div>
          <div class="text-2xl font-bold text-white">${{ formatCurrency(data.unrealized) }}</div>
          <div class="text-xs text-blue-400/80 mt-1">Open positions</div>
        </div>

        <div class="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
          <div class="flex items-center space-x-2 mb-2">
            <HeroIcon name="CreditCardIcon" class="w-4 h-4 text-red-400" />
            <span class="text-xs text-red-400 font-medium">Total Fees</span>
          </div>
          <div class="text-2xl font-bold text-white">${{ formatCurrency(Math.abs(data.fees)) }}</div>
          <div class="text-xs text-red-400/80 mt-1">Trading costs</div>
        </div>
      </div>

      <!-- P&L Breakdown Chart -->
      <div class="space-y-4">
        <h4 class="text-sm font-semibold text-white">P&L by Category</h4>

        <div class="space-y-3">
          <div
            v-for="category in data.breakdown"
            :key="category.category"
            class="flex items-center justify-between p-3 bg-slate-800/30 rounded-lg hover:bg-slate-700/30 transition-colors"
          >
            <!-- Category Info -->
            <div class="flex items-center space-x-3 flex-1">
              <div
                :class="getCategoryIconClass(category.category)"
                class="w-8 h-8 rounded-lg flex items-center justify-center"
              >
                <HeroIcon :name="getCategoryIcon(category.category)" class="w-4 h-4" />
              </div>
              <div>
                <div class="text-white font-medium">{{ category.category }}</div>
                <div class="text-xs text-white/60">{{ category.percentage }}% of total</div>
              </div>
            </div>

            <!-- P&L Amount -->
            <div class="text-right">
              <div :class="['text-lg font-bold', getPnLColor(category.pnl)]">
                {{ category.pnl >= 0 ? '+' : '' }}${{ formatCurrency(Math.abs(category.pnl)) }}
              </div>
              <div class="text-xs text-white/60">
                {{ category.pnl >= 0 ? 'Profit' : 'Loss' }}
              </div>
            </div>

            <!-- Progress Bar -->
            <div class="ml-4 w-24">
              <div class="h-2 bg-slate-700 rounded-full overflow-hidden">
                <div
                  :class="getPnLBarClass(category.pnl)"
                  :style="{ width: `${Math.abs(category.percentage)}%` }"
                  class="h-full transition-all duration-500"
                ></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Performance Metrics -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="text-center">
          <div class="text-xs text-white/60">Total P&L</div>
          <div :class="['text-xl font-bold', getTotalPnLColor()]">
            {{ totalPnL >= 0 ? '+' : '' }}${{ formatCurrency(Math.abs(totalPnL)) }}
          </div>
        </div>

        <div class="text-center">
          <div class="text-xs text-white/60">Win Rate</div>
          <div :class="['text-xl font-bold', getWinRateColor(winRate)]">
            {{ winRate.toFixed(1) }}%
          </div>
        </div>

        <div class="text-center">
          <div class="text-xs text-white/60">Profit Factor</div>
          <div :class="['text-xl font-bold', getProfitFactorColor(profitFactor)]">
            {{ profitFactor.toFixed(2) }}
          </div>
        </div>

        <div class="text-center">
          <div class="text-xs text-white/60">Return on Equity</div>
          <div :class="['text-xl font-bold', getROEColor(roe)]">
            {{ roe >= 0 ? '+' : '' }}{{ roe.toFixed(1) }}%
          </div>
        </div>
      </div>

      <!-- Monthly Trend -->
      <div class="space-y-3">
        <h4 class="text-sm font-semibold text-white">Recent Performance Trend</h4>

        <div class="grid grid-cols-6 gap-2">
          <div
            v-for="(month, index) in monthlyTrend"
            :key="index"
            class="text-center"
          >
            <div class="text-xs text-white/60 mb-1">{{ month.month }}</div>
            <div
              :class="[
                'h-12 rounded-md flex items-end justify-center p-1 transition-all duration-300',
                month.pnl >= 0 ? 'bg-green-500/20' : 'bg-red-500/20'
              ]"
            >
              <div
                :class="[
                  'w-full rounded-sm transition-all duration-500',
                  month.pnl >= 0 ? 'bg-green-400' : 'bg-red-400'
                ]"
                :style="{ height: `${Math.abs(month.pnl) / maxMonthlyPnL * 100}%` }"
              ></div>
            </div>
            <div :class="['text-xs font-medium mt-1', month.pnl >= 0 ? 'text-green-400' : 'text-red-400']">
              {{ month.pnl >= 0 ? '+' : '' }}{{ month.pnl.toFixed(1) }}%
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface PnLBreakdown {
  category: string
  pnl: number
  percentage: number
}

interface PnLData {
  realized: number
  unrealized: number
  fees: number
  breakdown: PnLBreakdown[]
}

interface Props {
  data: PnLData
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'drill-down': [data: PnLData]
}>()

// Computed properties
const totalPnL = computed(() => {
  return props.data.realized + props.data.unrealized + props.data.fees
})

const winRate = computed(() => {
  // Mock calculation - in real app would be based on trade history
  const profitableCategories = props.data.breakdown.filter(b => b.pnl > 0).length
  return (profitableCategories / props.data.breakdown.length) * 100
})

const profitFactor = computed(() => {
  const totalProfit = props.data.breakdown
    .filter(b => b.pnl > 0)
    .reduce((sum, b) => sum + b.pnl, 0)

  const totalLoss = Math.abs(props.data.breakdown
    .filter(b => b.pnl < 0)
    .reduce((sum, b) => sum + b.pnl, 0))

  return totalLoss > 0 ? totalProfit / totalLoss : totalProfit > 0 ? 999 : 0
})

const roe = computed(() => {
  // Mock ROE calculation
  const initialEquity = 1000000 // Mock initial investment
  return (totalPnL.value / initialEquity) * 100
})

const monthlyTrend = computed(() => {
  const months = ['Nov', 'Dec', 'Jan', 'Feb', 'Mar', 'Apr']
  return months.map(month => ({
    month,
    pnl: (Math.random() - 0.4) * 15 // Slight positive bias
  }))
})

const maxMonthlyPnL = computed(() => {
  return Math.max(...monthlyTrend.value.map(m => Math.abs(m.pnl)))
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(0)
}

function getCategoryIcon(category: string): string {
  const iconMap: Record<string, string> = {
    'Spot Trading': 'ArrowsRightLeftIcon',
    'DeFi Farming': 'CubeTransparentIcon',
    'Arbitrage': 'ScaleIcon',
    'Options': 'AdjustmentsHorizontalIcon',
    'Lending': 'BanknotesIcon',
    'Staking': 'LockClosedIcon'
  }
  return iconMap[category] || 'CurrencyDollarIcon'
}

function getCategoryIconClass(category: string): string {
  const classMap: Record<string, string> = {
    'Spot Trading': 'bg-blue-500/20 text-blue-400',
    'DeFi Farming': 'bg-green-500/20 text-green-400',
    'Arbitrage': 'bg-purple-500/20 text-purple-400',
    'Options': 'bg-orange-500/20 text-orange-400',
    'Lending': 'bg-yellow-500/20 text-yellow-400',
    'Staking': 'bg-cyan-500/20 text-cyan-400'
  }
  return classMap[category] || 'bg-slate-500/20 text-slate-400'
}

function getPnLColor(pnl: number): string {
  return pnl >= 0 ? 'text-green-400' : 'text-red-400'
}

function getPnLBarClass(pnl: number): string {
  return pnl >= 0 ? 'bg-green-400' : 'bg-red-400'
}

function getTotalPnLColor(): string {
  return totalPnL.value >= 0 ? 'text-green-400' : 'text-red-400'
}

function getWinRateColor(rate: number): string {
  if (rate >= 70) return 'text-green-400'
  if (rate >= 50) return 'text-yellow-400'
  return 'text-red-400'
}

function getProfitFactorColor(factor: number): string {
  if (factor >= 2) return 'text-green-400'
  if (factor >= 1.5) return 'text-yellow-400'
  return 'text-red-400'
}

function getROEColor(roe: number): string {
  return roe >= 0 ? 'text-green-400' : 'text-red-400'
}
</script>

<style scoped>
/* Progress bar animations */
.transition-all {
  transition: all 0.5s ease;
}

/* Monthly trend bar animations */
.h-12 > div {
  transition: height 0.5s ease;
}
</style>