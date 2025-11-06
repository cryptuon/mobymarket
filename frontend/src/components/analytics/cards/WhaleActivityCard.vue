<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="UserGroupIcon" class="w-5 h-5 text-cyan-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Whale Activity</h3>
            <p class="text-xs text-white/60">Large holder movements</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <select
            v-model="selectedTimeframe"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="1h">1 Hour</option>
            <option value="24h">24 Hours</option>
            <option value="7d">7 Days</option>
          </select>

          <div :class="getActivityBadgeClass()" class="px-2 py-1 rounded-lg text-xs font-medium">
            {{ getActivityLevel() }}
          </div>
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Loading State -->
      <div v-if="loading" class="space-y-3">
        <div v-for="i in 5" :key="i" class="animate-pulse">
          <div class="flex items-center space-x-3 p-3 bg-slate-800/30 rounded-lg">
            <div class="w-8 h-8 bg-slate-700/50 rounded-full"></div>
            <div class="flex-1 space-y-2">
              <div class="h-4 bg-slate-700/50 rounded w-3/4"></div>
              <div class="h-3 bg-slate-700/50 rounded w-1/2"></div>
            </div>
            <div class="h-4 bg-slate-700/50 rounded w-20"></div>
          </div>
        </div>
      </div>

      <div v-else class="space-y-6">
        <!-- Activity Summary -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="ArrowRightOnRectangleIcon" class="w-4 h-4 text-green-400" />
              <span class="text-xs text-white/60">Inflows</span>
            </div>
            <div class="text-xl font-bold text-green-400">${{ formatAmount(activityStats.inflows) }}</div>
            <div class="text-xs text-white/60">{{ activityStats.inflowCount }} transactions</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="ArrowLeftOnRectangleIcon" class="w-4 h-4 text-red-400" />
              <span class="text-xs text-white/60">Outflows</span>
            </div>
            <div class="text-xl font-bold text-red-400">${{ formatAmount(activityStats.outflows) }}</div>
            <div class="text-xs text-white/60">{{ activityStats.outflowCount }} transactions</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="ScaleIcon" class="w-4 h-4 text-blue-400" />
              <span class="text-xs text-white/60">Net Flow</span>
            </div>
            <div :class="['text-xl font-bold', getNetFlowColor(activityStats.netFlow)]">
              {{ activityStats.netFlow >= 0 ? '+' : '' }}${{ formatAmount(Math.abs(activityStats.netFlow)) }}
            </div>
            <div class="text-xs text-white/60">{{ getNetFlowLabel(activityStats.netFlow) }}</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="UserIcon" class="w-4 h-4 text-purple-400" />
              <span class="text-xs text-white/60">Active Whales</span>
            </div>
            <div class="text-xl font-bold text-white">{{ activityStats.activeWhales }}</div>
            <div :class="['text-xs', getChangeColor(activityStats.whaleChange)]">
              {{ activityStats.whaleChange >= 0 ? '+' : '' }}{{ activityStats.whaleChange }}%
            </div>
          </div>
        </div>

        <!-- Recent Whale Transactions -->
        <div class="space-y-3">
          <h4 class="text-sm font-semibold text-white">Recent Large Transactions</h4>
          <div class="space-y-2">
            <TransitionGroup
              name="whale-list"
              tag="div"
              class="space-y-2"
            >
              <div
                v-for="transaction in recentTransactions"
                :key="transaction.id"
                class="flex items-center space-x-3 p-3 bg-slate-800/30 hover:bg-slate-700/30 rounded-lg transition-all duration-200 cursor-pointer group"
                @click="selectTransaction(transaction)"
              >
                <!-- Transaction Type Icon -->
                <div :class="getTransactionIconClass(transaction.type)" class="p-2 rounded-lg">
                  <HeroIcon :name="getTransactionIcon(transaction.type)" class="w-4 h-4" />
                </div>

                <!-- Asset Info -->
                <div class="flex items-center space-x-2">
                  <img
                    :src="getAssetIcon(transaction.asset)"
                    :alt="transaction.asset"
                    class="w-6 h-6 rounded-full"
                  />
                  <div>
                    <div class="text-sm font-medium text-white">{{ transaction.asset }}</div>
                    <div class="text-xs text-white/60">{{ getTransactionLabel(transaction.type) }}</div>
                  </div>
                </div>

                <!-- Transaction Details -->
                <div class="flex-1 min-w-0">
                  <div class="flex items-center justify-between">
                    <div>
                      <div class="text-sm font-bold text-white">
                        {{ formatAmount(transaction.amount) }} {{ transaction.asset }}
                      </div>
                      <div class="text-xs text-white/60">
                        ${{ formatAmount(transaction.usdValue) }}
                      </div>
                    </div>
                    <div class="text-right">
                      <div class="text-xs text-white/60">{{ formatTimeAgo(transaction.timestamp) }}</div>
                      <div class="flex items-center space-x-1">
                        <div :class="getWhaleRankClass(transaction.whaleRank)" class="px-1 py-0.5 rounded text-xs font-medium">
                          {{ transaction.whaleRank }}
                        </div>
                        <HeroIcon name="ChevronRightIcon" class="w-3 h-3 text-white/40 group-hover:text-white/70 transition-colors" />
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </TransitionGroup>
          </div>
        </div>

        <!-- Whale Categories -->
        <div class="space-y-4">
          <h4 class="text-sm font-semibold text-white">Whale Categories</h4>
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
            <div
              v-for="category in whaleCategories"
              :key="category.name"
              class="bg-slate-800/20 rounded-lg p-4"
            >
              <div class="flex items-center space-x-2 mb-3">
                <HeroIcon :name="category.icon" :class="['w-4 h-4', category.iconColor]" />
                <span class="text-sm font-medium text-white">{{ category.name }}</span>
              </div>

              <div class="space-y-2">
                <div class="flex justify-between text-xs">
                  <span class="text-white/60">Active Count</span>
                  <span class="text-white font-medium">{{ category.activeCount }}</span>
                </div>
                <div class="flex justify-between text-xs">
                  <span class="text-white/60">Total Holdings</span>
                  <span class="text-white font-medium">${{ formatAmount(category.holdings) }}</span>
                </div>
                <div class="flex justify-between text-xs">
                  <span class="text-white/60">24h Activity</span>
                  <span :class="getActivityColor(category.activity)">
                    {{ category.activity >= 0 ? '+' : '' }}{{ category.activity.toFixed(1) }}%
                  </span>
                </div>

                <!-- Activity Bar -->
                <div class="w-full h-2 bg-slate-700/50 rounded-full overflow-hidden mt-2">
                  <div
                    :class="getActivityBarClass(category.activity)"
                    :style="{ width: `${Math.min(100, Math.abs(category.activity) * 2)}%` }"
                    class="h-full transition-all duration-500"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Whale Alerts -->
        <div v-if="whaleAlerts.length > 0" class="space-y-2">
          <h4 class="text-sm font-semibold text-white flex items-center space-x-2">
            <HeroIcon name="BellIcon" class="w-4 h-4 text-yellow-400" />
            <span>Whale Alerts</span>
          </h4>
          <div class="space-y-2">
            <div
              v-for="alert in whaleAlerts"
              :key="alert.id"
              :class="getAlertClass(alert.severity)"
              class="p-3 rounded-lg border"
            >
              <div class="flex items-start space-x-2">
                <HeroIcon :name="getAlertIcon(alert.severity)" class="w-4 h-4 mt-0.5 flex-shrink-0" />
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium">{{ alert.title }}</div>
                  <div class="text-xs mt-1 opacity-80">{{ alert.message }}</div>
                  <div class="text-xs text-white/50 mt-1">{{ formatTimeAgo(alert.timestamp) }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Flow Trend Chart -->
        <div class="space-y-3">
          <h4 class="text-sm font-semibold text-white">Flow Trend (24h)</h4>
          <div class="flex items-end space-x-1 h-16">
            <div
              v-for="(hour, index) in flowTrend"
              :key="index"
              class="flex-1 flex flex-col items-center"
            >
              <div class="relative w-full h-full flex flex-col justify-end">
                <!-- Inflow bar -->
                <div
                  v-if="hour.inflow > 0"
                  class="bg-green-400 w-full rounded-t-sm"
                  :style="{ height: `${(hour.inflow / maxFlow) * 100}%` }"
                ></div>
                <!-- Outflow bar -->
                <div
                  v-if="hour.outflow > 0"
                  class="bg-red-400 w-full rounded-b-sm"
                  :style="{ height: `${(hour.outflow / maxFlow) * 100}%` }"
                ></div>
              </div>
              <div class="text-xs text-white/60 mt-1">{{ index }}h</div>
            </div>
          </div>
          <div class="flex items-center justify-center space-x-6 text-xs">
            <div class="flex items-center space-x-2">
              <div class="w-3 h-2 bg-green-400 rounded"></div>
              <span class="text-white/70">Inflows</span>
            </div>
            <div class="flex items-center space-x-2">
              <div class="w-3 h-2 bg-red-400 rounded"></div>
              <span class="text-white/70">Outflows</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Transaction Detail Modal -->
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
        v-if="selectedTransaction"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
        @click="selectedTransaction = null"
      >
        <div
          class="bg-slate-800/90 backdrop-blur border border-white/20 rounded-xl p-6 max-w-md w-full"
          @click.stop
        >
          <div class="flex items-center justify-between mb-4">
            <h4 class="text-lg font-semibold text-white">Transaction Details</h4>
            <button
              @click="selectedTransaction = null"
              class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
            </button>
          </div>

          <div class="space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <div class="text-xs text-white/60">Asset</div>
                <div class="text-sm font-bold text-white">{{ selectedTransaction.asset }}</div>
              </div>
              <div>
                <div class="text-xs text-white/60">Type</div>
                <div class="text-sm font-bold text-white">{{ getTransactionLabel(selectedTransaction.type) }}</div>
              </div>
              <div>
                <div class="text-xs text-white/60">Amount</div>
                <div class="text-sm font-bold text-white">
                  {{ formatAmount(selectedTransaction.amount) }} {{ selectedTransaction.asset }}
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60">USD Value</div>
                <div class="text-sm font-bold text-white">${{ formatAmount(selectedTransaction.usdValue) }}</div>
              </div>
              <div>
                <div class="text-xs text-white/60">Whale Rank</div>
                <div :class="getWhaleRankClass(selectedTransaction.whaleRank)" class="text-sm font-bold px-2 py-1 rounded">
                  {{ selectedTransaction.whaleRank }}
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60">Time</div>
                <div class="text-sm font-bold text-white">{{ formatTimeAgo(selectedTransaction.timestamp) }}</div>
              </div>
            </div>

            <div v-if="selectedTransaction.txHash" class="pt-4 border-t border-white/10">
              <div class="text-xs text-white/60 mb-2">Transaction Hash</div>
              <div class="flex items-center space-x-2">
                <code class="text-xs bg-slate-700/50 px-2 py-1 rounded text-white/80 font-mono flex-1 truncate">
                  {{ selectedTransaction.txHash }}
                </code>
                <Button
                  variant="ghost"
                  size="xs"
                  @click="copyToClipboard(selectedTransaction.txHash)"
                >
                  Copy
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface WhaleTransaction {
  id: string
  asset: string
  amount: number
  usdValue: number
  type: 'deposit' | 'withdrawal' | 'transfer' | 'trade'
  timestamp: string
  whaleRank: 'Mega' | 'Large' | 'Medium'
  txHash?: string
}

interface WhaleAlert {
  id: string
  title: string
  message: string
  severity: 'low' | 'medium' | 'high'
  timestamp: string
}

interface Props {
  data: WhaleTransaction[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const selectedTimeframe = ref('24h')
const selectedTransaction = ref<WhaleTransaction | null>(null)

// Generate mock data for demonstration
const generateMockData = (): WhaleTransaction[] => {
  const assets = ['ETH', 'BTC', 'UNI', 'AAVE', 'COMP']
  const types: WhaleTransaction['type'][] = ['deposit', 'withdrawal', 'transfer', 'trade']
  const ranks: WhaleTransaction['whaleRank'][] = ['Mega', 'Large', 'Medium']

  return Array.from({ length: 10 }, (_, i) => ({
    id: `tx-${i}`,
    asset: assets[Math.floor(Math.random() * assets.length)],
    amount: 100 + Math.random() * 10000,
    usdValue: 100000 + Math.random() * 5000000,
    type: types[Math.floor(Math.random() * types.length)],
    timestamp: new Date(Date.now() - Math.random() * 24 * 60 * 60 * 1000).toISOString(),
    whaleRank: ranks[Math.floor(Math.random() * ranks.length)],
    txHash: `0x${Math.random().toString(16).substr(2, 64)}`
  }))
}

const recentTransactions = computed(() =>
  props.data.length ? props.data : generateMockData()
)

// Activity statistics
const activityStats = computed(() => {
  const transactions = recentTransactions.value
  const inflows = transactions.filter(t => t.type === 'deposit').reduce((sum, t) => sum + t.usdValue, 0)
  const outflows = transactions.filter(t => t.type === 'withdrawal').reduce((sum, t) => sum + t.usdValue, 0)

  return {
    inflows,
    outflows,
    netFlow: inflows - outflows,
    inflowCount: transactions.filter(t => t.type === 'deposit').length,
    outflowCount: transactions.filter(t => t.type === 'withdrawal').length,
    activeWhales: new Set(transactions.map(t => t.whaleRank)).size * 15, // Mock calculation
    whaleChange: Math.random() * 20 - 10 // Random change %
  }
})

// Whale categories
const whaleCategories = computed(() => [
  {
    name: 'Mega Whales',
    icon: 'UserIcon',
    iconColor: 'text-purple-400',
    activeCount: 12,
    holdings: 2500000000,
    activity: 8.5
  },
  {
    name: 'Large Whales',
    icon: 'UserGroupIcon',
    iconColor: 'text-blue-400',
    activeCount: 45,
    holdings: 1200000000,
    activity: -3.2
  },
  {
    name: 'Medium Whales',
    icon: 'UsersIcon',
    iconColor: 'text-cyan-400',
    activeCount: 128,
    holdings: 800000000,
    activity: 12.1
  }
])

// Whale alerts
const whaleAlerts = computed(() => {
  const alerts: WhaleAlert[] = []

  const largeTransactions = recentTransactions.value.filter(t => t.usdValue > 1000000)
  if (largeTransactions.length > 3) {
    alerts.push({
      id: '1',
      title: 'High Volume Alert',
      message: `${largeTransactions.length} transactions over $1M detected in the last hour`,
      severity: 'high',
      timestamp: new Date().toISOString()
    })
  }

  if (activityStats.value.netFlow < -5000000) {
    alerts.push({
      id: '2',
      title: 'Large Net Outflow',
      message: `Net outflow of $${formatAmount(Math.abs(activityStats.value.netFlow))} detected`,
      severity: 'medium',
      timestamp: new Date().toISOString()
    })
  }

  return alerts
})

// Flow trend data (24 hours)
const flowTrend = computed(() => {
  return Array.from({ length: 24 }, (_, i) => ({
    inflow: Math.random() * 1000000,
    outflow: Math.random() * 1000000
  }))
})

const maxFlow = computed(() => {
  return Math.max(...flowTrend.value.flatMap(h => [h.inflow, h.outflow]))
})

// Methods
function formatAmount(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
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

function getActivityLevel(): string {
  const totalValue = activityStats.value.inflows + activityStats.value.outflows
  if (totalValue > 50000000) return 'Very High'
  if (totalValue > 20000000) return 'High'
  if (totalValue > 5000000) return 'Moderate'
  return 'Low'
}

function getActivityBadgeClass(): string {
  const level = getActivityLevel()
  switch (level) {
    case 'Very High': return 'bg-red-500/20 text-red-400'
    case 'High': return 'bg-orange-500/20 text-orange-400'
    case 'Moderate': return 'bg-yellow-500/20 text-yellow-400'
    default: return 'bg-green-500/20 text-green-400'
  }
}

function getTransactionIcon(type: string): string {
  switch (type) {
    case 'deposit': return 'ArrowRightOnRectangleIcon'
    case 'withdrawal': return 'ArrowLeftOnRectangleIcon'
    case 'transfer': return 'ArrowsRightLeftIcon'
    case 'trade': return 'CurrencyDollarIcon'
    default: return 'DocumentIcon'
  }
}

function getTransactionIconClass(type: string): string {
  switch (type) {
    case 'deposit': return 'bg-green-500/20 text-green-400'
    case 'withdrawal': return 'bg-red-500/20 text-red-400'
    case 'transfer': return 'bg-blue-500/20 text-blue-400'
    case 'trade': return 'bg-purple-500/20 text-purple-400'
    default: return 'bg-gray-500/20 text-gray-400'
  }
}

function getTransactionLabel(type: string): string {
  switch (type) {
    case 'deposit': return 'Deposit'
    case 'withdrawal': return 'Withdrawal'
    case 'transfer': return 'Transfer'
    case 'trade': return 'Trade'
    default: return 'Unknown'
  }
}

function getWhaleRankClass(rank: string): string {
  switch (rank) {
    case 'Mega': return 'bg-purple-500/20 text-purple-400'
    case 'Large': return 'bg-blue-500/20 text-blue-400'
    case 'Medium': return 'bg-cyan-500/20 text-cyan-400'
    default: return 'bg-gray-500/20 text-gray-400'
  }
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

function getNetFlowColor(netFlow: number): string {
  return netFlow >= 0 ? 'text-green-400' : 'text-red-400'
}

function getNetFlowLabel(netFlow: number): string {
  return netFlow >= 0 ? 'Net inflow' : 'Net outflow'
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function getActivityColor(activity: number): string {
  return activity >= 0 ? 'text-green-400' : 'text-red-400'
}

function getActivityBarClass(activity: number): string {
  return activity >= 0 ? 'bg-green-400' : 'bg-red-400'
}

function getAlertClass(severity: string): string {
  switch (severity) {
    case 'high': return 'bg-red-500/10 border-red-500/30 text-red-400'
    case 'medium': return 'bg-yellow-500/10 border-yellow-500/30 text-yellow-400'
    default: return 'bg-blue-500/10 border-blue-500/30 text-blue-400'
  }
}

function getAlertIcon(severity: string): string {
  switch (severity) {
    case 'high': return 'ExclamationTriangleIcon'
    case 'medium': return 'ExclamationCircleIcon'
    default: return 'InformationCircleIcon'
  }
}

function selectTransaction(transaction: WhaleTransaction) {
  selectedTransaction.value = transaction
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text)
}
</script>

<style scoped>
/* List animations */
.whale-list-enter-active {
  transition: all 0.3s ease-out;
}

.whale-list-enter-from {
  transform: translateX(-20px);
  opacity: 0;
}

.whale-list-leave-active {
  transition: all 0.2s ease-in;
}

.whale-list-leave-to {
  transform: translateX(20px);
  opacity: 0;
}

.whale-list-move {
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

/* Card hover effects */
.group:hover .bg-slate-800\/30 {
  background-color: rgba(51, 65, 85, 0.4);
}
</style>