<template>
  <Teleport to="body">
    <Transition
      name="modal"
      enter-active-class="transition-all duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-all duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="activity"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
        @click="handleBackdropClick"
      >
        <Transition
          name="modal-content"
          enter-active-class="transition-all duration-300"
          enter-from-class="opacity-0 scale-95 translate-y-4"
          enter-to-class="opacity-100 scale-100 translate-y-0"
          leave-active-class="transition-all duration-200"
          leave-from-class="opacity-100 scale-100 translate-y-0"
          leave-to-class="opacity-0 scale-95 translate-y-4"
        >
          <div
            v-if="activity"
            class="bg-slate-900/95 backdrop-blur-xl border border-white/20 rounded-2xl shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-hidden"
            @click.stop
          >
            <!-- Header -->
            <div class="flex items-center justify-between p-6 border-b border-white/10">
              <div class="flex items-center space-x-4">
                <!-- Activity Type Icon -->
                <div :class="typeIconBackgroundClass" class="w-12 h-12 rounded-xl flex items-center justify-center">
                  <HeroIcon :name="typeIcon" class="w-6 h-6" :class="typeIconColorClass" />
                </div>

                <div>
                  <h2 class="text-2xl font-bold text-white">{{ activity.amount }} {{ activity.token }}</h2>
                  <div class="flex items-center space-x-2 mt-1">
                    <span :class="typeTextClass" class="text-xs font-medium px-2 py-1 rounded-full">
                      {{ activityTypeText }}
                    </span>
                    <span class="text-sm text-white/60">{{ formatTimestamp(activity.timestamp) }}</span>
                  </div>
                </div>
              </div>

              <div class="flex items-center space-x-2">
                <button
                  @click="toggleWatchlist"
                  :class="[
                    'p-2 rounded-lg transition-colors',
                    isWatched ? 'bg-yellow-500/20 text-yellow-400' : 'hover:bg-white/10 text-white/70'
                  ]"
                  :title="isWatched ? 'Remove from watchlist' : 'Add to watchlist'"
                >
                  <HeroIcon :name="isWatched ? 'StarIcon' : 'StarIcon'" class="w-5 h-5" :fill="isWatched ? 'currentColor' : 'none'" />
                </button>

                <button
                  @click="$emit('close')"
                  class="p-2 hover:bg-white/10 rounded-lg transition-colors"
                >
                  <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
                </button>
              </div>
            </div>

            <!-- Content -->
            <div class="p-6 overflow-y-auto max-h-[calc(90vh-200px)]">
              <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <!-- Main Details -->
                <div class="lg:col-span-2 space-y-6">
                  <!-- Key Metrics -->
                  <div class="grid grid-cols-2 gap-4">
                    <div class="bg-slate-800/50 rounded-xl p-4">
                      <div class="text-xs text-white/60 mb-1">USD Value</div>
                      <div class="text-2xl font-bold text-white">${{ formatValue(activity.usdValue) }}</div>
                      <div v-if="activity.priceAtTime" class="text-xs text-white/60 mt-1">
                        @ ${{ activity.priceAtTime.toLocaleString() }} per {{ activity.token }}
                      </div>
                    </div>

                    <div class="bg-slate-800/50 rounded-xl p-4">
                      <div class="text-xs text-white/60 mb-1">Wallet Address</div>
                      <div class="flex items-center space-x-2">
                        <code class="text-sm font-mono text-white">{{ formatAddress(activity.address) }}</code>
                        <button
                          @click="copyAddress"
                          class="p-1 hover:bg-white/10 rounded transition-colors"
                        >
                          <HeroIcon name="ClipboardIcon" class="w-4 h-4 text-white/60" />
                        </button>
                      </div>
                      <div class="flex items-center space-x-2 mt-2">
                        <Button
                          @click="viewWalletDetails"
                          variant="ghost"
                          size="xs"
                          icon-left="EyeIcon"
                        >
                          View Wallet
                        </Button>
                        <Button
                          @click="viewOnExplorer"
                          variant="ghost"
                          size="xs"
                          icon-left="ArrowTopRightOnSquareIcon"
                        >
                          Explorer
                        </Button>
                      </div>
                    </div>
                  </div>

                  <!-- Transaction Details -->
                  <Card variant="glass">
                    <template #header>
                      <h3 class="text-lg font-semibold text-white">Transaction Details</h3>
                    </template>

                    <div class="space-y-4">
                      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                          <div class="text-xs text-white/60">Transaction Hash</div>
                          <div class="flex items-center space-x-2 mt-1">
                            <code class="text-sm font-mono text-white break-all">{{ activity.txHash }}</code>
                            <button
                              @click="copyTxHash"
                              class="p-1 hover:bg-white/10 rounded transition-colors flex-shrink-0"
                            >
                              <HeroIcon name="ClipboardIcon" class="w-4 h-4 text-white/60" />
                            </button>
                          </div>
                        </div>

                        <div v-if="activity.blockNumber">
                          <div class="text-xs text-white/60">Block Number</div>
                          <div class="text-sm text-white mt-1">{{ activity.blockNumber.toLocaleString() }}</div>
                        </div>

                        <div v-if="activity.gasUsed">
                          <div class="text-xs text-white/60">Gas Used</div>
                          <div class="text-sm text-white mt-1">{{ activity.gasUsed.toLocaleString() }}</div>
                        </div>

                        <div v-if="activity.gasPrice">
                          <div class="text-xs text-white/60">Gas Price</div>
                          <div class="text-sm text-white mt-1">{{ activity.gasPrice.toFixed(2) }} Gwei</div>
                        </div>
                      </div>

                      <!-- DEX/Exchange Information -->
                      <div v-if="activity.dex || activity.exchange" class="pt-4 border-t border-white/10">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                          <div v-if="activity.dex">
                            <div class="text-xs text-white/60">DEX/Protocol</div>
                            <div class="text-sm text-white mt-1">{{ activity.dex }}</div>
                          </div>

                          <div v-if="activity.priceImpact">
                            <div class="text-xs text-white/60">Price Impact</div>
                            <div :class="[
                              'text-sm font-medium mt-1',
                              activity.priceImpact > 5 ? 'text-red-400' :
                              activity.priceImpact > 2 ? 'text-yellow-400' : 'text-green-400'
                            ]">
                              {{ activity.priceImpact.toFixed(2) }}%
                            </div>
                          </div>

                          <div v-if="activity.slippage">
                            <div class="text-xs text-white/60">Slippage</div>
                            <div class="text-sm text-white mt-1">{{ activity.slippage.toFixed(2) }}%</div>
                          </div>

                          <div v-if="activity.strategy">
                            <div class="text-xs text-white/60">Strategy</div>
                            <div class="text-sm text-white mt-1">{{ activity.strategy }}</div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </Card>

                  <!-- Market Impact Analysis -->
                  <Card v-if="showMarketImpact" variant="glass">
                    <template #header>
                      <h3 class="text-lg font-semibold text-white">Market Impact Analysis</h3>
                    </template>

                    <div class="space-y-4">
                      <div class="grid grid-cols-3 gap-4">
                        <div class="text-center">
                          <div class="text-xs text-white/60">Price Before</div>
                          <div class="text-lg font-semibold text-white mt-1">
                            ${{ activity.priceAtTime?.toFixed(2) || 'N/A' }}
                          </div>
                        </div>

                        <div class="text-center">
                          <div class="text-xs text-white/60">Price After</div>
                          <div class="text-lg font-semibold text-white mt-1">
                            ${{ marketImpact.priceAfter.toFixed(2) }}
                          </div>
                        </div>

                        <div class="text-center">
                          <div class="text-xs text-white/60">Impact</div>
                          <div :class="[
                            'text-lg font-semibold mt-1',
                            marketImpact.impact >= 0 ? 'text-green-400' : 'text-red-400'
                          ]">
                            {{ marketImpact.impact >= 0 ? '+' : '' }}{{ marketImpact.impact.toFixed(2) }}%
                          </div>
                        </div>
                      </div>

                      <!-- Volume Impact Chart -->
                      <div class="h-32 bg-slate-800/30 rounded-lg p-3">
                        <div class="text-xs text-white/60 mb-2">Volume Impact Timeline</div>
                        <div class="h-20 flex items-end space-x-1">
                          <div
                            v-for="(bar, index) in marketImpact.volumeBars"
                            :key="index"
                            :class="bar.color"
                            :style="{ height: `${bar.height}%` }"
                            class="flex-1 rounded-sm min-h-1"
                          ></div>
                        </div>
                      </div>
                    </div>
                  </Card>
                </div>

                <!-- Sidebar -->
                <div class="space-y-6">
                  <!-- Related Activities -->
                  <Card variant="glass">
                    <template #header>
                      <h3 class="text-lg font-semibold text-white">Related Activities</h3>
                    </template>

                    <div class="space-y-3">
                      <div
                        v-for="related in relatedActivities"
                        :key="related.id"
                        class="flex items-center space-x-3 p-3 bg-slate-800/30 rounded-lg hover:bg-slate-700/30 transition-colors cursor-pointer"
                        @click="selectRelatedActivity(related)"
                      >
                        <div :class="getActivityTypeColor(related.type)" class="w-8 h-8 rounded-lg flex items-center justify-center">
                          <HeroIcon :name="getActivityTypeIcon(related.type)" class="w-4 h-4" />
                        </div>
                        <div class="flex-1 min-w-0">
                          <div class="text-sm font-medium text-white">{{ related.amount }} {{ related.token }}</div>
                          <div class="text-xs text-white/60">{{ formatTimestamp(related.timestamp) }}</div>
                        </div>
                        <div class="text-xs text-white/60">${{ formatValue(related.usdValue) }}</div>
                      </div>
                    </div>

                    <template #footer>
                      <Button
                        @click="viewAllActivities"
                        variant="ghost"
                        size="sm"
                        class="w-full"
                        icon-left="EyeIcon"
                      >
                        View All Activities
                      </Button>
                    </template>
                  </Card>

                  <!-- Whale Profile -->
                  <Card variant="glass">
                    <template #header>
                      <h3 class="text-lg font-semibold text-white">Whale Profile</h3>
                    </template>

                    <div class="space-y-4">
                      <div class="grid grid-cols-2 gap-3">
                        <div>
                          <div class="text-xs text-white/60">Total Volume</div>
                          <div class="text-lg font-semibold text-white">${{ formatValue(whaleProfile.totalVolume) }}</div>
                        </div>
                        <div>
                          <div class="text-xs text-white/60">Win Rate</div>
                          <div :class="[
                            'text-lg font-semibold',
                            whaleProfile.winRate >= 70 ? 'text-green-400' :
                            whaleProfile.winRate >= 50 ? 'text-yellow-400' : 'text-red-400'
                          ]">
                            {{ whaleProfile.winRate }}%
                          </div>
                        </div>
                        <div>
                          <div class="text-xs text-white/60">Trade Count</div>
                          <div class="text-lg font-semibold text-white">{{ whaleProfile.tradeCount }}</div>
                        </div>
                        <div>
                          <div class="text-xs text-white/60">Avg Size</div>
                          <div class="text-lg font-semibold text-white">${{ formatValue(whaleProfile.avgTradeSize) }}</div>
                        </div>
                      </div>

                      <!-- Activity Heatmap -->
                      <div>
                        <div class="text-xs text-white/60 mb-2">Activity Pattern</div>
                        <div class="grid grid-cols-7 gap-1">
                          <div
                            v-for="(day, index) in whaleProfile.activityHeatmap"
                            :key="index"
                            :class="getHeatmapColor(day)"
                            class="aspect-square rounded-sm"
                            :title="`${day} activities`"
                          ></div>
                        </div>
                      </div>
                    </div>

                    <template #footer>
                      <Button
                        @click="viewWhaleProfile"
                        variant="primary"
                        size="sm"
                        class="w-full"
                        icon-left="UserIcon"
                      >
                        View Full Profile
                      </Button>
                    </template>
                  </Card>

                  <!-- Quick Actions -->
                  <Card variant="glass">
                    <template #header>
                      <h3 class="text-lg font-semibold text-white">Quick Actions</h3>
                    </template>

                    <div class="space-y-3">
                      <Button
                        @click="followTrade"
                        variant="primary"
                        size="sm"
                        class="w-full"
                        icon-left="ArrowPathIcon"
                      >
                        Copy Trade
                      </Button>

                      <Button
                        @click="setAlert"
                        variant="outline"
                        size="sm"
                        class="w-full"
                        icon-left="BellIcon"
                      >
                        Set Alert
                      </Button>

                      <Button
                        @click="addToWatchlist"
                        variant="ghost"
                        size="sm"
                        class="w-full"
                        icon-left="BookmarkIcon"
                      >
                        Add to Watchlist
                      </Button>
                    </div>
                  </Card>
                </div>
              </div>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useNotificationStore } from '@/stores/notifications'
