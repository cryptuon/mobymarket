<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ClockIcon" class="w-5 h-5 text-blue-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Recent Activity</h3>
            <p class="text-xs text-white/60">Latest portfolio changes</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Activity Filter -->
          <select
            v-model="activityFilter"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="all">All Activity</option>
            <option value="trades">Trades Only</option>
            <option value="yield">Yield & Rewards</option>
            <option value="transfers">Transfers</option>
          </select>

          <!-- Auto-refresh Toggle -->
          <button
            @click="autoRefresh = !autoRefresh"
            :class="[
              'p-2 rounded-lg transition-colors',
              autoRefresh ? 'bg-green-500/20 text-green-400' : 'hover:bg-white/10 text-white/70'
            ]"
            :title="autoRefresh ? 'Auto-refresh enabled' : 'Enable auto-refresh'"
          >
            <HeroIcon name="ArrowPathIcon" class="w-4 h-4" :class="{ 'animate-spin': isRefreshing }" />
          </button>
        </div>
      </div>
    </template>

    <div class="space-y-3">
      <!-- Loading State -->
      <div v-if="loading" class="space-y-3">
        <div v-for="i in 5" :key="i" class="animate-pulse">
          <div class="flex items-center space-x-3 p-3 bg-slate-800/30 rounded-lg">
            <div class="w-8 h-8 bg-slate-700/50 rounded-lg"></div>
            <div class="flex-1 space-y-2">
              <div class="h-4 bg-slate-700/50 rounded w-32"></div>
              <div class="h-3 bg-slate-700/50 rounded w-24"></div>
            </div>
            <div class="h-4 bg-slate-700/50 rounded w-16"></div>
          </div>
        </div>
      </div>

      <!-- Activity List -->
      <div v-else class="space-y-2 max-h-96 overflow-y-auto">
        <TransitionGroup
          name="activity-list"
          tag="div"
          class="space-y-2"
        >
          <div
            v-for="activity in filteredActivities"
            :key="activity.id"
            class="flex items-center space-x-3 p-3 bg-slate-800/30 hover:bg-slate-700/30 rounded-lg transition-all duration-200 cursor-pointer group"
            @click="selectActivity(activity)"
          >
            <!-- Activity Icon -->
            <div :class="getActivityIconClass(activity.type)" class="w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0">
              <HeroIcon :name="getActivityIcon(activity.type)" class="w-4 h-4" />
            </div>

            <!-- Activity Details -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center space-x-2 mb-1">
                <span class="text-white font-medium">{{ getActivityTitle(activity) }}</span>
                <span :class="getActivityStatusClass(activity.status)" class="px-2 py-0.5 rounded-full text-xs font-medium">
                  {{ activity.status }}
                </span>
              </div>
              <div class="text-xs text-white/60">
                {{ formatTimestamp(activity.timestamp) }} • {{ activity.asset }}
              </div>
            </div>

            <!-- Activity Value -->
            <div class="text-right flex-shrink-0">
              <div :class="['font-semibold', getValueColor(activity)]">
                {{ formatActivityValue(activity) }}
              </div>
              <div class="text-xs text-white/60">
                {{ activity.action === 'buy' || activity.action === 'deposit' ? '+' : '-' }}{{ activity.amount }}
              </div>
            </div>

            <!-- Expand Indicator -->
            <HeroIcon name="ChevronRightIcon" class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors" />
          </div>
        </TransitionGroup>

        <!-- Empty State -->
        <div v-if="filteredActivities.length === 0" class="text-center py-8">
          <HeroIcon name="ClockIcon" class="w-12 h-12 text-white/30 mx-auto mb-2" />
          <p class="text-white/60 text-sm">No recent {{ activityFilter === 'all' ? '' : activityFilter }} activity</p>
        </div>
      </div>

      <!-- Load More -->
      <div v-if="!loading && hasMore" class="pt-3 border-t border-white/10">
        <Button
          @click="loadMore"
          :loading="loadingMore"
          variant="ghost"
          size="sm"
          class="w-full"
          icon-left="ArrowDownIcon"
        >
          Load More Activity
        </Button>
      </div>
    </div>

    <!-- Activity Detail Modal -->
    <Transition
      name="modal"
      enter-active-class="transition-all duration-200"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition-all duration-150"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="selectedActivity"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
        @click="selectedActivity = null"
      >
        <div
          class="bg-slate-800/90 backdrop-blur border border-white/20 rounded-xl p-6 max-w-lg w-full"
          @click.stop
        >
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center space-x-3">
              <div :class="getActivityIconClass(selectedActivity.type)" class="w-10 h-10 rounded-lg flex items-center justify-center">
                <HeroIcon :name="getActivityIcon(selectedActivity.type)" class="w-5 h-5" />
              </div>
              <div>
                <h4 class="text-lg font-semibold text-white">{{ getActivityTitle(selectedActivity) }}</h4>
                <p class="text-sm text-white/60">{{ formatTimestamp(selectedActivity.timestamp, true) }}</p>
              </div>
            </div>
            <button
              @click="selectedActivity = null"
              class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
            </button>
          </div>

          <div class="grid grid-cols-2 gap-4 mb-4">
            <div>
              <div class="text-xs text-white/60">Asset</div>
              <div class="text-lg font-semibold text-white">{{ selectedActivity.asset }}</div>
            </div>
            <div>
              <div class="text-xs text-white/60">Amount</div>
              <div class="text-lg font-semibold text-white">{{ selectedActivity.amount }}</div>
            </div>
            <div>
              <div class="text-xs text-white/60">Value</div>
              <div class="text-lg font-semibold text-white">${{ formatCurrency(selectedActivity.value) }}</div>
            </div>
            <div>
              <div class="text-xs text-white/60">Fee</div>
              <div class="text-lg font-semibold text-white">${{ selectedActivity.fee?.toFixed(2) || '0.00' }}</div>
            </div>
          </div>

          <div v-if="selectedActivity.txHash" class="mb-4">
            <div class="text-xs text-white/60 mb-1">Transaction Hash</div>
            <div class="flex items-center space-x-2">
              <code class="text-sm font-mono text-white bg-slate-700/50 px-2 py-1 rounded flex-1 truncate">
                {{ selectedActivity.txHash }}
              </code>
              <Button
                @click="copyTxHash(selectedActivity.txHash!)"
                variant="ghost"
                size="xs"
                icon-left="ClipboardIcon"
              >
                Copy
              </Button>
            </div>
          </div>

          <div class="flex items-center space-x-2">
            <Button
              v-if="selectedActivity.txHash"
              @click="viewOnExplorer(selectedActivity.txHash!)"
              variant="outline"
              size="sm"
              icon-left="ArrowTopRightOnSquareIcon"
              class="flex-1"
            >
              View on Explorer
            </Button>
            <Button
              @click="selectedActivity = null"
              variant="primary"
              size="sm"
              class="flex-1"
            >
              Close
            </Button>
          </div>
        </div>
      </div>
    </Transition>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <span>{{ filteredActivities.length }} activities shown</span>
        <div class="flex items-center space-x-2">
          <div :class="['w-2 h-2 rounded-full', autoRefresh ? 'bg-green-400 animate-pulse' : 'bg-slate-500']"></div>
          <span>{{ autoRefresh ? 'Live updates' : 'Manual refresh' }}</span>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useNotificationStore } from '@/stores/notifications'

