<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="BeakerIcon" class="w-5 h-5 text-blue-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Liquidity Pools</h3>
            <p class="text-xs text-white/60">Pool positions and yields</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <select
            v-model="selectedProtocol"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="all">All Protocols</option>
            <option value="uniswap">Uniswap</option>
            <option value="sushiswap">SushiSwap</option>
            <option value="curve">Curve</option>
            <option value="balancer">Balancer</option>
          </select>

          <div class="flex bg-slate-800/50 rounded-lg p-1">
            <button
              v-for="view in viewOptions"
              :key="view.value"
              @click="selectedView = view.value"
              :class="[
                'px-2 py-1 text-xs rounded transition-all',
                selectedView === view.value
                  ? 'bg-moby-500 text-white'
                  : 'text-white/60 hover:text-white'
              ]"
            >
              {{ view.label }}
            </button>
          </div>
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Loading State -->
      <div v-if="loading" class="space-y-3">
        <div v-for="i in 4" :key="i" class="animate-pulse">
          <div class="flex items-center space-x-3 p-4 bg-slate-800/30 rounded-lg">
            <div class="flex space-x-2">
              <div class="w-8 h-8 bg-slate-700/50 rounded-full"></div>
              <div class="w-8 h-8 bg-slate-700/50 rounded-full -ml-3"></div>
            </div>
            <div class="flex-1 space-y-2">
              <div class="h-4 bg-slate-700/50 rounded w-3/4"></div>
              <div class="h-3 bg-slate-700/50 rounded w-1/2"></div>
            </div>
            <div class="h-4 bg-slate-700/50 rounded w-16"></div>
          </div>
        </div>
      </div>

      <div v-else class="space-y-6">
        <!-- Pool Overview -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="CurrencyDollarIcon" class="w-4 h-4 text-green-400" />
              <span class="text-xs text-white/60">Total Value</span>
            </div>
            <div class="text-xl font-bold text-white">${{ formatAmount(poolStats.totalValue) }}</div>
            <div :class="['text-xs', getChangeColor(poolStats.valueChange)]">
              {{ poolStats.valueChange >= 0 ? '+' : '' }}{{ poolStats.valueChange.toFixed(1) }}%
            </div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="SparklesIcon" class="w-4 h-4 text-yellow-400" />
              <span class="text-xs text-white/60">Total Rewards</span>
            </div>
            <div class="text-xl font-bold text-green-400">${{ formatAmount(poolStats.totalRewards) }}</div>
            <div class="text-xs text-white/60">7-day earnings</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="ChartBarIcon" class="w-4 h-4 text-blue-400" />
              <span class="text-xs text-white/60">Avg APY</span>
            </div>
            <div class="text-xl font-bold text-blue-400">{{ poolStats.avgApy.toFixed(1) }}%</div>
            <div class="text-xs text-white/60">Weighted average</div>
          </div>

          <div class="bg-slate-800/30 rounded-lg p-4 text-center">
            <div class="flex items-center justify-center space-x-2 mb-2">
              <HeroIcon name="ExclamationTriangleIcon" class="w-4 h-4 text-orange-400" />
              <span class="text-xs text-white/60">Impermanent Loss</span>
            </div>
            <div :class="['text-xl font-bold', getILColor(poolStats.impermanentLoss)]">
              {{ poolStats.impermanentLoss.toFixed(2) }}%
            </div>
            <div class="text-xs text-white/60">Estimated</div>
          </div>
        </div>

        <!-- Pool Positions -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <h4 class="text-sm font-semibold text-white">Active Positions</h4>
            <Button
              variant="ghost"
              size="xs"
              icon-right="PlusIcon"
              @click="$emit('add-liquidity')"
            >
              Add Liquidity
            </Button>
          </div>

          <div class="space-y-3">
            <div
              v-for="pool in filteredPools"
              :key="pool.id"
              class="bg-slate-800/30 hover:bg-slate-700/40 rounded-lg p-4 transition-all duration-200 cursor-pointer"
              @click="selectPool(pool)"
            >
              <div class="flex items-center justify-between">
                <!-- Pool Info -->
                <div class="flex items-center space-x-3">
                  <!-- Token Pair Icons -->
                  <div class="flex items-center -space-x-2">
                    <img
                      :src="getAssetIcon(pool.token0)"
                      :alt="pool.token0"
                      class="w-8 h-8 rounded-full border-2 border-slate-800 z-10"
                    />
                    <img
                      :src="getAssetIcon(pool.token1)"
                      :alt="pool.token1"
                      class="w-8 h-8 rounded-full border-2 border-slate-800"
                    />
                  </div>

                  <div>
                    <div class="flex items-center space-x-2">
                      <span class="text-sm font-medium text-white">{{ pool.token0 }}/{{ pool.token1 }}</span>
                      <div :class="getProtocolBadgeClass(pool.protocol)" class="px-2 py-0.5 rounded text-xs font-medium">
                        {{ pool.protocol }}
                      </div>
                      <div :class="getFeeTierClass(pool.feeTier)" class="px-2 py-0.5 rounded text-xs">
                        {{ pool.feeTier }}% fee
                      </div>
                    </div>
                    <div class="text-xs text-white/60">Position: ${{ formatAmount(pool.position) }}</div>
                  </div>
                </div>

                <!-- Pool Metrics -->
                <div class="flex items-center space-x-6">
                  <!-- APY -->
                  <div class="text-right">
                    <div class="text-xs text-white/60">APY</div>
                    <div :class="['text-sm font-bold', getAPYColor(pool.apy)]">
                      {{ pool.apy.toFixed(1) }}%
                    </div>
                  </div>

                  <!-- 24h Fees -->
                  <div class="text-right">
                    <div class="text-xs text-white/60">24h Fees</div>
                    <div class="text-sm font-bold text-green-400">
                      ${{ formatAmount(pool.fees24h) }}
                    </div>
                  </div>

                  <!-- P&L -->
                  <div class="text-right">
                    <div class="text-xs text-white/60">P&L</div>
                    <div :class="['text-sm font-bold', getChangeColor(pool.pnl)]">
                      {{ pool.pnl >= 0 ? '+' : '' }}${{ formatAmount(Math.abs(pool.pnl)) }}
                    </div>
                  </div>

                  <!-- Action Arrow -->
                  <HeroIcon name="ChevronRightIcon" class="w-4 h-4 text-white/40" />
                </div>
              </div>

              <!-- Pool Details (expanded view) -->
              <div v-if="selectedView === 'detailed'" class="mt-4 pt-4 border-t border-white/10">
                <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                  <div>
                    <div class="text-xs text-white/60">TVL</div>
                    <div class="text-sm font-medium text-white">${{ formatAmount(pool.tvl) }}</div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60">Volume 24h</div>
                    <div class="text-sm font-medium text-white">${{ formatAmount(pool.volume24h) }}</div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60">Your Share</div>
                    <div class="text-sm font-medium text-white">{{ pool.sharePercent.toFixed(3) }}%</div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60">IL Risk</div>
                    <div :class="['text-sm font-medium', getILRiskColor(pool.ilRisk)]">
                      {{ pool.ilRisk }}
                    </div>
                  </div>
                </div>

                <!-- Token Balances -->
                <div class="mt-4 grid grid-cols-2 gap-4">
                  <div class="bg-slate-700/30 rounded-lg p-3">
                    <div class="flex items-center space-x-2 mb-2">
                      <img :src="getAssetIcon(pool.token0)" :alt="pool.token0" class="w-4 h-4 rounded-full" />
                      <span class="text-xs text-white/60">{{ pool.token0 }} Balance</span>
                    </div>
                    <div class="text-sm font-bold text-white">{{ formatAmount(pool.balance0) }}</div>
                    <div class="text-xs text-white/60">${{ formatAmount(pool.balance0 * pool.price0) }}</div>
                  </div>
                  <div class="bg-slate-700/30 rounded-lg p-3">
                    <div class="flex items-center space-x-2 mb-2">
                      <img :src="getAssetIcon(pool.token1)" :alt="pool.token1" class="w-4 h-4 rounded-full" />
                      <span class="text-xs text-white/60">{{ pool.token1 }} Balance</span>
                    </div>
                    <div class="text-sm font-bold text-white">{{ formatAmount(pool.balance1) }}</div>
                    <div class="text-xs text-white/60">${{ formatAmount(pool.balance1 * pool.price1) }}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Yield Opportunities -->
        <div v-if="selectedView === 'opportunities'" class="space-y-3">
          <h4 class="text-sm font-semibold text-white">High Yield Opportunities</h4>
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
            <div
              v-for="opportunity in yieldOpportunities"
              :key="opportunity.id"
              class="bg-slate-800/20 rounded-lg p-4 border border-white/10 hover:border-moby-500/50 transition-all cursor-pointer"
              @click="$emit('add-liquidity', opportunity)"
            >
              <div class="flex items-center justify-between mb-3">
                <div class="flex items-center space-x-2">
                  <div class="flex items-center -space-x-1">
                    <img :src="getAssetIcon(opportunity.token0)" :alt="opportunity.token0" class="w-6 h-6 rounded-full border border-slate-700" />
                    <img :src="getAssetIcon(opportunity.token1)" :alt="opportunity.token1" class="w-6 h-6 rounded-full border border-slate-700" />
                  </div>
                  <span class="text-sm font-medium text-white">{{ opportunity.token0 }}/{{ opportunity.token1 }}</span>
                </div>
                <div :class="getAPYColor(opportunity.apy)" class="text-lg font-bold">
                  {{ opportunity.apy.toFixed(1) }}%
                </div>
              </div>

              <div class="space-y-2">
                <div class="flex justify-between text-xs">
                  <span class="text-white/60">Protocol</span>
                  <span class="text-white">{{ opportunity.protocol }}</span>
                </div>
                <div class="flex justify-between text-xs">
                  <span class="text-white/60">TVL</span>
                  <span class="text-white">${{ formatAmount(opportunity.tvl) }}</span>
                </div>
                <div class="flex justify-between text-xs">
                  <span class="text-white/60">Risk Level</span>
                  <span :class="getRiskLevelColor(opportunity.riskLevel)">{{ opportunity.riskLevel }}</span>
                </div>
              </div>

              <div class="mt-3 pt-3 border-t border-white/10">
                <div class="flex items-center justify-between text-xs">
                  <span class="text-white/60">Rewards breakdown</span>
                  <div class="flex space-x-2">
                    <span v-for="reward in opportunity.rewards" :key="reward.token" class="text-white">
                      {{ reward.apy.toFixed(1) }}% {{ reward.token }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Pool Analytics Chart -->
        <div v-if="selectedView === 'analytics'" class="space-y-4">
          <h4 class="text-sm font-semibold text-white">Pool Performance Analytics</h4>
          <div class="bg-slate-800/20 rounded-lg p-4">
            <div class="h-48 flex items-center justify-center text-white/60">
              <div class="text-center">
                <HeroIcon name="ChartBarIcon" class="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p class="text-sm">Pool analytics chart would be implemented here</p>
                <p class="text-xs mt-1">Showing fees earned, IL tracking, and yield curves</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Pool Detail Modal -->
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
        v-if="selectedPool"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
        @click="selectedPool = null"
      >
        <div
          class="bg-slate-800/90 backdrop-blur border border-white/20 rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto"
          @click.stop
        >
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center space-x-3">
              <div class="flex items-center -space-x-2">
                <img :src="getAssetIcon(selectedPool.token0)" :alt="selectedPool.token0" class="w-8 h-8 rounded-full border-2 border-slate-800" />
                <img :src="getAssetIcon(selectedPool.token1)" :alt="selectedPool.token1" class="w-8 h-8 rounded-full border-2 border-slate-800" />
              </div>
              <div>
                <h4 class="text-lg font-semibold text-white">{{ selectedPool.token0 }}/{{ selectedPool.token1 }}</h4>
                <p class="text-sm text-white/60">{{ selectedPool.protocol }} Pool</p>
              </div>
            </div>
            <button
              @click="selectedPool = null"
              class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
            </button>
          </div>

          <div class="grid grid-cols-2 gap-6 mb-6">
            <div class="space-y-4">
              <div>
                <div class="text-xs text-white/60 mb-1">Position Value</div>
                <div class="text-2xl font-bold text-white">${{ formatAmount(selectedPool.position) }}</div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">Current APY</div>
                <div :class="['text-xl font-bold', getAPYColor(selectedPool.apy)]">
                  {{ selectedPool.apy.toFixed(2) }}%
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">Fees Earned (24h)</div>
                <div class="text-xl font-bold text-green-400">${{ formatAmount(selectedPool.fees24h) }}</div>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <div class="text-xs text-white/60 mb-1">Impermanent Loss</div>
                <div :class="['text-xl font-bold', getILColor(selectedPool.il || 0)]">
                  {{ (selectedPool.il || 0).toFixed(2) }}%
                </div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">Your Pool Share</div>
                <div class="text-xl font-bold text-white">{{ selectedPool.sharePercent.toFixed(4) }}%</div>
              </div>
              <div>
                <div class="text-xs text-white/60 mb-1">Total P&L</div>
                <div :class="['text-xl font-bold', getChangeColor(selectedPool.pnl)]">
                  {{ selectedPool.pnl >= 0 ? '+' : '' }}${{ formatAmount(Math.abs(selectedPool.pnl)) }}
                </div>
              </div>
            </div>
          </div>

          <div class="flex space-x-3">
            <Button
              variant="primary"
              size="sm"
              class="flex-1"
              @click="$emit('add-liquidity', selectedPool)"
            >
              Add Liquidity
            </Button>
            <Button
              variant="secondary"
              size="sm"
              class="flex-1"
              @click="$emit('remove-liquidity', selectedPool)"
            >
              Remove Liquidity
            </Button>
            <Button
              variant="ghost"
              size="sm"
              @click="$emit('claim-rewards', selectedPool)"
            >
              Claim Rewards
            </Button>
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

interface PoolPosition {
  id: string
  token0: string
  token1: string
  protocol: string
  feeTier: number
  position: number
  apy: number
  fees24h: number
  pnl: number
  tvl: number
  volume24h: number
  sharePercent: number
  ilRisk: string
  balance0: number
  balance1: number
  price0: number
  price1: number
  il?: number
}

interface YieldOpportunity {
  id: string
  token0: string
  token1: string
  protocol: string
  apy: number
  tvl: number
  riskLevel: string
  rewards: Array<{ token: string; apy: number }>
}

interface Props {
  data: PoolPosition[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'add-liquidity': [pool?: PoolPosition | YieldOpportunity]
  'remove-liquidity': [pool: PoolPosition]
  'claim-rewards': [pool: PoolPosition]
}>()

const selectedProtocol = ref('all')
const selectedView = ref('positions')
const selectedPool = ref<PoolPosition | null>(null)

const viewOptions = [
  { label: 'Positions', value: 'positions' },
  { label: 'Detailed', value: 'detailed' },
  { label: 'Opportunities', value: 'opportunities' },
  { label: 'Analytics', value: 'analytics' }
]

// Generate mock data
const generateMockPools = (): PoolPosition[] => {
  const tokens = ['ETH', 'BTC', 'UNI', 'AAVE', 'COMP', 'USDC', 'USDT']
  const protocols = ['Uniswap', 'SushiSwap', 'Curve', 'Balancer']
  const feeTiers = [0.05, 0.3, 1.0]

  return Array.from({ length: 8 }, (_, i) => {
    const token0 = tokens[Math.floor(Math.random() * tokens.length)]
    let token1 = tokens[Math.floor(Math.random() * tokens.length)]
    while (token1 === token0) {
      token1 = tokens[Math.floor(Math.random() * tokens.length)]
    }

    return {
      id: `pool-${i}`,
      token0,
      token1,
      protocol: protocols[Math.floor(Math.random() * protocols.length)],
      feeTier: feeTiers[Math.floor(Math.random() * feeTiers.length)],
      position: 1000 + Math.random() * 50000,
      apy: 5 + Math.random() * 45,
      fees24h: 10 + Math.random() * 500,
      pnl: (Math.random() - 0.3) * 5000,
      tvl: 1000000 + Math.random() * 50000000,
      volume24h: 100000 + Math.random() * 5000000,
      sharePercent: Math.random() * 0.1,
      ilRisk: ['Low', 'Medium', 'High'][Math.floor(Math.random() * 3)],
      balance0: 10 + Math.random() * 100,
      balance1: 10 + Math.random() * 100,
      price0: 1000 + Math.random() * 2000,
      price1: 1000 + Math.random() * 2000,
      il: (Math.random() - 0.8) * 10
    }
  })
}

const poolPositions = computed(() => props.data.length ? props.data : generateMockPools())

const filteredPools = computed(() => {
  if (selectedProtocol.value === 'all') return poolPositions.value
  return poolPositions.value.filter(pool =>
    pool.protocol.toLowerCase() === selectedProtocol.value
  )
})

const poolStats = computed(() => {
  const pools = filteredPools.value
  const totalValue = pools.reduce((sum, pool) => sum + pool.position, 0)
  const totalRewards = pools.reduce((sum, pool) => sum + pool.fees24h * 7, 0)
  const weightedApy = pools.reduce((sum, pool) => sum + (pool.apy * pool.position), 0) / totalValue
  const avgIL = pools.reduce((sum, pool) => sum + (pool.il || 0), 0) / pools.length

  return {
    totalValue,
    valueChange: Math.random() * 10 - 2,
    totalRewards,
    avgApy: weightedApy,
    impermanentLoss: avgIL
  }
})

const yieldOpportunities = computed((): YieldOpportunity[] => [
  {
    id: 'opp-1',
    token0: 'ETH',
    token1: 'USDC',
    protocol: 'Uniswap V3',
    apy: 68.5,
    tvl: 125000000,
    riskLevel: 'Medium',
    rewards: [
      { token: 'LP', apy: 45.2 },
      { token: 'UNI', apy: 23.3 }
    ]
  },
  {
    id: 'opp-2',
    token0: 'BTC',
    token1: 'ETH',
    protocol: 'SushiSwap',
    apy: 72.1,
    tvl: 89000000,
    riskLevel: 'High',
    rewards: [
      { token: 'LP', apy: 52.8 },
      { token: 'SUSHI', apy: 19.3 }
    ]
  },
  {
    id: 'opp-3',
    token0: 'USDC',
    token1: 'USDT',
    protocol: 'Curve',
    apy: 35.7,
    tvl: 450000000,
    riskLevel: 'Low',
    rewards: [
      { token: 'LP', apy: 28.4 },
      { token: 'CRV', apy: 7.3 }
    ]
  },
  {
    id: 'opp-4',
    token0: 'UNI',
    token1: 'ETH',
    protocol: 'Balancer',
    apy: 45.9,
    tvl: 67000000,
    riskLevel: 'Medium',
    rewards: [
      { token: 'LP', apy: 35.1 },
      { token: 'BAL', apy: 10.8 }
    ]
  }
])

// Methods
function formatAmount(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function getAssetIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    UNI: '/tokens/uni.svg',
    AAVE: '/tokens/aave.svg',
    COMP: '/tokens/comp.svg',
    USDC: '/tokens/usdc.svg',
    USDT: '/tokens/usdt.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function getProtocolBadgeClass(protocol: string): string {
  const protocolColors: Record<string, string> = {
    'Uniswap': 'bg-pink-500/20 text-pink-400',
    'SushiSwap': 'bg-blue-500/20 text-blue-400',
    'Curve': 'bg-yellow-500/20 text-yellow-400',
    'Balancer': 'bg-purple-500/20 text-purple-400'
  }
  return protocolColors[protocol] || 'bg-gray-500/20 text-gray-400'
}

function getFeeTierClass(feeTier: number): string {
  if (feeTier <= 0.05) return 'bg-green-500/20 text-green-400'
  if (feeTier <= 0.3) return 'bg-yellow-500/20 text-yellow-400'
  return 'bg-red-500/20 text-red-400'
}

function getAPYColor(apy: number): string {
  if (apy >= 50) return 'text-green-400'
  if (apy >= 20) return 'text-yellow-400'
  return 'text-red-400'
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function getILColor(il: number): string {
  if (il <= -5) return 'text-red-400'
  if (il <= 0) return 'text-yellow-400'
  return 'text-green-400'
}

function getILRiskColor(risk: string): string {
  switch (risk) {
    case 'Low': return 'text-green-400'
    case 'Medium': return 'text-yellow-400'
    case 'High': return 'text-red-400'
    default: return 'text-gray-400'
  }
}

function getRiskLevelColor(risk: string): string {
  return getILRiskColor(risk)
}

function selectPool(pool: PoolPosition) {
  selectedPool.value = pool
}
</script>

<style scoped>
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
.hover\:bg-slate-700\/40:hover {
  background-color: rgba(51, 65, 85, 0.4);
}

/* Opportunity card hover */
.hover\:border-moby-500\/50:hover {
  border-color: rgba(59, 130, 246, 0.5);
}
</style>