import type { WhaleActivity } from '@/types'

interface Props {
  activity: WhaleActivity | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  'activity-selected': [activity: WhaleActivity]
}>()

const notificationStore = useNotificationStore()

const isWatched = ref(false)
const showMarketImpact = ref(true)

// Mock data for demo
const marketImpact = ref({
  priceAfter: 3250.75,
  impact: 2.3,
  volumeBars: Array.from({ length: 20 }, (_, i) => ({
    height: Math.random() * 100,
    color: Math.random() > 0.5 ? 'bg-green-500/60' : 'bg-red-500/60'
  }))
})

const relatedActivities = ref([
  {
    id: '2',
    type: 'buy',
    amount: '850',
    token: 'ETH',
    usdValue: 2720000,
    timestamp: new Date(Date.now() - 1800000).toISOString()
  },
  {
    id: '3',
    type: 'sell',
    amount: '425',
    token: 'ETH',
    usdValue: 1360000,
    timestamp: new Date(Date.now() - 3600000).toISOString()
  }
])

const whaleProfile = ref({
  totalVolume: 125000000,
  winRate: 78.5,
  tradeCount: 156,
  avgTradeSize: 8012820,
  activityHeatmap: Array.from({ length: 28 }, () => Math.floor(Math.random() * 10))
})

