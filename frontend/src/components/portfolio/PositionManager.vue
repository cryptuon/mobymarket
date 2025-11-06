<template>
  <div class="space-y-6">
    <!-- Header -->
    <Card variant="glass">
      <div class="flex items-center justify-between p-6">
        <div class="flex items-center space-x-4">
          <div class="w-10 h-10 bg-gradient-to-br from-purple-400 to-purple-600 rounded-lg flex items-center justify-center">
            <HeroIcon name="RectangleStackIcon" class="w-5 h-5 text-white" />
          </div>
          <div>
            <h2 class="text-xl font-bold text-white">Position Manager</h2>
            <p class="text-sm text-white/60">Manage and optimize your positions</p>
          </div>
        </div>

        <div class="flex items-center space-x-3">
          <Button
            variant="primary"
            icon-left="PlusIcon"
            @click="$emit('add-position')"
          >
            Add Position
          </Button>
          <Button
            variant="secondary"
            icon-left="ArrowPathIcon"
            @click="$emit('rebalance-portfolio')"
          >
            Rebalance
          </Button>
          <Button
            variant="ghost"
            icon-left="AdjustmentsHorizontalIcon"
            @click="showFilters = !showFilters"
          />
        </div>
      </div>
    </Card>

    <!-- Filters -->
    <Transition
      name="slide-down"
      enter-active-class="transition-all duration-300"
      enter-from-class="opacity-0 -translate-y-4"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition-all duration-200"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-4"
    >
      <Card v-if="showFilters" variant="glass" class="p-4">
        <div class="grid grid-cols-1 lg:grid-cols-4 gap-4">
          <div>
            <label class="block text-xs text-white/60 mb-2">Asset Type</label>
            <select
              v-model="filters.assetType"
              class="w-full bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-moby-500/50"
            >
              <option value="">All Assets</option>
              <option value="crypto">Cryptocurrency</option>
              <option value="defi">DeFi Tokens</option>
              <option value="nft">NFTs</option>
              <option value="stablecoin">Stablecoins</option>
            </select>
          </div>

          <div>
            <label class="block text-xs text-white/60 mb-2">Performance</label>
            <select
              v-model="filters.performance"
              class="w-full bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-moby-500/50"
            >
              <option value="">All Positions</option>
              <option value="winners">Winners Only</option>
              <option value="losers">Losers Only</option>
              <option value="breakeven">Break Even</option>
            </select>
          </div>

          <div>
            <label class="block text-xs text-white/60 mb-2">Size Range</label>
            <select
              v-model="filters.sizeRange"
              class="w-full bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-moby-500/50"
            >
              <option value="">All Sizes</option>
              <option value="large">Large (>5%)</option>
              <option value="medium">Medium (1-5%)</option>
              <option value="small">Small (<1%)</option>
            </select>
          </div>

          <div>
            <label class="block text-xs text-white/60 mb-2">Sort By</label>
            <select
              v-model="sortBy"
              class="w-full bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-moby-500/50"
            >
              <option value="value">Position Value</option>
              <option value="allocation">Allocation %</option>
              <option value="pnl">P&L Amount</option>
              <option value="pnlPercent">P&L %</option>
              <option value="symbol">Asset Name</option>
            </select>
          </div>
        </div>

        <div class="flex items-center justify-between mt-4 pt-4 border-t border-white/10">
          <div class="text-sm text-white/60">
            Showing {{ filteredPositions.length }} of {{ positions.length }} positions
          </div>
          <Button
            variant="ghost"
            size="xs"
            @click="resetFilters"
          >
            Reset Filters
          </Button>
        </div>
      </Card>
    </Transition>

    <!-- Positions Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
      <TransitionGroup
        name="position-grid"
        tag="div"
        class="contents"
      >
        <Card
          v-for="position in filteredPositions"
          :key="position.symbol"
          variant="glass"
          class="hover:shadow-lg hover:shadow-moby-500/10 transition-all duration-300 cursor-pointer"
          @click="selectPosition(position)"
        >
          <div class="p-6">
            <!-- Asset Header -->
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center space-x-3">
                <img
                  :src="getAssetIcon(position.symbol)"
                  :alt="position.symbol"
                  class="w-10 h-10 rounded-full"
                />
                <div>
                  <h3 class="text-lg font-bold text-white">{{ position.symbol }}</h3>
                  <p class="text-sm text-white/60">{{ position.name }}</p>
                </div>
              </div>

              <div class="flex items-center space-x-2">
                <div :class="getStatusBadgeClass(position.status)" class="px-2 py-1 rounded-lg text-xs font-medium">
                  {{ position.status }}
                </div>
                <Button
                  variant="ghost"
                  size="xs"
                  icon-left="EllipsisVerticalIcon"
                  @click.stop="showPositionMenu(position)"
                />
              </div>
            </div>

            <!-- Position Value -->
            <div class="mb-4">
              <div class="text-2xl font-bold text-white mb-1">
                ${{ formatAmount(position.value) }}
              </div>
              <div class="flex items-center space-x-4 text-sm">
                <span class="text-white/60">{{ formatAmount(position.amount) }} {{ position.symbol }}</span>
                <span class="text-white/60">•</span>
                <span class="text-white/60">{{ position.allocation.toFixed(1) }}% allocation</span>
              </div>
            </div>

            <!-- P&L Section -->
            <div class="grid grid-cols-2 gap-4 mb-4">
              <div class="bg-slate-800/30 rounded-lg p-3">
                <div class="text-xs text-white/60 mb-1">Unrealized P&L</div>
                <div :class="['text-lg font-bold', getChangeColor(position.pnl)]">
                  {{ position.pnl >= 0 ? '+' : '' }}${{ formatAmount(Math.abs(position.pnl)) }}
                </div>
                <div :class="['text-xs', getChangeColor(position.pnlPercent)]">
                  {{ position.pnlPercent >= 0 ? '+' : '' }}{{ position.pnlPercent.toFixed(1) }}%
                </div>
              </div>

              <div class="bg-slate-800/30 rounded-lg p-3">
                <div class="text-xs text-white/60 mb-1">Current Price</div>
                <div class="text-lg font-bold text-white">
                  ${{ formatAmount(position.currentPrice) }}
                </div>
                <div :class="['text-xs', getChangeColor(position.priceChange24h)]">
                  {{ position.priceChange24h >= 0 ? '+' : '' }}{{ position.priceChange24h.toFixed(1) }}%
                </div>
              </div>
            </div>

            <!-- Performance Chart -->
            <div class="mb-4">
              <div class="h-16 bg-slate-800/20 rounded-lg p-2">
                <svg class="w-full h-full" viewBox="0 0 200 40">
                  <path
                    :d="getPositionChartPath(position)"
                    fill="none"
                    :stroke="getPositionChartColor(position.pnlPercent)"
                    stroke-width="2"
                    class="drop-shadow-sm"
                  />
                </svg>
              </div>
            </div>

            <!-- Action Buttons -->
            <div class="flex space-x-2">
              <Button
                variant="secondary"
                size="sm"
                class="flex-1"
                @click.stop="$emit('add-to-position', position)"
              >
                Add
              </Button>
              <Button
                variant="secondary"
                size="sm"
                class="flex-1"
                @click.stop="$emit('reduce-position', position)"
              >
                Reduce
              </Button>
              <Button
                variant="ghost"
                size="sm"
                icon-left="ChartBarIcon"
                @click.stop="$emit('view-details', position)"
              />
            </div>

            <!-- Position Alerts -->
            <div v-if="position.alerts && position.alerts.length > 0" class="mt-4 pt-4 border-t border-white/10">
              <div class="space-y-2">
                <div
                  v-for="alert in position.alerts"
                  :key="alert.id"
                  :class="getAlertClass(alert.type)"
                  class="p-2 rounded-lg text-xs"
                >
                  <div class="flex items-center space-x-2">
                    <HeroIcon :name="getAlertIcon(alert.type)" class="w-3 h-3 flex-shrink-0" />
                    <span>{{ alert.message }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Card>
      </TransitionGroup>
    </div>

    <!-- Empty State -->
    <div v-if="filteredPositions.length === 0" class="text-center py-12">
      <HeroIcon name="CubeTransparentIcon" class="w-16 h-16 text-white/30 mx-auto mb-4" />
      <h3 class="text-lg font-semibold text-white mb-2">No positions found</h3>
      <p class="text-white/60 mb-6">
        {{ positions.length === 0 ? 'Start by adding your first position' : 'Try adjusting your filters' }}
      </p>
      <Button
        variant="primary"
        icon-left="PlusIcon"
        @click="$emit('add-position')"
      >
        Add Position
      </Button>
    </div>

    <!-- Position Detail Modal -->
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
        v-if="selectedPosition"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
        @click="selectedPosition = null"
      >
        <div
          class="bg-slate-800/90 backdrop-blur border border-white/20 rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto"
          @click.stop
        >
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center space-x-3">
              <img
                :src="getAssetIcon(selectedPosition.symbol)"
                :alt="selectedPosition.symbol"
                class="w-12 h-12 rounded-full"
              />
              <div>
                <h3 class="text-xl font-bold text-white">{{ selectedPosition.symbol }}</h3>
                <p class="text-white/60">{{ selectedPosition.name }}</p>
              </div>
            </div>
            <button
              @click="selectedPosition = null"
              class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
            </button>
          </div>

          <div class="grid grid-cols-2 gap-6 mb-6">
            <div class="space-y-4">
              <div>
                <div class="text-xs text-white/60 mb-1">Current Value</div>
                <div class="text-2xl font-bold text-white">${{ formatAmount(selectedPosition.value) }}</div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">Holdings</div>
                <div class="text-xl font-bold text-white">
                  {{ formatAmount(selectedPosition.amount) }} {{ selectedPosition.symbol }}
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">Average Cost</div>
                <div class="text-xl font-bold text-white">${{ formatAmount(selectedPosition.avgCost) }}</div>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <div class="text-xs text-white/60 mb-1">Unrealized P&L</div>
                <div :class="['text-2xl font-bold', getChangeColor(selectedPosition.pnl)]">
                  {{ selectedPosition.pnl >= 0 ? '+' : '' }}${{ formatAmount(Math.abs(selectedPosition.pnl)) }}
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">P&L Percentage</div>
                <div :class="['text-xl font-bold', getChangeColor(selectedPosition.pnlPercent)]">
                  {{ selectedPosition.pnlPercent >= 0 ? '+' : '' }}{{ selectedPosition.pnlPercent.toFixed(2) }}%
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">Allocation</div>
                <div class="text-xl font-bold text-white">{{ selectedPosition.allocation.toFixed(2) }}%</div>
              </div>
            </div>
          </div>

          <div class="flex space-x-3 mb-6">
            <Button
              variant="primary"
              size="sm"
              class="flex-1"
              @click="$emit('add-to-position', selectedPosition)"
            >
              Add to Position
            </Button>
            <Button
              variant="secondary"
              size="sm"
              class="flex-1"
              @click="$emit('reduce-position', selectedPosition)"
            >
              Reduce Position
            </Button>
            <Button
              variant="ghost"
              size="sm"
              @click="$emit('close-position', selectedPosition)"
            >
              Close
            </Button>
          </div>

          <div class="space-y-4">
            <h4 class="text-sm font-semibold text-white">Position History</h4>
            <div class="bg-slate-800/30 rounded-lg p-4">
              <div class="text-sm text-white/60 text-center">
                Position history chart and transaction details would be displayed here
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface PositionAlert {
  id: string
  type: 'warning' | 'info' | 'success'
  message: string
}

interface Position {
  symbol: string
  name: string
  amount: number
  value: number
  allocation: number
  pnl: number
  pnlPercent: number
  currentPrice: number
  avgCost: number
  priceChange24h: number
  status: 'active' | 'watching' | 'closed'
  assetType: string
  alerts?: PositionAlert[]
}

interface Props {
  positions: Position[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'add-position': []
  'rebalance-portfolio': []
  'add-to-position': [position: Position]
  'reduce-position': [position: Position]
  'close-position': [position: Position]
  'view-details': [position: Position]
}>()

const showFilters = ref(false)
const selectedPosition = ref<Position | null>(null)
const sortBy = ref('value')

const filters = ref({
  assetType: '',
  performance: '',
  sizeRange: ''
})

// Generate mock data if none provided
const generateMockPositions = (): Position[] => {
  const assets = [
    { symbol: 'ETH', name: 'Ethereum', type: 'crypto' },
    { symbol: 'BTC', name: 'Bitcoin', type: 'crypto' },
    { symbol: 'UNI', name: 'Uniswap', type: 'defi' },
    { symbol: 'AAVE', name: 'Aave', type: 'defi' },
    { symbol: 'COMP', name: 'Compound', type: 'defi' },
    { symbol: 'SUSHI', name: 'SushiSwap', type: 'defi' },
    { symbol: 'CRV', name: 'Curve', type: 'defi' },
    { symbol: 'MKR', name: 'Maker', type: 'defi' },
    { symbol: 'USDC', name: 'USD Coin', type: 'stablecoin' },
    { symbol: 'LINK', name: 'Chainlink', type: 'crypto' }
  ]

  return assets.map((asset, index) => {
    const allocation = index === 0 ? 25 : index === 1 ? 20 : 55 / (assets.length - 2)
    const value = (125000 * allocation) / 100
    const currentPrice = 1000 + Math.random() * 2000
    const avgCost = currentPrice * (0.8 + Math.random() * 0.4)
    const amount = value / currentPrice
    const pnl = (currentPrice - avgCost) * amount
    const pnlPercent = ((currentPrice - avgCost) / avgCost) * 100
    const priceChange24h = (Math.random() - 0.5) * 10

    const alerts: PositionAlert[] = []
    if (pnl < -value * 0.1) {
      alerts.push({
        id: '1',
        type: 'warning',
        message: 'Position down >10%'
      })
    }
    if (allocation > 30) {
      alerts.push({
        id: '2',
        type: 'info',
        message: 'Large allocation detected'
      })
    }

    return {
      symbol: asset.symbol,
      name: asset.name,
      amount,
      value,
      allocation,
      pnl,
      pnlPercent,
      currentPrice,
      avgCost,
      priceChange24h,
      status: 'active' as const,
      assetType: asset.type,
      alerts: alerts.length > 0 ? alerts : undefined
    }
  })
}

const positions = computed(() => props.positions.length ? props.positions : generateMockPositions())

const filteredPositions = computed(() => {
  let filtered = [...positions.value]

  // Apply filters
  if (filters.value.assetType) {
    filtered = filtered.filter(p => p.assetType === filters.value.assetType)
  }

  if (filters.value.performance) {
    switch (filters.value.performance) {
      case 'winners':
        filtered = filtered.filter(p => p.pnl > 0)
        break
      case 'losers':
        filtered = filtered.filter(p => p.pnl < 0)
        break
      case 'breakeven':
        filtered = filtered.filter(p => Math.abs(p.pnl) < p.value * 0.01)
        break
    }
  }

  if (filters.value.sizeRange) {
    switch (filters.value.sizeRange) {
      case 'large':
        filtered = filtered.filter(p => p.allocation > 5)
        break
      case 'medium':
        filtered = filtered.filter(p => p.allocation >= 1 && p.allocation <= 5)
        break
      case 'small':
        filtered = filtered.filter(p => p.allocation < 1)
        break
    }
  }

  // Apply sorting
  filtered.sort((a, b) => {
    switch (sortBy.value) {
      case 'value':
        return b.value - a.value
      case 'allocation':
        return b.allocation - a.allocation
      case 'pnl':
        return b.pnl - a.pnl
      case 'pnlPercent':
        return b.pnlPercent - a.pnlPercent
      case 'symbol':
        return a.symbol.localeCompare(b.symbol)
      default:
        return 0
    }
  })

  return filtered
})

// Methods
function formatAmount(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function getAssetIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    UNI: '/tokens/uni.svg',
    AAVE: '/tokens/aave.svg',
    COMP: '/tokens/comp.svg',
    SUSHI: '/tokens/sushi.svg',
    CRV: '/tokens/crv.svg',
    MKR: '/tokens/mkr.svg',
    USDC: '/tokens/usdc.svg',
    LINK: '/tokens/link.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function getStatusBadgeClass(status: string): string {
  switch (status) {
    case 'active': return 'bg-green-500/20 text-green-400'
    case 'watching': return 'bg-yellow-500/20 text-yellow-400'
    case 'closed': return 'bg-gray-500/20 text-gray-400'
    default: return 'bg-gray-500/20 text-gray-400'
  }
}

function getAlertClass(type: string): string {
  switch (type) {
    case 'warning': return 'bg-yellow-500/10 border border-yellow-500/30 text-yellow-400'
    case 'info': return 'bg-blue-500/10 border border-blue-500/30 text-blue-400'
    case 'success': return 'bg-green-500/10 border border-green-500/30 text-green-400'
    default: return 'bg-gray-500/10 border border-gray-500/30 text-gray-400'
  }
}

function getAlertIcon(type: string): string {
  switch (type) {
    case 'warning': return 'ExclamationTriangleIcon'
    case 'info': return 'InformationCircleIcon'
    case 'success': return 'CheckCircleIcon'
    default: return 'ExclamationCircleIcon'
  }
}

function getPositionChartPath(position: Position): string {
  // Generate simple mock chart path
  const points = Array.from({ length: 20 }, (_, i) => {
    const x = (i / 19) * 190 + 5
    const y = 20 + Math.sin(i / 3) * 8 + (position.pnlPercent > 0 ? -5 : 5)
    return `${i === 0 ? 'M' : 'L'} ${x} ${y}`
  })
  return points.join(' ')
}

function getPositionChartColor(pnlPercent: number): string {
  return pnlPercent >= 0 ? '#4ade80' : '#f87171'
}

function selectPosition(position: Position) {
  selectedPosition.value = position
}

function showPositionMenu(position: Position) {
  // TODO: Implement position menu
  console.log('Position menu for', position.symbol)
}

function resetFilters() {
  filters.value = {
    assetType: '',
    performance: '',
    sizeRange: ''
  }
  sortBy.value = 'value'
}
</script>

<style scoped>
/* Slide down animation */
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.3s ease;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-16px);
}

/* Position grid animations */
.position-grid-enter-active {
  transition: all 0.4s ease-out;
}

.position-grid-leave-active {
  transition: all 0.3s ease-in;
}

.position-grid-enter-from {
  opacity: 0;
  transform: scale(0.95) translateY(20px);
}

.position-grid-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(-20px);
}

.position-grid-move {
  transition: transform 0.4s ease;
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
.hover\:shadow-lg:hover {
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
}

.hover\:shadow-moby-500\/10:hover {
  box-shadow: 0 10px 15px -3px rgba(59, 130, 246, 0.1), 0 4px 6px -2px rgba(59, 130, 246, 0.05);
}
</style>