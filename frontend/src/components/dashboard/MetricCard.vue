<template>
  <Card variant="glass" :glow="highlight">
    <div class="p-6">
      <div class="flex items-center justify-between">
        <!-- Icon -->
        <div :class="iconBackgroundClass" class="w-12 h-12 rounded-xl flex items-center justify-center">
          <HeroIcon :name="icon" class="w-6 h-6" :class="iconColorClass" />
        </div>

        <!-- Change Indicator -->
        <div v-if="change !== undefined" :class="changeClass" class="flex items-center space-x-1 px-2 py-1 rounded-lg text-xs font-semibold">
          <HeroIcon
            :name="change >= 0 ? 'ArrowTrendingUpIcon' : 'ArrowTrendingDownIcon'"
            class="w-4 h-4"
          />
          <span>{{ Math.abs(change).toFixed(1) }}%</span>
        </div>
      </div>

      <div class="mt-4">
        <!-- Value -->
        <div class="text-2xl font-bold text-white">{{ displayValue }}</div>

        <!-- Title -->
        <div class="text-sm text-white/60 mt-1">{{ title }}</div>

        <!-- Additional Info -->
        <div v-if="subtitle" class="text-xs text-white/40 mt-2">{{ subtitle }}</div>
      </div>

      <!-- Trend Sparkline (optional) -->
      <div v-if="trendData && trendData.length > 0" class="mt-4">
        <div class="h-8 flex items-end space-x-1">
          <div
            v-for="(point, index) in normalizedTrendData"
            :key="index"
            :class="trendBarClass"
            :style="{ height: `${point}%` }"
            class="flex-1 rounded-sm min-h-1"
          ></div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface Props {
  title: string
  value: string | number
  change?: number
  icon: string
  color?: 'blue' | 'green' | 'purple' | 'orange' | 'red' | 'yellow'
  subtitle?: string
  highlight?: boolean
  trendData?: number[]
  formatValue?: (value: number) => string
}

const props = withDefaults(defineProps<Props>(), {
  color: 'blue',
  highlight: false
})

// Computed properties
const displayValue = computed(() => {
  if (typeof props.value === 'string') return props.value

  if (props.formatValue) {
    return props.formatValue(props.value)
  }

  // Default number formatting
  if (props.value >= 1e9) return `${(props.value / 1e9).toFixed(2)}B`
  if (props.value >= 1e6) return `${(props.value / 1e6).toFixed(2)}M`
  if (props.value >= 1e3) return `${(props.value / 1e3).toFixed(2)}K`
  return props.value.toLocaleString()
})

const iconBackgroundClass = computed(() => {
  const colorMap = {
    blue: 'bg-blue-500/20',
    green: 'bg-green-500/20',
    purple: 'bg-purple-500/20',
    orange: 'bg-orange-500/20',
    red: 'bg-red-500/20',
    yellow: 'bg-yellow-500/20'
  }
  return colorMap[props.color]
})

const iconColorClass = computed(() => {
  const colorMap = {
    blue: 'text-blue-400',
    green: 'text-green-400',
    purple: 'text-purple-400',
    orange: 'text-orange-400',
    red: 'text-red-400',
    yellow: 'text-yellow-400'
  }
  return colorMap[props.color]
})

const changeClass = computed(() => {
  if (props.change === undefined) return ''

  if (props.change >= 0) {
    return 'bg-green-500/20 text-green-400'
  } else {
    return 'bg-red-500/20 text-red-400'
  }
})

const trendBarClass = computed(() => {
  return props.change !== undefined && props.change >= 0
    ? 'bg-green-400/60'
    : 'bg-red-400/60'
})

const normalizedTrendData = computed(() => {
  if (!props.trendData || props.trendData.length === 0) return []

  const min = Math.min(...props.trendData)
  const max = Math.max(...props.trendData)
  const range = max - min

  if (range === 0) return props.trendData.map(() => 50)

  return props.trendData.map(value => {
    const normalized = ((value - min) / range) * 80 + 20 // 20-100% range
    return Math.max(10, normalized) // Minimum 10% height
  })
})
</script>