// Computed properties
const typeIcon = computed(() => {
  if (!props.activity) return 'CurrencyDollarIcon'
  switch (props.activity.type) {
    case 'buy': return 'ArrowTrendingUpIcon'
    case 'sell': return 'ArrowTrendingDownIcon'
    case 'transfer': return 'ArrowsRightLeftIcon'
    default: return 'CurrencyDollarIcon'
  }
})

const typeIconBackgroundClass = computed(() => {
  if (!props.activity) return 'bg-white/10'
  switch (props.activity.type) {
    case 'buy': return 'bg-green-500/20'
    case 'sell': return 'bg-red-500/20'
    case 'transfer': return 'bg-blue-500/20'
    default: return 'bg-white/10'
  }
})

const typeIconColorClass = computed(() => {
  if (!props.activity) return 'text-white/70'
  switch (props.activity.type) {
    case 'buy': return 'text-green-400'
    case 'sell': return 'text-red-400'
    case 'transfer': return 'text-blue-400'
    default: return 'text-white/70'
  }
})

const typeTextClass = computed(() => {
  if (!props.activity) return 'bg-white/10 text-white/70'
  switch (props.activity.type) {
    case 'buy': return 'bg-green-500/20 text-green-400'
    case 'sell': return 'bg-red-500/20 text-red-400'
    case 'transfer': return 'bg-blue-500/20 text-blue-400'
    default: return 'bg-white/10 text-white/70'
  }
})

