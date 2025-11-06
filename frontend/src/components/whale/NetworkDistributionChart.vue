<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="ChartPieIcon" class="w-5 h-5 text-cyan-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Network Distribution</h3>
            <p class="text-xs text-white/60">Whale activity by blockchain</p>
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

          <!-- Sort Option -->
          <select
            v-model="sortBy"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
          >
            <option value="volume">Volume</option>
            <option value="percentage">Percentage</option>
            <option value="growth">Growth</option>
            <option value="whales">Whale Count</option>
          </select>
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
              stroke-width="16"
            />

            <!-- Network Segments -->
            <circle
              v-for="(segment, index) in chartSegments"
              :key="segment.network"
              cx="100"
              cy="100"
              r="80"
              fill="none"
              :stroke="segment.color"
              stroke-width="16"
              :stroke-dasharray="`${segment.circumference} ${totalCircumference - segment.circumference}`"
              :stroke-dashoffset="segment.offset"
              :class="[
                'transition-all duration-500 cursor-pointer',
                selectedNetwork === segment.network ? 'stroke-[20]' : 'hover:stroke-[18]'
              ]"
              @click="selectNetwork(segment.network)"
            />

            <!-- Center Value -->
            <text
              x="100"
              y="95"
              text-anchor="middle"
              class="fill-white text-lg font-bold transform rotate-90"
              style="font-family: system-ui;"
            >
              ${{ formatCurrency(totalVolume) }}
            </text>
            <text
              x="100"
              y="110"
              text-anchor="middle"
              class="fill-white/60 text-xs transform rotate-90"
              style="font-family: system-ui;"
            >
              Total Volume
            </text>
          </svg>

          <!-- Hover Tooltip -->
          <Transition
            name="tooltip"
            enter-active-class="transition-all duration-150"
            enter-from-class="opacity-0 scale-95"
            enter-to-class="opacity-100 scale-100"
            leave-active-class="transition-all duration-100"
            leave-from-class="opacity-100 scale-100"
            leave-to-class="opacity-0 scale-95"
          >
            <div
              v-if="hoveredNetwork"
              class="absolute top-2 left-2 bg-slate-800/90 backdrop-blur border border-white/20 rounded-lg p-3 min-w-32 pointer-events-none z-10"
            >
              <div class="text-sm font-semibold text-white mb-1">{{ hoveredNetwork.network }}</div>
              <div class="text-xs text-white/60">Volume: ${{ formatCurrency(hoveredNetwork.volume) }}</div>
              <div class="text-xs text-white/60">Share: {{ hoveredNetwork.percentage.toFixed(1) }}%</div>
              <div v-if="hoveredNetwork.whales" class="text-xs text-white/60">
                Whales: {{ hoveredNetwork.whales }}
              </div>
            </div>
          </Transition>
        </div>

        <!-- Legend & Stats -->
        <div class="flex-1 space-y-3">
          <div
            v-for="(network, index) in sortedData"
            :key="network.network"
            class="flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all duration-200"
            :class="[
              selectedNetwork === network.network
                ? 'bg-slate-700/50 border border-slate-500/50'
                : 'bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30'
            ]"
            @click="selectNetwork(network.network)"
            @mouseenter="hoveredNetwork = network"
            @mouseleave="hoveredNetwork = null"
          >
            <!-- Network Info -->
            <div class="flex items-center space-x-3 flex-1">
              <div
                class="w-4 h-4 rounded-full flex-shrink-0"
                :style="{ backgroundColor: network.color }"
              ></div>
              <div class="flex-1">
                <div class="flex items-center space-x-2">
                  <span class="text-white font-medium">{{ network.network }}</span>
                  <span class="text-xs text-white/60">#{index + 1}</span>
                </div>
                <div class="text-xs text-white/60 mt-1">
                  {{ network.percentage.toFixed(1) }}% of total volume
                </div>
              </div>
            </div>

            <!-- Metrics -->
            <div class="text-right space-y-1">
              <div class="text-white font-semibold">${{ formatCurrency(network.volume) }}</div>
              <div class="flex items-center space-x-2 text-xs">
                <span :class="getGrowthColor(network.growth)" class="font-medium">
                  {{ network.growth >= 0 ? '+' : '' }}{{ network.growth?.toFixed(1) || 0 }}%
                </span>
                <span class="text-white/60">{{ network.whales || 0 }} whales</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Bar Chart View -->
      <div v-else-if="viewMode === 'bar'" class="space-y-4">
        <div
          v-for="(network, index) in sortedData"
          :key="network.network"
          class="space-y-2"
        >
          <!-- Network Header -->
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-3">
              <div
                class="w-3 h-3 rounded-full"
                :style="{ backgroundColor: network.color }"
              ></div>
              <span class="text-white font-medium">{{ network.network }}</span>
              <span class="text-xs text-white/60">{{ network.percentage.toFixed(1) }}%</span>
            </div>
            <div class="text-right">
              <div class="text-white font-semibold text-sm">${{ formatCurrency(network.volume) }}</div>
              <div class="text-xs text-white/60">{{ network.whales || 0 }} whales</div>
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="relative h-6 bg-slate-800/30 rounded-lg overflow-hidden">
            <div
              class="h-full rounded-lg transition-all duration-700 ease-out"
              :style="{
                width: `${network.percentage}%`,
                backgroundColor: network.color,
                opacity: selectedNetwork === network.network ? 1 : 0.7
              }"
            ></div>

            <!-- Growth Indicator -->
            <div
              v-if="network.growth !== undefined"
              class="absolute right-2 top-1/2 transform -translate-y-1/2"
            >
              <span
                :class="['text-xs font-medium', getGrowthColor(network.growth)]"
              >
                {{ network.growth >= 0 ? '+' : '' }}{{ network.growth.toFixed(1) }}%
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
                  Network
                </th>
                <th class="px-4 py-3 text-right text-xs font-medium text-white/70 uppercase tracking-wider">
                  Volume
                </th>
                <th class="px-4 py-3 text-right text-xs font-medium text-white/70 uppercase tracking-wider">
                  Share
                </th>
                <th class="px-4 py-3 text-right text-xs font-medium text-white/70 uppercase tracking-wider">
                  Growth
                </th>
                <th class="px-4 py-3 text-right text-xs font-medium text-white/70 uppercase tracking-wider">
                  Whales
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-600/30">
              <tr
                v-for="(network, index) in sortedData"
                :key="network.network"
                class="hover:bg-slate-800/20 transition-colors cursor-pointer"
                @click="selectNetwork(network.network)"
              >
                <td class="px-4 py-3">
                  <div class="flex items-center space-x-3">
                    <div
                      class="w-3 h-3 rounded-full"
                      :style="{ backgroundColor: network.color }"
                    ></div>
                    <div>
                      <div class="text-white font-medium">{{ network.network }}</div>
                      <div class="text-xs text-white/60">Rank #{{ index + 1 }}</div>
                    </div>
                  </div>
                </td>
                <td class="px-4 py-3 text-right">
                  <div class="text-white font-semibold">${{ formatCurrency(network.volume) }}</div>
                </td>
                <td class="px-4 py-3 text-right">
                  <div class="text-white">{{ network.percentage.toFixed(1) }}%</div>
                </td>
                <td class="px-4 py-3 text-right">
                  <span :class="['font-medium', getGrowthColor(network.growth)]">
                    {{ network.growth >= 0 ? '+' : '' }}{{ network.growth?.toFixed(1) || 0 }}%
                  </span>
                </td>
                <td class="px-4 py-3 text-right">
                  <div class="text-white">{{ network.whales || 0 }}</div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Network Comparison -->
      <div v-if="selectedNetwork" class="bg-slate-800/30 rounded-xl p-4 border border-slate-600/30">
        <h4 class="text-lg font-semibold text-white mb-3">{{ selectedNetwork }} Deep Dive</h4>
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div>
            <div class="text-xs text-white/60">Market Share</div>
            <div class="text-xl font-bold text-white">
              {{ getNetworkData(selectedNetwork)?.percentage.toFixed(1) }}%
            </div>
          </div>
          <div>
            <div class="text-xs text-white/60">Volume Rank</div>
            <div class="text-xl font-bold text-white">
              #{{ sortedData.findIndex(n => n.network === selectedNetwork) + 1 }}
            </div>
          </div>
          <div>
            <div class="text-xs text-white/60">Active Whales</div>
            <div class="text-xl font-bold text-white">
              {{ getNetworkData(selectedNetwork)?.whales || 0 }}
            </div>
          </div>
          <div>
            <div class="text-xs text-white/60">24h Growth</div>
            <div class="text-xl font-bold" :class="getGrowthColor(getNetworkData(selectedNetwork)?.growth || 0)">
              {{ getNetworkData(selectedNetwork)?.growth >= 0 ? '+' : '' }}{{ getNetworkData(selectedNetwork)?.growth?.toFixed(1) || 0 }}%
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <span>{{ data.length }} networks tracked</span>
        <div class="flex items-center space-x-2">
          <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
          <span>Live data</span>
        </div>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface NetworkData {
  network: string
  volume: number
  percentage: number
  color: string
  growth?: number
  whales?: number
}

