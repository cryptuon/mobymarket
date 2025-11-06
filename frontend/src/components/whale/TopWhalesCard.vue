<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="TrophyIcon" class="w-5 h-5 text-yellow-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Top Whales</h3>
            <p class="text-xs text-white/60">Ranked by 24h volume</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Time Range -->
          <select
            v-model="timeRange"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="24h">24H</option>
            <option value="7d">7D</option>
            <option value="30d">30D</option>
          </select>

          <!-- View Mode Toggle -->
          <button
            @click="viewMode = viewMode === 'grid' ? 'list' : 'grid'"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            :title="viewMode === 'grid' ? 'Switch to List View' : 'Switch to Grid View'"
          >
            <HeroIcon
              :name="viewMode === 'grid' ? 'ListBulletIcon' : 'Squares2X2Icon'"
              class="w-4 h-4 text-white/70"
            />
          </button>
        </div>
      </div>
    </template>

    <div class="space-y-3">
      <!-- Grid View -->
      <div v-if="viewMode === 'grid'" class="grid grid-cols-1 lg:grid-cols-2 gap-3">
        <div
          v-for="whale in whales"
          :key="whale.id"
          class="bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30 hover:border-slate-500/50 rounded-xl p-4 cursor-pointer transition-all duration-200 group"
          @click="selectWhale(whale)"
        >
          <!-- Rank Badge -->
          <div class="flex items-center justify-between mb-3">
            <div :class="getRankBadgeClass(whale.rank)" class="px-2 py-1 rounded-lg text-xs font-bold">
              #{{ whale.rank }}
            </div>
            <div class="flex items-center space-x-1 text-xs text-white/60">
              <HeroIcon name="ClockIcon" class="w-3 h-3" />
              <span>{{ whale.lastActive }}</span>
            </div>
          </div>

          <!-- Address & Info -->
          <div class="mb-3">
            <div class="flex items-center space-x-2 mb-1">
              <code class="text-white font-mono text-sm">{{ whale.address }}</code>
              <button
                @click.stop="copyAddress(whale.address)"
                class="p-1 hover:bg-white/10 rounded transition-colors"
              >
                <HeroIcon name="ClipboardIcon" class="w-3 h-3 text-white/50" />
              </button>
            </div>
            <div class="flex items-center space-x-3 text-xs text-white/60">
              <span>{{ whale.trades }} trades</span>
              <span class="flex items-center space-x-1">
                <div :class="getWinRateColor(whale.winRate)" class="w-2 h-2 rounded-full"></div>
                <span>{{ whale.winRate }}% win rate</span>
              </span>
            </div>
          </div>

          <!-- Volume & Metrics -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <div class="text-xs text-white/60">24h Volume</div>
              <div class="text-lg font-bold text-white">${{ formatCurrency(whale.volume24h) }}</div>
            </div>
            <div>
              <div class="text-xs text-white/60">Avg Size</div>
              <div class="text-lg font-bold text-white">${{ formatCurrency(whale.avgSize) }}</div>
            </div>
          </div>

          <!-- Activity Indicator -->
          <div class="mt-3 pt-3 border-t border-white/10">
            <div class="flex items-center justify-between">
              <div class="flex items-center space-x-2">
                <div :class="getActivityIndicator(whale.lastActive)" class="w-2 h-2 rounded-full"></div>
                <span class="text-xs text-white/60">{{ getActivityStatus(whale.lastActive) }}</span>
              </div>
              <Button
                variant="ghost"
                size="xs"
                icon-right="ArrowTopRightOnSquareIcon"
                @click.stop="viewWhaleDetails(whale)"
              >
                View
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- List View -->
      <div v-else class="space-y-2">
        <div
          v-for="whale in whales"
          :key="whale.id"
          class="flex items-center space-x-4 p-3 bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30 hover:border-slate-500/50 rounded-lg cursor-pointer transition-all duration-200 group"
          @click="selectWhale(whale)"
        >
          <!-- Rank -->
          <div :class="getRankBadgeClass(whale.rank)" class="px-2 py-1 rounded-lg text-xs font-bold flex-shrink-0">
            #{{ whale.rank }}
          </div>

          <!-- Address -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center space-x-2">
              <code class="text-white font-mono text-sm">{{ whale.address }}</code>
              <button
                @click.stop="copyAddress(whale.address)"
                class="p-1 hover:bg-white/10 rounded transition-colors"
              >
                <HeroIcon name="ClipboardIcon" class="w-3 h-3 text-white/50" />
              </button>
            </div>
            <div class="text-xs text-white/60">{{ whale.trades }} trades • {{ whale.lastActive }}</div>
          </div>

          <!-- Volume -->
          <div class="text-right flex-shrink-0">
            <div class="text-white font-semibold">${{ formatCurrency(whale.volume24h) }}</div>
            <div class="text-xs text-white/60">24h volume</div>
          </div>

          <!-- Win Rate -->
          <div class="text-right flex-shrink-0">
            <div class="flex items-center space-x-1">
              <div :class="getWinRateColor(whale.winRate)" class="w-2 h-2 rounded-full"></div>
              <span class="text-white font-semibold">{{ whale.winRate }}%</span>
            </div>
            <div class="text-xs text-white/60">win rate</div>
          </div>

          <!-- Action -->
          <Button
            variant="ghost"
            size="xs"
            icon-right="ArrowTopRightOnSquareIcon"
            @click.stop="viewWhaleDetails(whale)"
          >
            View
          </Button>
        </div>
      </div>

      <!-- Load More / Pagination -->
      <div v-if="hasMore" class="flex justify-center pt-4">
        <Button
          @click="loadMore"
          :loading="isLoading"
          variant="outline"
          size="sm"
          icon-left="ArrowDownIcon"
        >
          Load More Whales
        </Button>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <span>Showing top {{ whales.length }} whales</span>
        <div class="flex items-center space-x-2">
          <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
          <span>Live data</span>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useNotificationStore } from '@/stores/notifications'