const activityTypeText = computed(() => {
  if (!props.activity) return 'ACTIVITY'
  switch (props.activity.type) {
    case 'buy': return 'BUY'
    case 'sell': return 'SELL'
    case 'transfer': return 'TRANSFER'
    default: return 'ACTIVITY'
  }
})

// Methods
function handleBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}

function formatAddress(address: string): string {
  return `${address.slice(0, 8)}...${address.slice(-6)}`
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60))

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`

  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function formatValue(value: number): string {
  if (value >= 1e9) return `${(value / 1e9).toFixed(2)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(2)}M`
  if (value >= 1e3) return `${(value / 1e3).toFixed(2)}K`
  return value.toFixed(2)
}

function getActivityTypeIcon(type: string): string {
  switch (type) {
    case 'buy': return 'ArrowTrendingUpIcon'
    case 'sell': return 'ArrowTrendingDownIcon'
    case 'transfer': return 'ArrowsRightLeftIcon'
    default: return 'CurrencyDollarIcon'
  }
}

function getActivityTypeColor(type: string): string {
  switch (type) {
    case 'buy': return 'bg-green-500/20 text-green-400'
    case 'sell': return 'bg-red-500/20 text-red-400'
    case 'transfer': return 'bg-blue-500/20 text-blue-400'
    default: return 'bg-white/10 text-white/70'
  }
}

function getHeatmapColor(value: number): string {
  if (value === 0) return 'bg-slate-700/30'
  if (value <= 2) return 'bg-blue-500/20'
  if (value <= 4) return 'bg-blue-500/40'
  if (value <= 6) return 'bg-blue-500/60'
  if (value <= 8) return 'bg-blue-500/80'
  return 'bg-blue-500'
}

async function copyAddress() {
  if (!props.activity) return

  try {
    await navigator.clipboard.writeText(props.activity.address)
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

async function copyTxHash() {
  if (!props.activity) return

  try {
    await navigator.clipboard.writeText(props.activity.txHash)
    notificationStore.notifySystem(
      'Copied',
      'Transaction hash copied to clipboard',
      'success'
    )
  } catch (error) {
    notificationStore.notifySystem(
      'Copy Failed',
      'Failed to copy transaction hash',
      'error'
    )
  }
}

function viewOnExplorer() {
  if (!props.activity) return

  const explorerUrls: Record<string, string> = {
    ethereum: 'https://etherscan.io',
    polygon: 'https://polygonscan.com',
    arbitrum: 'https://arbiscan.io',
    optimism: 'https://optimistic.etherscan.io',
    base: 'https://basescan.org'
  }

  const explorerUrl = explorerUrls[props.activity.chain?.toLowerCase() || 'ethereum']
  const url = `${explorerUrl}/tx/${props.activity.txHash}`

  window.open(url, '_blank', 'noopener,noreferrer')
}

function toggleWatchlist() {
  isWatched.value = !isWatched.value
  notificationStore.notifySystem(
    isWatched.value ? 'Added to Watchlist' : 'Removed from Watchlist',
    isWatched.value ? 'You will receive notifications for this whale' : 'Notifications disabled for this whale',
    'info'
  )
}

function selectRelatedActivity(activity: any) {
  emit('activity-selected', activity)
}

function viewWalletDetails() {
  // Navigate to wallet detail page
  notificationStore.notifySystem('Opening Wallet', 'Loading detailed wallet analysis...', 'info')
}

function viewAllActivities() {
  // Navigate to all activities for this whale
  notificationStore.notifySystem('Loading Activities', 'Fetching all whale activities...', 'info')
}

function viewWhaleProfile() {
  // Navigate to whale profile page
  notificationStore.notifySystem('Opening Profile', 'Loading whale profile...', 'info')
}

function followTrade() {
  notificationStore.notifySystem('Copy Trade', 'Setting up trade copy parameters...', 'info')
}

function setAlert() {
  notificationStore.notifySystem('Alert Created', 'You will be notified of similar activities', 'success')
}

function addToWatchlist() {
  notificationStore.notifySystem('Added to Watchlist', 'Whale added to your watchlist', 'success')
}
</script>

<style scoped>
/* Modal animations */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-content-enter-active,
.modal-content-leave-active {
  transition: all 0.3s ease;
}

.modal-content-enter-from,
.modal-content-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(1rem);
}

/* Custom scrollbar */
.overflow-y-auto::-webkit-scrollbar {
  width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
  background: rgba(14, 165, 233, 0.5);
  border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: rgba(14, 165, 233, 0.7);
}

/* Glass morphism effect */
.bg-slate-900\/95 {
  background: rgba(15, 23, 42, 0.95);
}

/* Enhanced backdrop blur */
.backdrop-blur-xl {
  backdrop-filter: blur(20px);
}
</style>