<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <div class="relative">
            <HeroIcon name="SparklesIcon" class="w-6 h-6 text-purple-400" />
            <div class="absolute -top-1 -right-1 w-3 h-3 bg-purple-400 rounded-full animate-pulse"></div>
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white">AI Insights</h3>
            <p class="text-xs text-white/60">Powered by advanced market analysis</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Insight Type Filter -->
          <select
            v-model="insightFilter"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="all">All Insights</option>
            <option value="opportunity">Opportunities</option>
            <option value="warning">Warnings</option>
            <option value="insight">Analysis</option>
            <option value="recommendation">Recommendations</option>
          </select>

          <!-- Refresh Button -->
          <button
            @click="$emit('refresh-insights')"
            :disabled="loading"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors disabled:opacity-50"
          >
            <HeroIcon
              name="ArrowPathIcon"
              class="w-4 h-4 text-white/70"
              :class="{ 'animate-spin': loading }"
            />
          </button>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- Loading State -->
      <div v-if="loading && filteredInsights.length === 0" class="flex items-center justify-center py-8">
        <div class="text-center">
          <div class="inline-flex items-center space-x-2 mb-4">
            <div class="animate-spin rounded-full h-6 w-6 border-2 border-purple-500/20 border-t-purple-400"></div>
            <HeroIcon name="CpuChipIcon" class="w-6 h-6 text-purple-400 animate-pulse" />
            <div class="animate-spin rounded-full h-6 w-6 border-2 border-purple-500/20 border-t-purple-400"></div>
          </div>
          <p class="text-white/60 text-sm">AI is analyzing market data...</p>
          <p class="text-white/40 text-xs mt-1">This may take up to 30 seconds</p>
        </div>
      </div>

      <!-- Insights List -->
      <div v-else class="space-y-3">
        <TransitionGroup
          name="insight-list"
          tag="div"
          class="space-y-3"
        >
          <div
            v-for="insight in filteredInsights"
            :key="insight.id"
            :class="getInsightCardClass(insight.type)"
            class="rounded-xl border p-4 transition-all duration-200 hover:scale-[1.02] cursor-pointer group"
            @click="toggleInsightDetails(insight.id)"
          >
            <!-- Insight Header -->
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-start space-x-3 flex-1">
                <!-- Type Icon -->
                <div :class="getInsightIconClass(insight.type)" class="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0">
                  <HeroIcon :name="getInsightIcon(insight.type)" class="w-5 h-5" />
                </div>

                <div class="flex-1 min-w-0">
                  <div class="flex items-center space-x-2 mb-1">
                    <h4 class="text-white font-semibold">{{ insight.title }}</h4>
                    <span :class="getInsightTypeClass(insight.type)" class="px-2 py-1 rounded-full text-xs font-medium">
                      {{ insight.type.toUpperCase() }}
                    </span>
                  </div>
                  <p class="text-white/70 text-sm">{{ insight.description }}</p>
                </div>
              </div>

              <!-- Confidence Score -->
              <div class="flex items-center space-x-2 flex-shrink-0">
                <div class="text-right">
                  <div class="text-xs text-white/60">Confidence</div>
                  <div :class="getConfidenceColor(insight.confidence)" class="text-sm font-bold">
                    {{ insight.confidence }}%
                  </div>
                </div>
                <div class="relative w-12 h-12">
                  <svg class="w-12 h-12 transform -rotate-90" viewBox="0 0 48 48">
                    <circle
                      cx="24"
                      cy="24"
                      r="20"
                      fill="none"
                      stroke="rgba(255,255,255,0.1)"
                      stroke-width="3"
                    />
                    <circle
                      cx="24"
                      cy="24"
                      r="20"
                      fill="none"
                      :stroke="getConfidenceStroke(insight.confidence)"
                      stroke-width="3"
                      :stroke-dasharray="`${(insight.confidence / 100) * 126} 126`"
                      stroke-linecap="round"
                      class="transition-all duration-500"
                    />
                  </svg>
                </div>
              </div>
            </div>

            <!-- Quick Metrics -->
            <div class="grid grid-cols-3 gap-4 mb-3">
              <div>
                <div class="text-xs text-white/60">Expected Return</div>
                <div :class="getReturnColor(insight.expectedReturn)" class="text-sm font-semibold">
                  {{ insight.expectedReturn >= 0 ? '+' : '' }}{{ insight.expectedReturn.toFixed(1) }}%
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60">Risk Level</div>
                <div :class="getRiskColor(insight.riskLevel)" class="text-sm font-semibold capitalize">
                  {{ insight.riskLevel }}
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60">Timeframe</div>
                <div class="text-sm font-semibold text-white">{{ insight.timeframe }}</div>
              </div>
            </div>

            <!-- Action Recommendation -->
            <div class="bg-white/5 rounded-lg p-3 mb-3">
              <div class="text-xs text-white/60 mb-1">Recommended Action</div>
              <div class="text-sm text-white font-medium">{{ insight.action }}</div>
            </div>

            <!-- Expanded Details -->
            <Transition
              name="details"
              enter-active-class="transition-all duration-200"
              enter-from-class="opacity-0 max-h-0"
              enter-to-class="opacity-100 max-h-96"
              leave-active-class="transition-all duration-150"
              leave-from-class="opacity-100 max-h-96"
              leave-to-class="opacity-0 max-h-0"
            >
              <div v-if="expandedInsights.has(insight.id)" class="pt-3 border-t border-white/10 space-y-3 overflow-hidden">
                <!-- Supporting Data -->
                <div>
                  <h5 class="text-sm font-semibold text-white mb-2">Supporting Analysis</h5>
                  <div class="space-y-2 text-sm text-white/70">
                    <div v-for="point in getSupportingData(insight.type)" :key="point" class="flex items-start space-x-2">
                      <HeroIcon name="CheckCircleIcon" class="w-4 h-4 text-green-400 mt-0.5 flex-shrink-0" />
                      <span>{{ point }}</span>
                    </div>
                  </div>
                </div>

                <!-- Risk Assessment -->
                <div v-if="insight.risks">
                  <h5 class="text-sm font-semibold text-white mb-2">Risk Factors</h5>
                  <div class="space-y-1 text-sm text-white/70">
                    <div v-for="risk in insight.risks" :key="risk" class="flex items-start space-x-2">
                      <HeroIcon name="ExclamationTriangleIcon" class="w-4 h-4 text-yellow-400 mt-0.5 flex-shrink-0" />
                      <span>{{ risk }}</span>
                    </div>
                  </div>
                </div>

                <!-- Historical Performance -->
                <div>
                  <h5 class="text-sm font-semibold text-white mb-2">Historical Accuracy</h5>
                  <div class="grid grid-cols-3 gap-3 text-xs">
                    <div class="text-center">
                      <div class="text-green-400 font-semibold">78%</div>
                      <div class="text-white/60">Similar Signals</div>
                    </div>
                    <div class="text-center">
                      <div class="text-blue-400 font-semibold">12.3%</div>
                      <div class="text-white/60">Avg Return</div>
                    </div>
                    <div class="text-center">
                      <div class="text-purple-400 font-semibold">5.2d</div>
                      <div class="text-white/60">Avg Duration</div>
                    </div>
                  </div>
                </div>
              </div>
            </Transition>

            <!-- Action Buttons -->
            <div class="flex items-center space-x-2 pt-3">
              <Button
                @click.stop="$emit('apply-suggestion', insight)"
                :variant="insight.type === 'opportunity' ? 'primary' : 'outline'"
                size="sm"
                :icon-left="getActionIcon(insight.type)"
              >
                {{ getActionText(insight.type) }}
              </Button>

              <Button
                @click.stop="dismissInsight(insight.id)"
                variant="ghost"
                size="sm"
                icon-left="XMarkIcon"
              >
                Dismiss
              </Button>

              <Button
                @click.stop="toggleInsightDetails(insight.id)"
                variant="ghost"
                size="sm"
                :icon-left="expandedInsights.has(insight.id) ? 'ChevronUpIcon' : 'ChevronDownIcon'"
              >
                {{ expandedInsights.has(insight.id) ? 'Less' : 'More' }}
              </Button>
            </div>
          </div>
        </TransitionGroup>

        <!-- Empty State -->
        <div v-if="filteredInsights.length === 0 && !loading" class="text-center py-8">
          <HeroIcon name="SparklesIcon" class="w-12 h-12 text-white/30 mx-auto mb-2" />
          <p class="text-white/60 text-sm">No {{ insightFilter === 'all' ? '' : insightFilter }} insights available</p>
          <p class="text-white/40 text-xs mt-1">AI is continuously analyzing market conditions</p>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <div class="flex items-center space-x-2">
          <HeroIcon name="ShieldCheckIcon" class="w-4 h-4" />
          <span>AI insights are suggestions only • Always DYOR</span>
        </div>
        <span>{{ filteredInsights.length }} insights</span>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useNotificationStore } from '@/stores/notifications'

