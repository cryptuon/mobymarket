<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ChartPieIcon" class="w-5 h-5 text-cyan-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Asset Allocation</h3>
            <p class="text-xs text-white/60">Portfolio distribution</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- View Toggle -->
          <div class="flex bg-slate-800/50 rounded-lg p-1">
            <button
              v-for="view in viewModes"
              :key="view.value"
              @click="viewMode = view.value"
              :class="[
                'px-3 py-1 rounded-md text-xs font-medium transition-all duration-200',
                viewMode === view.value
                  ? 'bg-moby-500 text-white'
                  : 'text-white/60 hover:text-white hover:bg-white/10'
              ]"
            >
              {{ view.label }}
            </button>
          </div>
        </div>
      </div>
    </template>

    <div class="space-y-6">
      <!-- Donut Chart View -->
      <div v-if="viewMode === 'donut'" class="flex flex-col lg:flex-row items-center space-y-6 lg:space-y-0 lg:space-x-8">
        <!-- Chart -->
        <div class="relative w-64 h-64 flex-shrink-0">
          <svg viewBox="0 0 200 200" class="w-full h-full transform -rotate-90">
            <!-- Background Circle -->
            <circle
              cx="100"
              cy="100"
              r="80"
              fill="none"
              stroke="rgba(255,255,255,0.1)"
              stroke-width="20"
            />

            <!-- Asset Segments -->
            <circle
              v-for="(segment, index) in chartSegments"
              :key="segment.asset"
              cx="100"
              cy="100"
              r="80"
              fill="none"
              :stroke="segment.color"
              stroke-width="20"
              :stroke-dasharray="`${segment.circumference} ${totalCircumference - segment.circumference}`"
              :stroke-dashoffset="segment.offset"
              :class="[
                'transition-all duration-500 cursor-pointer',
                selectedAsset === segment.asset ? 'stroke-[24]' : 'hover:stroke-[22]'
              ]"
              @click="selectAsset(segment.asset)"
            />

            <!-- Center Value -->
            <text
              x="100"
              y="95"
              text-anchor="middle"
              class="fill-white text-lg font-bold transform rotate-90"
              style="font-family: system-ui;"
            >
              ${{ formatCurrency(totalValue) }}
            </text>
            <text
              x="100"
              y="110"
              text-anchor="middle"
              class="fill-white/60 text-xs transform rotate-90"
              style="font-family: system-ui;"
            >
              Total Value
            </text>
          </svg>

          <!-- Hover Info -->
          <Transition
            name="info"
            enter-active-class="transition-all duration-150"
            enter-from-class="opacity-0 scale-95"
            enter-to-class="opacity-100 scale-100"
            leave-active-class="transition-all duration-100"
            leave-from-class="opacity-100 scale-100"
            leave-to-class="opacity-0 scale-95"
          >
            <div
              v-if="hoveredAsset"
              class="absolute top-2 left-2 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 min-w-32 pointer-events-none z-10"
            >
              <div class="text-sm font-semibold text-white mb-1">{{ hoveredAsset.asset }}</div>
              <div class="text-xs text-white/60">Value: ${{ formatCurrency(hoveredAsset.value) }}</div>
              <div class="text-xs text-white/60">Share: {{ hoveredAsset.percentage.toFixed(1) }}%</div>
              <div class="text-xs" :class="getChangeColor(hoveredAsset.change24h)">
                24h: {{ hoveredAsset.change24h >= 0 ? '+' : '' }}{{ hoveredAsset.change24h.toFixed(1) }}%
              </div>
            </div>
          </Transition>
        </div>

        <!-- Legend & Details -->
        <div class="flex-1 space-y-3">
          <div
            v-for="(asset, index) in sortedData"
            :key="asset.asset"
            class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all duration-200"
            :class="[
              selectedAsset === asset.asset
                ? 'bg-slate-700/50 border border-slate-500/50'
                : 'bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30'
            ]"
            @click="selectAsset(asset.asset)"
            @mouseenter="hoveredAsset = asset"
            @mouseleave="hoveredAsset = null"
          >
            <!-- Asset Info -->
            <div class="flex items-center space-x-3 flex-1">
              <div
                class="w-4 h-4 rounded-full flex-shrink-0"
                :style="{ backgroundColor: asset.color }"
              ></div>
              <div class="flex-1">
                <div class="flex items-center space-x-2">
                  <span class="text-white font-medium">{{ asset.asset }}</span>
                  <span class="text-xs text-white/60">{{ asset.percentage.toFixed(1) }}%</span>
                </div>
                <div class="text-xs text-white/60 mt-1">
                  ${{ formatCurrency(asset.value) }}
                </div>
              </div>
            </div>

            <!-- Performance -->
            <div class="text-right">
              <div :class="['text-sm font-medium', getChangeColor(asset.change24h)]">
                {{ asset.change24h >= 0 ? '+' : '' }}{{ asset.change24h.toFixed(1) }}%
              </div>
              <div class="text-xs text-white/60">24h</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Bar Chart View -->
      <div v-else-if="viewMode === 'bar'" class="space-y-4">
        <div
          v-for="(asset, index) in sortedData"
          :key="asset.asset"
          class="space-y-2"
        >
          <!-- Asset Header -->
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-3">
              <div
                class="w-3 h-3 rounded-full"
                :style="{ backgroundColor: asset.color }"
              ></div>
              <span class="text-white font-medium">{{ asset.asset }}</span>
              <span class="text-xs text-white/60">{{ asset.percentage.toFixed(1) }}%</span>
            </div>
            <div class="text-right">
              <div class="text-white font-semibold text-sm">${{ formatCurrency(asset.value) }}</div>
              <div :class="['text-xs', getChangeColor(asset.change24h)]">
                {{ asset.change24h >= 0 ? '+' : '' }}{{ asset.change24h.toFixed(1) }}%
              </div>
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="relative h-6 bg-slate-800/30 rounded-lg overflow-hidden">
            <div
              class="h-full rounded-lg transition-all duration-700 ease-out"
              :style="{
                width: `${asset.percentage}%`,
                backgroundColor: asset.color,
                opacity: selectedAsset === asset.asset ? 1 : 0.8
              }"
            ></div>

            <!-- Value Label -->
            <div class="absolute right-2 top-1/2 transform -translate-y-1/2">
              <span class="text-xs font-medium text-white">
                ${{ formatCurrency(asset.value) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Table View -->
      <div v-else class="overflow-hidden rounded-lg border border-slate-600/30">
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead class="bg-slate-800/50">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-white/70 uppercase tracking-wider">
                  Asset
                </th>
                <th class="px-4 py-3 text-right text-xs font-medium text-white/70 uppercase tracking-wider">
                  Value
                </th>
                <th class="px-4 py-3 text-right text-xs font-medium text-white/70 uppercase tracking-wider">
                  Allocation
                </th>
                <th class="px-4 py-3 text-right text-xs font-medium text-white/70 uppercase tracking-wider">
                  24h Change
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-600/30">
              <tr
                v-for="(asset, index) in sortedData"
                :key="asset.asset"
                class="hover:bg-slate-800/20 transition-colors cursor-pointer"
                @click="selectAsset(asset.asset)"
              >
                <td class="px-4 py-3">
                  <div class="flex items-center space-x-3">
                    <div
                      class="w-3 h-3 rounded-full"
                      :style="{ backgroundColor: asset.color }"
                    ></div>
                    <div>
                      <div class="text-white font-medium">{{ asset.asset }}</div>
                    </div>
                  </div>
                </td>
                <td class="px-4 py-3 text-right">
                  <div class="text-white font-semibold">${{ formatCurrency(asset.value) }}</div>
                </td>
                <td class="px-4 py-3 text-right">
                  <div class="text-white">{{ asset.percentage.toFixed(1) }}%</div>
                </td>
                <td class="px-4 py-3 text-right">
                  <span :class="['font-medium', getChangeColor(asset.change24h)]">
                    {{ asset.change24h >= 0 ? '+' : '' }}{{ asset.change24h.toFixed(1) }}%
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Asset Detail Panel -->
      <div v-if="selectedAsset" class="bg-slate-800/30 rounded-xl p-4 border border-slate-600/30">
        <h4 class="text-lg font-semibold text-white mb-3">{{ selectedAsset }} Details</h4>
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div>
            <div class="text-xs text-white/60">Current Value</div>
            <div class="text-xl font-bold text-white">
              ${{ formatCurrency(getAssetData(selectedAsset)?.value || 0) }}
            </div>
          </div>
          <div>
            <div class="text-xs text-white/60">Allocation</div>
            <div class="text-xl font-bold text-white">
              {{ getAssetData(selectedAsset)?.percentage.toFixed(1) }}%
            </div>
          </div>
          <div>
            <div class="text-xs text-white/60">24h Change</div>
            <div class="text-xl font-bold" :class="getChangeColor(getAssetData(selectedAsset)?.change24h || 0)">
              {{ (getAssetData(selectedAsset)?.change24h || 0) >= 0 ? '+' : '' }}{{ (getAssetData(selectedAsset)?.change24h || 0).toFixed(1) }}%
            </div>
          </div>
          <div>
            <div class="text-xs text-white/60">Weight Target</div>
            <div class="text-xl font-bold text-white">
              {{ getTargetAllocation(selectedAsset) }}%
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <span>{{ data.length }} assets tracked</span>
        <div class="flex items-center space-x-2">
          <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
          <span>Live prices</span>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface AssetData {
  asset: string
  value: number
  percentage: number
  color: string
  change24h: number
}

interface Props {
  data: AssetData[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'asset-selected': [asset: string]
  'rebalance': []
}>()

const viewMode = ref<'donut' | 'bar' | 'table'>('donut')
const selectedAsset = ref<string | null>(null)
const hoveredAsset = ref<AssetData | null>(null)

const viewModes = [
  { value: 'donut', label: 'Chart' },
  { value: 'bar', label: 'Bars' },
  { value: 'table', label: 'Table' }
]

// Computed properties
const totalValue = computed(() => {
  return props.data.reduce((sum, asset) => sum + asset.value, 0)
})

const sortedData = computed(() => {
  return [...props.data].sort((a, b) => b.percentage - a.percentage)
})

const totalCircumference = 2 * Math.PI * 80 // radius = 80

const chartSegments = computed(() => {
  let currentOffset = 0

  return sortedData.value.map(asset => {
    const circumference = (asset.percentage / 100) * totalCircumference
    const segment = {
      asset: asset.asset,
      color: asset.color,
      circumference,
      offset: -currentOffset
    }
    currentOffset += circumference
    return segment
  })
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(2)
}

function getChangeColor(change: number): string {
  return change >= 0 ? 'text-green-400' : 'text-red-400'
}

function selectAsset(asset: string) {
  selectedAsset.value = selectedAsset.value === asset ? null : asset
  emit('asset-selected', asset)
}

function getAssetData(asset: string): AssetData | undefined {
  return props.data.find(a => a.asset === asset)
}

function getTargetAllocation(asset: string): number {
  // Mock target allocations - in real app would come from props or API
  const targets: Record<string, number> = {
    'ETH': 40,
    'BTC': 35,
    'USDC': 20,
    'UNI': 3,
    'AAVE': 2
  }
  return targets[asset] || 0
}
</script>

<style scoped>
/* Chart transitions */
svg circle {
  transition: stroke-width 0.3s ease;
}

/* Info panel animations */
.info-enter-active,
.info-leave-active {
  transition: all 0.15s ease;
}

.info-enter-from,
.info-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

/* Progress bar animations */
.progress-bar {
  transition: width 0.7s ease-out;
}

/* Table hover effects */
tbody tr:hover {
  background-color: rgba(30, 41, 59, 0.2);
}
</style>