interface Activity {
  id: string
  type: 'trade' | 'yield' | 'transfer' | 'swap'
  action: 'buy' | 'sell' | 'deposit' | 'withdraw' | 'harvest' | 'claim'
  asset: string
  amount: number
  value: number
  fee?: number
  timestamp: string
  status: 'pending' | 'completed' | 'failed'
  txHash?: string
}

interface Props {
  activities: Activity[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const notificationStore = useNotificationStore()

const activityFilter = ref<'all' | 'trades' | 'yield' | 'transfers'>('all')
const autoRefresh = ref(true)
const isRefreshing = ref(false)
const loadingMore = ref(false)
const hasMore = ref(true)
const selectedActivity = ref<Activity | null>(null)

let refreshInterval: NodeJS.Timeout | null = null

// Computed properties
const filteredActivities = computed(() => {
  return props.activities.filter(activity => {
    switch (activityFilter.value) {
      case 'trades':
        return activity.type === 'trade' || activity.type === 'swap'
      case 'yield':
        return activity.action === 'harvest' || activity.action === 'claim'
      case 'transfers':
        return activity.type === 'transfer'
      default:
        return true
    }
  })
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function formatTimestamp(timestamp: string, detailed = false): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60))

  if (detailed) {
    return date.toLocaleString()
  }

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`

  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric'
  })
}

function getActivityIcon(type: string): string {
  switch (type) {
    case 'trade':
    case 'swap':
      return 'ArrowsRightLeftIcon'
    case 'yield':
      return 'SparklesIcon'
    case 'transfer':
      return 'PaperAirplaneIcon'
    default:
      return 'CurrencyDollarIcon'
  }
}

function getActivityIconClass(type: string): string {
  switch (type) {
    case 'trade':
    case 'swap':
      return 'bg-blue-500/20 text-blue-400'
    case 'yield':
      return 'bg-green-500/20 text-green-400'
    case 'transfer':
      return 'bg-purple-500/20 text-purple-400'
    default:
      return 'bg-slate-500/20 text-slate-400'
  }
}

function getActivityTitle(activity: Activity): string {
  const actionMap: Record<string, string> = {
    buy: 'Buy',
    sell: 'Sell',
    deposit: 'Deposit',
    withdraw: 'Withdraw',
    harvest: 'Harvest Rewards',
    claim: 'Claim Rewards'
  }
  return actionMap[activity.action] || activity.action
}

function getActivityStatusClass(status: string): string {
  switch (status) {
    case 'completed':
      return 'bg-green-500/20 text-green-400'
    case 'pending':
      return 'bg-yellow-500/20 text-yellow-400'
    case 'failed':
      return 'bg-red-500/20 text-red-400'
    default:
      return 'bg-slate-500/20 text-slate-400'
  }
}

function getValueColor(activity: Activity): string {
  if (activity.status === 'failed') return 'text-red-400'
  if (activity.action === 'buy' || activity.action === 'deposit') return 'text-green-400'
  return 'text-white'
}

function formatActivityValue(activity: Activity): string {
  const prefix = activity.action === 'sell' || activity.action === 'withdraw' ? '+' : ''
  return `${prefix}$${formatCurrency(activity.value)}`
}

function selectActivity(activity: Activity) {
  selectedActivity.value = activity
}

async function copyTxHash(txHash: string) {
  try {
    await navigator.clipboard.writeText(txHash)
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

function viewOnExplorer(txHash: string) {
  // Mock explorer URL - in real app would use appropriate explorer
  const url = `https://etherscan.io/tx/${txHash}`
  window.open(url, '_blank', 'noopener,noreferrer')
}

async function loadMore() {
  loadingMore.value = true
  try {
    // Simulate loading more data
    await new Promise(resolve => setTimeout(resolve, 1000))
    // In real app, would emit event to parent to load more data
    hasMore.value = Math.random() > 0.5 // Random chance of having more
  } finally {
    loadingMore.value = false
  }
}

async function refreshData() {
  if (isRefreshing.value) return

  isRefreshing.value = true
  try {
    // Simulate refresh
    await new Promise(resolve => setTimeout(resolve, 1000))
    // In real app, would emit refresh event to parent
  } finally {
    isRefreshing.value = false
  }
}

// Auto-refresh functionality
onMounted(() => {
  if (autoRefresh.value) {
    refreshInterval = setInterval(refreshData, 30000) // Refresh every 30 seconds
  }
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

// Watch autoRefresh changes
function toggleAutoRefresh() {
  if (autoRefresh.value) {
    refreshInterval = setInterval(refreshData, 30000)
  } else if (refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  }
}

// Watch autoRefresh
watch(() => autoRefresh.value, toggleAutoRefresh)
</script>

<style scoped>
/* Activity list animations */
.activity-list-enter-active {
  transition: all 0.3s ease-out;
}

.activity-list-enter-from {
  transform: translateY(-10px);
  opacity: 0;
}

.activity-list-leave-active {
  transition: all 0.2s ease-in;
}

.activity-list-leave-to {
  transform: translateY(10px);
  opacity: 0;
}

.activity-list-move {
  transition: transform 0.3s ease;
}

/* Modal animations */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

/* Custom scrollbar */
.max-h-96::-webkit-scrollbar {
  width: 4px;
}

.max-h-96::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.max-h-96::-webkit-scrollbar-thumb {
  background: rgba(14, 165, 233, 0.5);
  border-radius: 2px;
}

.max-h-96::-webkit-scrollbar-thumb:hover {
  background: rgba(14, 165, 233, 0.7);
}
</style>