interface WhaleData {
  id: string
  address: string
  volume24h: number
  trades: number
  winRate: number
  avgSize: number
  lastActive: string
  rank: number
}

interface Props {
  whales: WhaleData[]
  loading?: boolean
  hasMore?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  hasMore: false
})

const emit = defineEmits<{
  'whale-selected': [whale: WhaleData]
  'load-more': []
}>()

const notificationStore = useNotificationStore()

const timeRange = ref('24h')
const viewMode = ref<'grid' | 'list'>('grid')
const isLoading = ref(false)

// Methods
function getRankBadgeClass(rank: number): string {
  if (rank === 1) return 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30'
  if (rank === 2) return 'bg-gray-400/20 text-gray-300 border border-gray-400/30'
  if (rank === 3) return 'bg-orange-500/20 text-orange-400 border border-orange-500/30'
  if (rank <= 10) return 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
  return 'bg-slate-500/20 text-slate-400 border border-slate-500/30'
}

function getWinRateColor(winRate: number): string {
  if (winRate >= 80) return 'bg-green-400'
  if (winRate >= 60) return 'bg-yellow-400'
  return 'bg-red-400'
}

function getActivityIndicator(lastActive: string): string {
  const minutes = parseLastActive(lastActive)
  if (minutes <= 5) return 'bg-green-400 animate-pulse'
  if (minutes <= 30) return 'bg-yellow-400'
  if (minutes <= 60) return 'bg-orange-400'
  return 'bg-red-400'
}

function getActivityStatus(lastActive: string): string {
  const minutes = parseLastActive(lastActive)
  if (minutes <= 5) return 'Very Active'
  if (minutes <= 30) return 'Active'
  if (minutes <= 60) return 'Recently Active'
  return 'Inactive'
}

function parseLastActive(lastActive: string): number {
  if (lastActive.includes('min')) {
    return parseInt(lastActive.split(' ')[0])
  }
  if (lastActive.includes('hr')) {
    return parseInt(lastActive.split(' ')[0]) * 60
  }
  return 9999 // Assume very old
}

function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

async function copyAddress(address: string) {
  try {
    await navigator.clipboard.writeText(address)
    notificationStore.notifySystem(
      'Copied',
      'Wallet address copied to clipboard',
      'success'
    )
  } catch (error) {
    notificationStore.notifySystem(
      'Copy Failed',
      'Failed to copy wallet address',
      'error'
    )
  }
}

function selectWhale(whale: WhaleData) {
  emit('whale-selected', whale)
}

function viewWhaleDetails(whale: WhaleData) {
  // Could open a detailed modal or navigate to whale profile
  selectWhale(whale)
}

async function loadMore() {
  if (isLoading.value) return

  isLoading.value = true
  try {
    emit('load-more')
    // Simulate loading delay
    await new Promise(resolve => setTimeout(resolve, 500))
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
/* Custom scrollbar for long lists */
.space-y-2::-webkit-scrollbar {
  width: 4px;
}

.space-y-2::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.space-y-2::-webkit-scrollbar-thumb {
  background: rgba(14, 165, 233, 0.5);
  border-radius: 2px;
}

.space-y-2::-webkit-scrollbar-thumb:hover {
  background: rgba(14, 165, 233, 0.7);
}

/* Hover animations */
.group:hover .opacity-0 {
  opacity: 1;
}

.group:hover .scale-0 {
  transform: scale(1);
}
</style>