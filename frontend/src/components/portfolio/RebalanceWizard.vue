<template>
  <div class="space-y-6">
    <!-- Wizard Header -->
    <Card variant="glass">
      <div class="p-6">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center space-x-4">
            <div class="w-12 h-12 bg-gradient-to-br from-orange-400 to-orange-600 rounded-xl flex items-center justify-center">
              <HeroIcon name="ScaleIcon" class="w-6 h-6 text-white" />
            </div>
            <div>
              <h2 class="text-2xl font-bold text-white">Portfolio Rebalancing</h2>
              <p class="text-sm text-white/60">Optimize your asset allocation</p>
            </div>
          </div>

          <Button
            variant="ghost"
            icon-left="XMarkIcon"
            @click="$emit('close')"
          />
        </div>

        <!-- Progress Steps -->
        <div class="flex items-center space-x-4">
          <div
            v-for="(step, index) in steps"
            :key="step.id"
            class="flex items-center"
          >
            <div class="flex items-center space-x-3">
              <div
                :class="[
                  'w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-all',
                  index < currentStep
                    ? 'bg-green-500 text-white'
                    : index === currentStep
                    ? 'bg-moby-500 text-white'
                    : 'bg-slate-700 text-white/60'
                ]"
              >
                <HeroIcon
                  v-if="index < currentStep"
                  name="CheckIcon"
                  class="w-4 h-4"
                />
                <span v-else>{{ index + 1 }}</span>
              </div>
              <div>
                <div :class="['text-sm font-medium', index <= currentStep ? 'text-white' : 'text-white/60']">
                  {{ step.title }}
                </div>
                <div class="text-xs text-white/60">{{ step.description }}</div>
              </div>
            </div>
            <div
              v-if="index < steps.length - 1"
              :class="[
                'w-12 h-0.5 mx-4 transition-all',
                index < currentStep ? 'bg-green-500' : 'bg-slate-700'
              ]"
            ></div>
          </div>
        </div>
      </div>
    </Card>

    <!-- Step Content -->
    <Card variant="glass">
      <div class="p-6">
        <!-- Step 1: Strategy Selection -->
        <div v-if="currentStep === 0" class="space-y-6">
          <h3 class="text-xl font-bold text-white">Choose Rebalancing Strategy</h3>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <div
              v-for="strategy in rebalanceStrategies"
              :key="strategy.id"
              :class="[
                'p-6 rounded-lg border-2 cursor-pointer transition-all',
                selectedStrategy?.id === strategy.id
                  ? 'border-moby-500 bg-moby-500/10'
                  : 'border-slate-600/50 bg-slate-800/30 hover:border-slate-500/50'
              ]"
              @click="selectedStrategy = strategy"
            >
              <div class="flex items-start space-x-4">
                <div :class="strategy.iconClass" class="p-3 rounded-lg">
                  <HeroIcon :name="strategy.icon" class="w-6 h-6" />
                </div>
                <div class="flex-1">
                  <h4 class="text-lg font-semibold text-white mb-2">{{ strategy.name }}</h4>
                  <p class="text-sm text-white/70 mb-4">{{ strategy.description }}</p>

                  <div class="space-y-2">
                    <div class="flex justify-between text-xs">
                      <span class="text-white/60">Risk Level</span>
                      <span :class="getRiskColor(strategy.riskLevel)">{{ strategy.riskLevel }}</span>
                    </div>
                    <div class="flex justify-between text-xs">
                      <span class="text-white/60">Frequency</span>
                      <span class="text-white">{{ strategy.frequency }}</span>
                    </div>
                    <div class="flex justify-between text-xs">
                      <span class="text-white/60">Expected Return</span>
                      <span class="text-green-400">{{ strategy.expectedReturn }}</span>
                    </div>
                  </div>
                </div>
              </div>

              <div v-if="selectedStrategy?.id === strategy.id" class="mt-4 pt-4 border-t border-white/10">
                <div class="text-sm text-white/70">
                  <strong>Details:</strong> {{ strategy.details }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 2: Target Allocation -->
        <div v-if="currentStep === 1" class="space-y-6">
          <div class="flex items-center justify-between">
            <h3 class="text-xl font-bold text-white">Set Target Allocation</h3>
            <div class="flex space-x-2">
              <Button
                variant="ghost"
                size="sm"
                @click="useRecommendedAllocation"
              >
                Use Recommended
              </Button>
              <Button
                variant="ghost"
                size="sm"
                @click="resetToCurrentAllocation"
              >
                Reset to Current
              </Button>
            </div>
          </div>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <!-- Current vs Target Comparison -->
            <div class="space-y-4">
              <h4 class="text-lg font-semibold text-white">Allocation Comparison</h4>

              <div class="space-y-3">
                <div
                  v-for="position in positions"
                  :key="position.symbol"
                  class="flex items-center space-x-4 p-3 bg-slate-800/30 rounded-lg"
                >
                  <img :src="getAssetIcon(position.symbol)" :alt="position.symbol" class="w-8 h-8 rounded-full" />

                  <div class="flex-1">
                    <div class="flex items-center justify-between mb-2">
                      <span class="text-sm font-medium text-white">{{ position.symbol }}</span>
                      <div class="flex items-center space-x-2 text-xs">
                        <span class="text-white/60">Current: {{ position.currentAllocation.toFixed(1) }}%</span>
                        <span class="text-white/40">→</span>
                        <span class="text-white">Target: {{ position.targetAllocation.toFixed(1) }}%</span>
                      </div>
                    </div>

                    <!-- Allocation Slider -->
                    <div class="space-y-2">
                      <input
                        v-model="position.targetAllocation"
                        type="range"
                        min="0"
                        max="50"
                        step="0.1"
                        class="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer slider"
                        @input="updateTargetAllocation"
                      />
                      <div class="flex justify-between text-xs text-white/60">
                        <span>0%</span>
                        <span>25%</span>
                        <span>50%</span>
                      </div>
                    </div>
                  </div>

                  <div class="text-right">
                    <input
                      v-model="position.targetAllocation"
                      type="number"
                      min="0"
                      max="100"
                      step="0.1"
                      class="w-16 bg-slate-700/50 border border-slate-600/50 rounded px-2 py-1 text-xs text-white text-right focus:outline-none focus:border-moby-500/50"
                      @input="updateTargetAllocation"
                    />
                    <div class="text-xs text-white/60 mt-1">%</div>
                  </div>
                </div>
              </div>

              <!-- Total Allocation Check -->
              <div :class="['p-3 rounded-lg', getTotalAllocationClass()]">
                <div class="flex items-center space-x-2">
                  <HeroIcon :name="getTotalAllocationIcon()" class="w-4 h-4" />
                  <span class="text-sm font-medium">
                    Total Allocation: {{ totalTargetAllocation.toFixed(1) }}%
                  </span>
                </div>
                <div v-if="totalTargetAllocation !== 100" class="text-xs mt-1 opacity-80">
                  {{ totalTargetAllocation > 100 ? 'Reduce by' : 'Add' }} {{ Math.abs(100 - totalTargetAllocation).toFixed(1) }}%
                </div>
              </div>
            </div>

            <!-- Allocation Visualization -->
            <div class="space-y-4">
              <h4 class="text-lg font-semibold text-white">Visual Comparison</h4>

              <div class="space-y-6">
                <!-- Current Allocation Pie -->
                <div>
                  <h5 class="text-sm font-medium text-white mb-3">Current Allocation</h5>
                  <div class="relative w-48 h-48 mx-auto">
                    <svg class="w-48 h-48" viewBox="0 0 200 200">
                      <g v-for="(segment, index) in currentAllocationSegments" :key="index">
                        <path
                          :d="segment.path"
                          :fill="segment.color"
                          class="opacity-80"
                        />
                      </g>
                    </svg>
                  </div>
                </div>

                <!-- Target Allocation Pie -->
                <div>
                  <h5 class="text-sm font-medium text-white mb-3">Target Allocation</h5>
                  <div class="relative w-48 h-48 mx-auto">
                    <svg class="w-48 h-48" viewBox="0 0 200 200">
                      <g v-for="(segment, index) in targetAllocationSegments" :key="index">
                        <path
                          :d="segment.path"
                          :fill="segment.color"
                          class="opacity-80"
                        />
                      </g>
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 3: Review & Execute -->
        <div v-if="currentStep === 2" class="space-y-6">
          <h3 class="text-xl font-bold text-white">Review Rebalancing Plan</h3>

          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <!-- Trades Required -->
            <div class="space-y-4">
              <h4 class="text-lg font-semibold text-white">Required Trades</h4>

              <div class="space-y-3">
                <div
                  v-for="trade in requiredTrades"
                  :key="trade.asset"
                  class="flex items-center space-x-4 p-4 bg-slate-800/30 rounded-lg"
                >
                  <div :class="getTradeTypeClass(trade.type)" class="p-2 rounded-lg">
                    <HeroIcon :name="getTradeTypeIcon(trade.type)" class="w-4 h-4" />
                  </div>

                  <div class="flex-1">
                    <div class="flex items-center space-x-2 mb-1">
                      <img :src="getAssetIcon(trade.asset)" :alt="trade.asset" class="w-5 h-5 rounded-full" />
                      <span class="text-sm font-medium text-white">{{ trade.type }} {{ trade.asset }}</span>
                    </div>
                    <div class="text-xs text-white/60">
                      {{ formatAmount(trade.amount) }} {{ trade.asset }} • ${{ formatAmount(trade.value) }}
                    </div>
                  </div>

                  <div class="text-right">
                    <div class="text-sm font-bold text-white">${{ formatAmount(trade.estimatedFee) }}</div>
                    <div class="text-xs text-white/60">Fee</div>
                  </div>
                </div>
              </div>

              <div class="p-4 bg-slate-800/20 rounded-lg">
                <div class="flex justify-between items-center">
                  <span class="text-sm font-medium text-white">Total Estimated Fees</span>
                  <span class="text-lg font-bold text-white">
                    ${{ formatAmount(totalEstimatedFees) }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Impact Summary -->
            <div class="space-y-4">
              <h4 class="text-lg font-semibold text-white">Rebalancing Impact</h4>

              <div class="space-y-3">
                <div class="p-4 bg-slate-800/30 rounded-lg">
                  <div class="flex items-center space-x-2 mb-2">
                    <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-blue-400" />
                    <span class="text-sm font-medium text-white">Risk Adjustment</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-white/60">Current Risk Score</span>
                    <span class="text-white">{{ currentRiskScore }}/100</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-white/60">Target Risk Score</span>
                    <span :class="getRiskChangeColor(targetRiskScore - currentRiskScore)">
                      {{ targetRiskScore }}/100
                      ({{ targetRiskScore > currentRiskScore ? '+' : '' }}{{ (targetRiskScore - currentRiskScore).toFixed(0) }})
                    </span>
                  </div>
                </div>

                <div class="p-4 bg-slate-800/30 rounded-lg">
                  <div class="flex items-center space-x-2 mb-2">
                    <HeroIcon name="ScaleIcon" class="w-4 h-4 text-purple-400" />
                    <span class="text-sm font-medium text-white">Diversification</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-white/60">Current Diversity Score</span>
                    <span class="text-white">{{ currentDiversityScore }}/100</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-white/60">Target Diversity Score</span>
                    <span :class="getRiskChangeColor(targetDiversityScore - currentDiversityScore)">
                      {{ targetDiversityScore }}/100
                      ({{ targetDiversityScore > currentDiversityScore ? '+' : '' }}{{ (targetDiversityScore - currentDiversityScore).toFixed(0) }})
                    </span>
                  </div>
                </div>

                <div class="p-4 bg-slate-800/30 rounded-lg">
                  <div class="flex items-center space-x-2 mb-2">
                    <HeroIcon name="CurrencyDollarIcon" class="w-4 h-4 text-green-400" />
                    <span class="text-sm font-medium text-white">Cost Analysis</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-white/60">Trading Fees</span>
                    <span class="text-red-400">${{ formatAmount(totalEstimatedFees) }}</span>
                  </div>
                  <div class="flex justify-between text-sm">
                    <span class="text-white/60">Tax Impact</span>
                    <span class="text-yellow-400">${{ formatAmount(estimatedTaxImpact) }}</span>
                  </div>
                  <div class="flex justify-between text-sm font-medium border-t border-white/10 pt-2 mt-2">
                    <span class="text-white">Total Cost</span>
                    <span class="text-white">${{ formatAmount(totalEstimatedFees + estimatedTaxImpact) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Risk Warning -->
          <div v-if="hasHighRiskTrades" class="p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
            <div class="flex items-start space-x-3">
              <HeroIcon name="ExclamationTriangleIcon" class="w-5 h-5 text-yellow-400 mt-0.5 flex-shrink-0" />
              <div>
                <h5 class="text-sm font-semibold text-yellow-400 mb-1">High Risk Rebalancing Detected</h5>
                <p class="text-xs text-yellow-400/80">
                  This rebalancing involves significant position changes that may impact your portfolio's risk profile.
                  Please review carefully before proceeding.
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Card>

    <!-- Action Buttons -->
    <div class="flex items-center justify-between">
      <Button
        v-if="currentStep > 0"
        variant="ghost"
        icon-left="ChevronLeftIcon"
        @click="previousStep"
      >
        Previous
      </Button>
      <div v-else></div>

      <div class="flex space-x-3">
        <Button
          variant="secondary"
          @click="$emit('close')"
        >
          Cancel
        </Button>

        <Button
          v-if="currentStep < steps.length - 1"
          variant="primary"
          icon-right="ChevronRightIcon"
          :disabled="!canProceed"
          @click="nextStep"
        >
          Continue
        </Button>

        <Button
          v-else
          variant="primary"
          icon-left="PlayIcon"
          :disabled="!canExecute"
          @click="executeRebalancing"
        >
          Execute Rebalancing
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface Position {
  symbol: string
  name: string
  currentAllocation: number
  targetAllocation: number
  currentValue: number
}

interface RebalanceStrategy {
  id: string
  name: string
  description: string
  details: string
  icon: string
  iconClass: string
  riskLevel: string
  frequency: string
  expectedReturn: string
}

interface Trade {
  asset: string
  type: 'buy' | 'sell'
  amount: number
  value: number
  estimatedFee: number
}

interface Props {
  positions: Position[]
  portfolioValue: number
}

const props = withDefaults(defineProps<Props>(), {
  portfolioValue: 125000
})

const emit = defineEmits<{
  close: []
  execute: [trades: Trade[]]
}>()

const currentStep = ref(0)
const selectedStrategy = ref<RebalanceStrategy | null>(null)

const steps = [
  {
    id: 'strategy',
    title: 'Strategy',
    description: 'Choose approach'
  },
  {
    id: 'allocation',
    title: 'Allocation',
    description: 'Set targets'
  },
  {
    id: 'review',
    title: 'Review',
    description: 'Execute plan'
  }
]

const rebalanceStrategies: RebalanceStrategy[] = [
  {
    id: 'conservative',
    name: 'Conservative Rebalancing',
    description: 'Minimal changes with focus on stability and low fees',
    details: 'Only rebalances when allocations drift more than 5% from target. Prioritizes stability over optimization.',
    icon: 'ShieldCheckIcon',
    iconClass: 'bg-green-500/20 text-green-400',
    riskLevel: 'Low',
    frequency: 'Quarterly',
    expectedReturn: '8-12%'
  },
  {
    id: 'balanced',
    name: 'Balanced Optimization',
    description: 'Moderate rebalancing with risk-adjusted returns focus',
    details: 'Rebalances when allocations drift more than 3% from target. Balances growth potential with risk management.',
    icon: 'ScaleIcon',
    iconClass: 'bg-blue-500/20 text-blue-400',
    riskLevel: 'Medium',
    frequency: 'Monthly',
    expectedReturn: '12-18%'
  },
  {
    id: 'aggressive',
    name: 'Aggressive Growth',
    description: 'Active rebalancing for maximum growth potential',
    details: 'Frequently rebalances to capture market opportunities. Higher risk but potentially higher returns.',
    icon: 'RocketLaunchIcon',
    iconClass: 'bg-red-500/20 text-red-400',
    riskLevel: 'High',
    frequency: 'Weekly',
    expectedReturn: '18-25%'
  },
  {
    id: 'custom',
    name: 'Custom Strategy',
    description: 'Define your own rebalancing parameters',
    details: 'Set custom thresholds, frequency, and optimization criteria based on your specific needs.',
    icon: 'CogIcon',
    iconClass: 'bg-purple-500/20 text-purple-400',
    riskLevel: 'Variable',
    frequency: 'Custom',
    expectedReturn: 'Variable'
  }
]

// Generate mock positions if none provided
const generateMockPositions = (): Position[] => {
  const assets = [
    { symbol: 'ETH', name: 'Ethereum' },
    { symbol: 'BTC', name: 'Bitcoin' },
    { symbol: 'UNI', name: 'Uniswap' },
    { symbol: 'AAVE', name: 'Aave' },
    { symbol: 'COMP', name: 'Compound' }
  ]

  return assets.map((asset, index) => {
    const currentAllocation = index === 0 ? 35 : index === 1 ? 25 : 40 / (assets.length - 2)
    const targetAllocation = index === 0 ? 30 : index === 1 ? 30 : 40 / (assets.length - 2)

    return {
      symbol: asset.symbol,
      name: asset.name,
      currentAllocation,
      targetAllocation,
      currentValue: (props.portfolioValue * currentAllocation) / 100
    }
  })
}

const positions = ref(props.positions.length ? [...props.positions] : generateMockPositions())

const totalTargetAllocation = computed(() => {
  return positions.value.reduce((sum, p) => sum + p.targetAllocation, 0)
})

const canProceed = computed(() => {
  switch (currentStep.value) {
    case 0:
      return selectedStrategy.value !== null
    case 1:
      return Math.abs(totalTargetAllocation.value - 100) < 0.1
    case 2:
      return true
    default:
      return false
  }
})

const canExecute = computed(() => {
  return canProceed.value && requiredTrades.value.length > 0
})

const requiredTrades = computed((): Trade[] => {
  const trades: Trade[] = []

  positions.value.forEach(position => {
    const currentValue = position.currentValue
    const targetValue = (props.portfolioValue * position.targetAllocation) / 100
    const difference = targetValue - currentValue

    if (Math.abs(difference) > 100) { // Only trade if difference > $100
      trades.push({
        asset: position.symbol,
        type: difference > 0 ? 'buy' : 'sell',
        amount: Math.abs(difference) / 2000, // Mock calculation
        value: Math.abs(difference),
        estimatedFee: Math.abs(difference) * 0.003 // 0.3% fee
      })
    }
  })

  return trades
})

const totalEstimatedFees = computed(() => {
  return requiredTrades.value.reduce((sum, trade) => sum + trade.estimatedFee, 0)
})

const estimatedTaxImpact = computed(() => {
  // Mock calculation - assume 15% capital gains on profitable trades
  const profitableTrades = requiredTrades.value.filter(t => t.type === 'sell')
  return profitableTrades.reduce((sum, trade) => sum + trade.value * 0.15 * 0.5, 0) // Assume 50% profit
})

const hasHighRiskTrades = computed(() => {
  return requiredTrades.value.some(trade => trade.value > props.portfolioValue * 0.1)
})

const currentRiskScore = computed(() => 65) // Mock value
const targetRiskScore = computed(() => 58) // Mock value
const currentDiversityScore = computed(() => 72) // Mock value
const targetDiversityScore = computed(() => 85) // Mock value

// Pie chart segments
const currentAllocationSegments = computed(() => {
  return generatePieSegments(positions.value.map(p => ({
    ...p,
    percentage: p.currentAllocation
  })))
})

const targetAllocationSegments = computed(() => {
  return generatePieSegments(positions.value.map(p => ({
    ...p,
    percentage: p.targetAllocation
  })))
})

// Methods
function generatePieSegments(data: any[]) {
  let currentAngle = 0
  const radius = 80
  const centerX = 100
  const centerY = 100

  return data.map((item, index) => {
    const angle = (item.percentage / 100) * 2 * Math.PI
    const startAngle = currentAngle
    const endAngle = currentAngle + angle

    const x1 = centerX + radius * Math.cos(startAngle)
    const y1 = centerY + radius * Math.sin(startAngle)
    const x2 = centerX + radius * Math.cos(endAngle)
    const y2 = centerY + radius * Math.sin(endAngle)

    const largeArcFlag = angle > Math.PI ? 1 : 0

    const path = [
      `M ${centerX} ${centerY}`,
      `L ${x1} ${y1}`,
      `A ${radius} ${radius} 0 ${largeArcFlag} 1 ${x2} ${y2}`,
      `Z`
    ].join(' ')

    currentAngle += angle

    return {
      path,
      color: getAssetColor(index)
    }
  })
}

function getAssetColor(index: number): string {
  const colors = ['#60a5fa', '#a855f7', '#4ade80', '#facc15', '#f87171']
  return colors[index % colors.length]
}

function formatAmount(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function getRiskColor(risk: string): string {
  switch (risk.toLowerCase()) {
    case 'low': return 'text-green-400'
    case 'medium': return 'text-yellow-400'
    case 'high': return 'text-red-400'
    default: return 'text-gray-400'
  }
}

function getRiskChangeColor(change: number): string {
  return change > 0 ? 'text-green-400' : change < 0 ? 'text-red-400' : 'text-white'
}

function getAssetIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    UNI: '/tokens/uni.svg',
    AAVE: '/tokens/aave.svg',
    COMP: '/tokens/comp.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function getTotalAllocationClass(): string {
  const diff = Math.abs(100 - totalTargetAllocation.value)
  if (diff < 0.1) return 'bg-green-500/10 border border-green-500/30 text-green-400'
  if (diff < 5) return 'bg-yellow-500/10 border border-yellow-500/30 text-yellow-400'
  return 'bg-red-500/10 border border-red-500/30 text-red-400'
}

function getTotalAllocationIcon(): string {
  const diff = Math.abs(100 - totalTargetAllocation.value)
  if (diff < 0.1) return 'CheckCircleIcon'
  if (diff < 5) return 'ExclamationCircleIcon'
  return 'ExclamationTriangleIcon'
}

function getTradeTypeClass(type: string): string {
  return type === 'buy' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'
}

function getTradeTypeIcon(type: string): string {
  return type === 'buy' ? 'ArrowDownIcon' : 'ArrowUpIcon'
}

function nextStep() {
  if (canProceed.value && currentStep.value < steps.length - 1) {
    currentStep.value++
  }
}

function previousStep() {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

function updateTargetAllocation() {
  // Force reactivity update
  positions.value = [...positions.value]
}

function useRecommendedAllocation() {
  // Apply recommended allocation based on selected strategy
  const recommended = [30, 30, 15, 15, 10] // Example percentages
  positions.value.forEach((position, index) => {
    position.targetAllocation = recommended[index] || 0
  })
}

function resetToCurrentAllocation() {
  positions.value.forEach(position => {
    position.targetAllocation = position.currentAllocation
  })
}

function executeRebalancing() {
  emit('execute', requiredTrades.value)
}
</script>

<style scoped>
/* Slider styling */
.slider::-webkit-slider-thumb {
  appearance: none;
  height: 16px;
  width: 16px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid #1e293b;
}

.slider::-moz-range-thumb {
  height: 16px;
  width: 16px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid #1e293b;
}

/* Step progress line animation */
.transition-all {
  transition: all 0.3s ease;
}

/* Card hover effects */
.cursor-pointer:hover {
  transform: translateY(-1px);
}
</style>