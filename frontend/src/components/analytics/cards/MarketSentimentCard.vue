<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="FaceSmileIcon" class="w-5 h-5 text-yellow-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Market Sentiment</h3>
            <p class="text-xs text-white/60">Crypto market mood analysis</p>
          </div>
        </div>

        <div :class="getSentimentBadgeClass()" class="px-3 py-1 rounded-lg text-xs font-medium">
          {{ getSentimentLabel() }}
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Loading State -->
      <div v-if="loading" class="space-y-4">
        <div class="animate-pulse">
          <div class="h-32 bg-slate-700/30 rounded-lg"></div>
        </div>
        <div class="grid grid-cols-3 gap-4">
          <div v-for="i in 3" :key="i" class="animate-pulse">
            <div class="h-16 bg-slate-700/30 rounded-lg"></div>
          </div>
        </div>
      </div>

      <div v-else class="space-y-6">
        <!-- Sentiment Gauge -->
        <div class="flex items-center justify-center">
          <div class="relative w-40 h-40">
            <svg class="w-40 h-40 transform -rotate-90" viewBox="0 0 160 160">
              <!-- Background Arc -->
              <path
                d="M 30 80 A 50 50 0 0 1 130 80"
                fill="none"
                stroke="rgba(255,255,255,0.1)"
                stroke-width="12"
                stroke-linecap="round"
              />

              <!-- Sentiment Arc -->
              <path
                d="M 30 80 A 50 50 0 0 1 130 80"
                fill="none"
                :stroke="getSentimentColor(data.overall)"
                stroke-width="12"
                :stroke-dasharray="`${(data.overall / 100) * 157} 157`"
                stroke-linecap="round"
                class="transition-all duration-1000"
              />

              <!-- Needle -->
              <line
                :x1="80"
                :y1="80"
                :x2="80 + 40 * Math.cos((data.overall / 100) * Math.PI - Math.PI)"
                :y2="80 + 40 * Math.sin((data.overall / 100) * Math.PI - Math.PI)"
                stroke="white"
                stroke-width="2"
                stroke-linecap="round"
                class="transition-all duration-1000"
              />
              <circle cx="80" cy="80" r="4" fill="white" />
            </svg>

            <!-- Center Content -->
            <div class="absolute inset-0 flex items-center justify-center">
              <div class="text-center mt-8">
                <div :class="['text-3xl font-bold', getSentimentTextColor(data.overall)]">
                  {{ data.overall }}
                </div>
                <div class="text-xs text-white/60">Sentiment Score</div>
              </div>
            </div>

            <!-- Gauge Labels -->
            <div class="absolute bottom-4 left-0 text-xs text-red-400 font-medium">Fear</div>
            <div class="absolute bottom-4 right-0 text-xs text-green-400 font-medium">Greed</div>
            <div class="absolute bottom-0 left-1/2 transform -translate-x-1/2 text-xs text-yellow-400 font-medium">
              Neutral
            </div>
          </div>
        </div>

        <!-- Sentiment Indicators -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="NewspaperIcon" class="w-4 h-4 text-blue-400" />
              <span class="text-xs text-white/60">News Sentiment</span>
            </div>
            <div :class="['text-lg font-bold', getSentimentTextColor(data.news)]">
              {{ data.news }}
            </div>
            <div class="text-xs text-white/60">{{ getSentimentLabel(data.news) }}</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="ChatBubbleLeftRightIcon" class="w-4 h-4 text-purple-400" />
              <span class="text-xs text-white/60">Social Media</span>
            </div>
            <div :class="['text-lg font-bold', getSentimentTextColor(data.social)]">
              {{ data.social }}
            </div>
            <div class="text-xs text-white/60">{{ getSentimentLabel(data.social) }}</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="CurrencyDollarIcon" class="w-4 h-4 text-green-400" />
              <span class="text-xs text-white/60">On-Chain</span>
            </div>
            <div :class="['text-lg font-bold', getSentimentTextColor(data.onChain)]">
              {{ data.onChain }}
            </div>
            <div class="text-xs text-white/60">{{ getSentimentLabel(data.onChain) }}</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-orange-400" />
              <span class="text-xs text-white/60">Technical</span>
            </div>
            <div :class="['text-lg font-bold', getSentimentTextColor(data.technical)]">
              {{ data.technical }}
            </div>
            <div class="text-xs text-white/60">{{ getSentimentLabel(data.technical) }}</div>
          </div>
        </div>

        <!-- Sentiment Breakdown -->
        <div class="space-y-4">
          <h4 class="text-sm font-semibold text-white">Sentiment Drivers</h4>
          <div class="space-y-3">
            <div
              v-for="driver in sentimentDrivers"
              :key="driver.name"
              class="flex items-center justify-between"
            >
              <div class="flex items-center space-x-3">
                <HeroIcon :name="driver.icon" :class="['w-4 h-4', driver.iconColor]" />
                <span class="text-sm text-white/70">{{ driver.name }}</span>
              </div>

              <div class="flex items-center space-x-3">
                <div class="w-24 h-2 bg-slate-700/50 rounded-full overflow-hidden">
                  <div
                    :class="getSentimentBarClass(driver.value)"
                    :style="{ width: `${driver.value}%` }"
                    class="h-full transition-all duration-500"
                  ></div>
                </div>
                <span :class="['text-sm font-medium w-8 text-right', getSentimentTextColor(driver.value)]">
                  {{ driver.value }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Fear & Greed Components -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div class="bg-slate-800/20 rounded-lg p-4">
            <h4 class="text-sm font-semibold text-white mb-3 flex items-center space-x-2">
              <HeroIcon name="ExclamationTriangleIcon" class="w-4 h-4 text-red-400" />
              <span>Fear Indicators</span>
            </h4>
            <div class="space-y-2">
              <div
                v-for="indicator in fearIndicators"
                :key="indicator.name"
                class="flex items-center justify-between py-2 border-b border-white/10 last:border-b-0"
              >
                <span class="text-sm text-white/70">{{ indicator.name }}</span>
                <div class="flex items-center space-x-2">
                  <div :class="['text-sm font-medium', indicator.trend === 'up' ? 'text-red-400' : 'text-green-400']">
                    {{ indicator.value }}
                  </div>
                  <HeroIcon
                    :name="indicator.trend === 'up' ? 'ArrowTrendingUpIcon' : 'ArrowTrendingDownIcon'"
                    :class="['w-3 h-3', indicator.trend === 'up' ? 'text-red-400' : 'text-green-400']"
                  />
                </div>
              </div>
            </div>
          </div>

          <div class="bg-slate-800/20 rounded-lg p-4">
            <h4 class="text-sm font-semibold text-white mb-3 flex items-center space-x-2">
              <HeroIcon name="SparklesIcon" class="w-4 h-4 text-green-400" />
              <span>Greed Indicators</span>
            </h4>
            <div class="space-y-2">
              <div
                v-for="indicator in greedIndicators"
                :key="indicator.name"
                class="flex items-center justify-between py-2 border-b border-white/10 last:border-b-0"
              >
                <span class="text-sm text-white/70">{{ indicator.name }}</span>
                <div class="flex items-center space-x-2">
                  <div :class="['text-sm font-medium', indicator.trend === 'up' ? 'text-green-400' : 'text-red-400']">
                    {{ indicator.value }}
                  </div>
                  <HeroIcon
                    :name="indicator.trend === 'up' ? 'ArrowTrendingUpIcon' : 'ArrowTrendingDownIcon'"
                    :class="['w-3 h-3', indicator.trend === 'up' ? 'text-green-400' : 'text-red-400']"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Market Events Impact -->
        <div class="space-y-3">
          <h4 class="text-sm font-semibold text-white">Recent Market Events</h4>
          <div class="space-y-2">
            <div
              v-for="event in marketEvents"
              :key="event.id"
              class="flex items-start space-x-3 p-3 bg-slate-800/30 rounded-lg"
            >
              <HeroIcon
                :name="event.icon"
                :class="['w-4 h-4 mt-0.5 flex-shrink-0', event.iconColor]"
              />
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between">
                  <span class="text-sm font-medium text-white">{{ event.title }}</span>
                  <span :class="['text-xs px-2 py-1 rounded', getSentimentImpactClass(event.impact)]">
                    {{ event.impact > 0 ? '+' : '' }}{{ event.impact }}
                  </span>
                </div>
                <p class="text-xs text-white/70 mt-1">{{ event.description }}</p>
                <div class="text-xs text-white/50 mt-1">{{ formatTimeAgo(event.timestamp) }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Sentiment History -->
        <div class="space-y-3">
          <h4 class="text-sm font-semibold text-white">7-Day Sentiment Trend</h4>
          <div class="flex items-end space-x-1 h-16">
            <div
              v-for="(day, index) in sentimentHistory"
              :key="index"
              class="flex-1 flex flex-col items-center"
            >
              <div
                :class="getSentimentBarClass(day.sentiment)"
                :style="{ height: `${day.sentiment}%` }"
                class="w-full rounded-sm min-h-1 transition-all duration-300"
              ></div>
              <div class="text-xs text-white/60 mt-1">{{ day.day }}</div>
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
          @click="$emit('detailed-sentiment')"
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

interface MarketEvent {
  id: string
  title: string
  description: string
  impact: number
  timestamp: string
  icon: string
  iconColor: string
}

interface SentimentData {
  overall: number
  news: number
  social: number
  onChain: number
  technical: number
  lastUpdated: string
}

interface Props {
  data: SentimentData
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  data: () => ({
    overall: 65,
    news: 70,
    social: 62,
    onChain: 68,
    technical: 61,
    lastUpdated: new Date().toISOString()
  })
})

const emit = defineEmits<{
  'detailed-sentiment': []
}>()

// Sentiment drivers breakdown
const sentimentDrivers = computed(() => [
  {
    name: 'Institutional Flow',
    value: 72,
    icon: 'BuildingOffice2Icon',
    iconColor: 'text-blue-400'
  },
  {
    name: 'Whale Activity',
    value: 58,
    icon: 'UserGroupIcon',
    iconColor: 'text-purple-400'
  },
  {
    name: 'DEX Volume',
    value: 81,
    icon: 'ArrowsRightLeftIcon',
    iconColor: 'text-green-400'
  },
  {
    name: 'Options Flow',
    value: 44,
    icon: 'ChartBarIcon',
    iconColor: 'text-orange-400'
  },
  {
    name: 'Stablecoin Supply',
    value: 69,
    icon: 'CurrencyDollarIcon',
    iconColor: 'text-yellow-400'
  }
])

// Fear indicators
const fearIndicators = computed(() => [
  { name: 'VIX Equivalent', value: '28.5', trend: 'up' },
  { name: 'Put/Call Ratio', value: '1.34', trend: 'up' },
  { name: 'Funding Rates', value: '-0.05%', trend: 'down' },
  { name: 'Market Cap/TVL', value: '0.82', trend: 'down' }
])

// Greed indicators
const greedIndicators = computed(() => [
  { name: 'ETF Inflows', value: '$1.2B', trend: 'up' },
  { name: 'Active Addresses', value: '985K', trend: 'up' },
  { name: 'Social Volume', value: '156%', trend: 'up' },
  { name: 'Google Trends', value: '89', trend: 'up' }
])

// Market events
const marketEvents = computed(() => [
  {
    id: '1',
    title: 'Fed Rate Decision',
    description: 'Federal Reserve maintained current rates, signaling dovish stance',
    impact: 8,
    timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    icon: 'BanknotesIcon',
    iconColor: 'text-green-400'
  },
  {
    id: '2',
    title: 'Large ETH Withdrawal',
    description: '50K ETH moved from Coinbase to unknown wallet',
    impact: -3,
    timestamp: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(),
    icon: 'ArrowRightOnRectangleIcon',
    iconColor: 'text-yellow-400'
  },
  {
    id: '3',
    title: 'DeFi TVL Surge',
    description: 'Total value locked increased by 12% this week',
    impact: 5,
    timestamp: new Date(Date.now() - 6 * 60 * 60 * 1000).toISOString(),
    icon: 'TrendingUpIcon',
    iconColor: 'text-blue-400'
  }
])

// Sentiment history (7 days)
const sentimentHistory = computed(() => {
  const days = ['M', 'T', 'W', 'T', 'F', 'S', 'S']
  return days.map(day => ({
    day,
    sentiment: 40 + Math.random() * 40 // Random sentiment between 40-80
  }))
})

// Methods
function getSentimentLabel(score?: number): string {
  const sentiment = score || props.data.overall
  if (sentiment >= 75) return 'Extreme Greed'
  if (sentiment >= 55) return 'Greed'
  if (sentiment >= 45) return 'Neutral'
  if (sentiment >= 25) return 'Fear'
  return 'Extreme Fear'
}

function getSentimentColor(score: number): string {
  if (score >= 75) return '#22c55e'
  if (score >= 55) return '#84cc16'
  if (score >= 45) return '#fbbf24'
  if (score >= 25) return '#f97316'
  return '#ef4444'
}

function getSentimentTextColor(score: number): string {
  if (score >= 75) return 'text-green-400'
  if (score >= 55) return 'text-lime-400'
  if (score >= 45) return 'text-yellow-400'
  if (score >= 25) return 'text-orange-400'
  return 'text-red-400'
}

function getSentimentBadgeClass(): string {
  const score = props.data.overall
  if (score >= 75) return 'bg-green-500/20 text-green-400'
  if (score >= 55) return 'bg-lime-500/20 text-lime-400'
  if (score >= 45) return 'bg-yellow-500/20 text-yellow-400'
  if (score >= 25) return 'bg-orange-500/20 text-orange-400'
  return 'bg-red-500/20 text-red-400'
}

function getSentimentBarClass(score: number): string {
  if (score >= 75) return 'bg-green-400'
  if (score >= 55) return 'bg-lime-400'
  if (score >= 45) return 'bg-yellow-400'
  if (score >= 25) return 'bg-orange-400'
  return 'bg-red-400'
}

function getSentimentImpactClass(impact: number): string {
  if (impact > 5) return 'bg-green-500/20 text-green-400'
  if (impact > 0) return 'bg-blue-500/20 text-blue-400'
  if (impact > -5) return 'bg-yellow-500/20 text-yellow-400'
  return 'bg-red-500/20 text-red-400'
}

function formatTimeAgo(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`
  return date.toLocaleDateString()
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
</script>

<style scoped>
/* Gauge animations */
path {
  transition: stroke-dasharray 1s ease-out;
}

line {
  transition: all 1s ease-out;
}

/* Bar animations */
.transition-all {
  transition: all 0.5s ease;
}

/* Event cards */
.bg-slate-800\/30:hover {
  background-color: rgba(30, 41, 59, 0.4);
}
</style>