interface AIInsight {
  id: string
  type: 'opportunity' | 'warning' | 'insight' | 'recommendation'
  title: string
  description: string
  confidence: number
  action: string
  expectedReturn: number
  riskLevel: 'low' | 'medium' | 'high'
  timeframe: string
  risks?: string[]
}

interface Props {
  insights: AIInsight[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'refresh-insights': []
  'apply-suggestion': [insight: AIInsight]
}>()

const notificationStore = useNotificationStore()

const insightFilter = ref<'all' | 'opportunity' | 'warning' | 'insight' | 'recommendation'>('all')
const expandedInsights = ref<Set<string>>(new Set())
const dismissedInsights = ref<Set<string>>(new Set())

// Computed properties
const filteredInsights = computed(() => {
  return props.insights.filter(insight => {
    if (dismissedInsights.value.has(insight.id)) return false
    if (insightFilter.value === 'all') return true
    return insight.type === insightFilter.value
  })
})

// Methods
function getInsightCardClass(type: string): string {
  const baseClass = 'bg-slate-800/30 hover:bg-slate-700/30'

  switch (type) {
    case 'opportunity':
      return `${baseClass} border-green-500/30 hover:border-green-500/50`
    case 'warning':
      return `${baseClass} border-red-500/30 hover:border-red-500/50`
    case 'insight':
      return `${baseClass} border-blue-500/30 hover:border-blue-500/50`
    case 'recommendation':
      return `${baseClass} border-purple-500/30 hover:border-purple-500/50`
    default:
      return `${baseClass} border-slate-600/30 hover:border-slate-500/50`
  }
}

function getInsightIconClass(type: string): string {
  switch (type) {
    case 'opportunity':
      return 'bg-green-500/20 text-green-400'
    case 'warning':
      return 'bg-red-500/20 text-red-400'
    case 'insight':
      return 'bg-blue-500/20 text-blue-400'
    case 'recommendation':
      return 'bg-purple-500/20 text-purple-400'
    default:
      return 'bg-white/10 text-white/70'
  }
}

function getInsightIcon(type: string): string {
  switch (type) {
    case 'opportunity':
      return 'TrendingUpIcon'
    case 'warning':
      return 'ExclamationTriangleIcon'
    case 'insight':
      return 'LightBulbIcon'
    case 'recommendation':
      return 'SparklesIcon'
    default:
      return 'InformationCircleIcon'
  }
}

function getInsightTypeClass(type: string): string {
  switch (type) {
    case 'opportunity':
      return 'bg-green-500/20 text-green-400'
    case 'warning':
      return 'bg-red-500/20 text-red-400'
    case 'insight':
      return 'bg-blue-500/20 text-blue-400'
    case 'recommendation':
      return 'bg-purple-500/20 text-purple-400'
    default:
      return 'bg-white/10 text-white/70'
  }
}

function getConfidenceColor(confidence: number): string {
  if (confidence >= 80) return 'text-green-400'
  if (confidence >= 60) return 'text-yellow-400'
  return 'text-red-400'
}

function getConfidenceStroke(confidence: number): string {
  if (confidence >= 80) return '#4ade80'
  if (confidence >= 60) return '#facc15'
  return '#f87171'
}

function getReturnColor(expectedReturn: number): string {
  return expectedReturn >= 0 ? 'text-green-400' : 'text-red-400'
}

function getRiskColor(riskLevel: string): string {
  switch (riskLevel) {
    case 'low':
      return 'text-green-400'
    case 'medium':
      return 'text-yellow-400'
    case 'high':
      return 'text-red-400'
    default:
      return 'text-white/70'
  }
}

function getActionIcon(type: string): string {
  switch (type) {
    case 'opportunity':
      return 'RocketLaunchIcon'
    case 'warning':
      return 'ShieldExclamationIcon'
    case 'insight':
      return 'EyeIcon'
    case 'recommendation':
      return 'CheckCircleIcon'
    default:
      return 'ArrowRightIcon'
  }
}

function getActionText(type: string): string {
  switch (type) {
    case 'opportunity':
      return 'Execute'
    case 'warning':
      return 'Review'
    case 'insight':
      return 'Analyze'
    case 'recommendation':
      return 'Apply'
    default:
      return 'View'
  }
}

function getSupportingData(type: string): string[] {
  const mockData: Record<string, string[]> = {
    opportunity: [
      'Whale accumulation increased 45% in last 24h',
      'Technical indicators show strong bullish divergence',
      'On-chain metrics suggest undervaluation',
      'Correlation with BTC at historical lows'
    ],
    warning: [
      'Position concentration exceeds risk limits',
      'Market volatility above 95th percentile',
      'Correlation between assets increasing',
      'Liquidity depth decreased 30% today'
    ],
    insight: [
      'DeFi TVL showing seasonal patterns',
      'Gas fees indicate network congestion',
      'Staking rewards optimally distributed',
      'Portfolio beta within target range'
    ],
    recommendation: [
      'Rebalancing would improve Sharpe ratio',
      'Current allocation deviates from targets',
      'Tax-loss harvesting opportunities available',
      'Yield farming APY exceeds benchmarks'
    ]
  }

  return mockData[type] || ['No supporting data available']
}

function toggleInsightDetails(id: string) {
  if (expandedInsights.value.has(id)) {
    expandedInsights.value.delete(id)
  } else {
    expandedInsights.value.add(id)
  }
}

function dismissInsight(id: string) {
  dismissedInsights.value.add(id)
  expandedInsights.value.delete(id)

  notificationStore.notifySystem(
    'Insight Dismissed',
    'AI insight has been removed from your feed',
    'info'
  )
}
</script>

<style scoped>
/* Insight list animations */
.insight-list-enter-active {
  transition: all 0.3s ease-out;
}

.insight-list-enter-from {
  transform: translateY(-20px) scale(0.95);
  opacity: 0;
}

.insight-list-leave-active {
  transition: all 0.2s ease-in;
}

.insight-list-leave-to {
  transform: translateY(20px) scale(0.95);
  opacity: 0;
}

.insight-list-move {
  transition: transform 0.3s ease;
}

/* Details expansion animation */
.details-enter-active,
.details-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.details-enter-from,
.details-leave-to {
  opacity: 0;
  max-height: 0;
}

.details-enter-to,
.details-leave-from {
  opacity: 1;
  max-height: 24rem;
}

/* Hover scale effect */
.hover\:scale-\[1\.02\]:hover {
  transform: scale(1.02);
}

/* Confidence circle animation */
circle {
  transition: stroke-dasharray 0.5s ease;
}
</style>