interface Props {
  data: NetworkData[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'network-selected': [network: string]
}>()

const viewMode = ref<'donut' | 'bar' | 'table'>('donut')
const sortBy = ref('volume')
const selectedNetwork = ref<string | null>(null)
const hoveredNetwork = ref<NetworkData | null>(null)

const viewModes = [
  { value: 'donut', label: 'Chart' },
  { value: 'bar', label: 'Bars' },
  { value: 'table', label: 'Table' }
]

// Computed properties
const totalVolume = computed(() => {
  return props.data.reduce((sum, network) => sum + network.volume, 0)
})

const sortedData = computed(() => {
  const sorted = [...props.data].sort((a, b) => {
    switch (sortBy.value) {
      case 'percentage':
        return b.percentage - a.percentage
      case 'growth':
        return (b.growth || 0) - (a.growth || 0)
      case 'whales':
        return (b.whales || 0) - (a.whales || 0)
      default:
        return b.volume - a.volume
    }
  })
  return sorted
})

const totalCircumference = 2 * Math.PI * 80 // radius = 80

const chartSegments = computed(() => {
  let currentOffset = 0

  return sortedData.value.map(network => {
    const circumference = (network.percentage / 100) * totalCircumference
    const segment = {
      network: network.network,
      color: network.color,
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

function getGrowthColor(growth?: number): string {
  if (!growth) return 'text-white/60'
  return growth >= 0 ? 'text-green-400' : 'text-red-400'
}

function selectNetwork(network: string) {
  selectedNetwork.value = selectedNetwork.value === network ? null : network
  emit('network-selected', network)
}

function getNetworkData(network: string): NetworkData | undefined {
  return props.data.find(n => n.network === network)
}
</script>

<style scoped>
/* Chart transitions */
svg circle {
  transition: stroke-width 0.3s ease;
}

/* Tooltip animations */
.tooltip-enter-active,
.tooltip-leave-active {
  transition: all 0.15s ease;
}

.tooltip-enter-from,
.tooltip-leave-to {
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

/* Chart segment hover effects */
.chart-segment:hover {
  stroke-width: 18;
}
</style>