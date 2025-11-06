<template>
  <Card variant="glass" class="h-full">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <div class="relative">
            <HeroIcon name="EyeIcon" class="w-6 h-6 text-moby-400" />
            <div
              v-if="isConnected"
              class="absolute -top-1 -right-1 w-3 h-3 bg-green-400 rounded-full animate-pulse"
            ></div>
          </div>
          <div>
            <h3 class="text-lg font-semibold text-white">Live Whale Activity</h3>
            <p class="text-xs text-white/60">Real-time large transactions</p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Connection Status -->
          <div :class="connectionStatusClass" class="flex items-center space-x-1 px-2 py-1 rounded-lg text-xs font-medium">
            <div :class="statusDotClass" class="w-2 h-2 rounded-full"></div>
            <span>{{ connectionStatusText }}</span>
          </div>

          <!-- Settings -->
          <button
            @click="showSettings = !showSettings"
            class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            aria-label="Feed settings"
          >
            <HeroIcon name="Cog6ToothIcon" class="w-4 h-4 text-white/70" />
          </button>
        </div>
      </div>
    </template>

    <!-- Settings Panel -->
    <Transition
      name="settings-slide"
      enter-active-class="transition-all duration-200"
      enter-from-class="transform -translate-y-2 opacity-0"
      enter-to-class="transform translate-y-0 opacity-100"
      leave-active-class="transition-all duration-150"
      leave-from-class="transform translate-y-0 opacity-100"
      leave-to-class="transform -translate-y-2 opacity-0"
    >
      <div v-if="showSettings" class="border-b border-white/10 p-4 space-y-3">
        <!-- Minimum Value Filter -->
        <div class="flex items-center justify-between">
          <label class="text-sm text-white/80">Minimum Value</label>
          <select
            v-model="minValueFilter"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-sm focus:outline-none focus:border-moby-500/50"
          >
            <option value="0">All</option>
            <option value="100000">$100K+</option>
            <option value="500000">$500K+</option>
            <option value="1000000">$1M+</option>
            <option value="5000000">$5M+</option>
          </select>
        </div>

        <!-- Token Filter -->
        <div class="flex items-center justify-between">
          <label class="text-sm text-white/80">Token Filter</label>
          <select
            v-model="tokenFilter"
            class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-sm focus:outline-none focus:border-moby-500/50"
          >
            <option value="">All Tokens</option>
            <option value="ETH">ETH</option>
            <option value="BTC">BTC</option>
            <option value="USDC">USDC</option>
            <option value="USDT">USDT</option>
          </select>
        </div>

        <!-- Auto-scroll Toggle -->
        <div class="flex items-center justify-between">
          <label class="text-sm text-white/80">Auto-scroll</label>
          <Toggle v-model="autoScroll" size="sm" />
        </div>
      </div>
    </Transition>

    <!-- Activity Feed -->
    <div
      ref="feedContainer"
      class="space-y-2 max-h-96 overflow-y-auto"
      :class="{ 'opacity-50': !isConnected }"
    >
      <!-- Loading State -->
      <div v-if="isConnecting" class="flex items-center justify-center py-8">
        <div class="text-center">
          <div class="animate-spin rounded-full h-8 w-8 border-2 border-white/20 border-t-white mx-auto mb-2"></div>
          <p class="text-white/60 text-sm">Connecting to live feed...</p>
        </div>
      </div>

      <!-- No Connection -->
      <div v-else-if="!isConnected" class="flex items-center justify-center py-8">
        <div class="text-center">
          <HeroIcon name="WifiIcon" class="w-12 h-12 text-white/30 mx-auto mb-2" />
          <p class="text-white/60 text-sm mb-2">Connection lost</p>
          <Button @click="reconnect" variant="outline" size="sm">
            Reconnect
          </Button>
        </div>
      </div>

      <!-- Activity Items -->
      <TransitionGroup
        v-else
        name="activity-list"
        tag="div"
        class="space-y-2"
      >
        <div
          v-for="activity in filteredActivities"
          :key="activity.id"
          class="activity-item"
        >
          <WhaleActivityItem
            :activity="activity"
            :highlight="isRecentActivity(activity)"
            @click="$emit('activity-click', activity)"
          />
        </div>
      </TransitionGroup>

      <!-- Empty State -->
      <div v-if="isConnected && filteredActivities.length === 0" class="text-center py-8">
        <HeroIcon name="MagnifyingGlassIcon" class="w-12 h-12 text-white/30 mx-auto mb-2" />
        <p class="text-white/60 text-sm">No whale activity matching your filters</p>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-between text-xs text-white/50">
        <span>Last update: {{ formatLastUpdate(lastUpdate) }}</span>
        <span>{{ filteredActivities.length }} activities</span>
      </div>
    </template>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import Toggle from '@components/ui/Toggle.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'
import WhaleActivityItem from './WhaleActivityItem.vue'

import { useRealTimeData } from '@/composables/useRealTimeData'
import type { WhaleActivity } from '@/types'

const emit = defineEmits<{
  'activity-click': [activity: WhaleActivity]
}>()

const {
  isConnected,
  isConnecting,
  liveWhaleActivity,
  lastUpdate,
  connect
} = useRealTimeData()

const feedContainer = ref<HTMLElement>()
const showSettings = ref(false)
const autoScroll = ref(true)
const minValueFilter = ref<number>(0)
const tokenFilter = ref<string>('')
const recentActivityIds = ref<Set<string>>(new Set())

// Computed properties
const connectionStatusClass = computed(() => {
  if (isConnected.value) return 'bg-green-500/20 text-green-400'
  if (isConnecting.value) return 'bg-yellow-500/20 text-yellow-400'
  return 'bg-red-500/20 text-red-400'
})

const statusDotClass = computed(() => {
  if (isConnected.value) return 'bg-green-400 animate-pulse'
  if (isConnecting.value) return 'bg-yellow-400 animate-pulse'
  return 'bg-red-400'
})

const connectionStatusText = computed(() => {
  if (isConnected.value) return 'Live'
  if (isConnecting.value) return 'Connecting'
  return 'Offline'
})

const filteredActivities = computed(() => {
  return liveWhaleActivity.value.filter(activity => {
    // Value filter
    if (minValueFilter.value > 0 && activity.usdValue < minValueFilter.value) {
      return false
    }

    // Token filter
    if (tokenFilter.value && activity.token !== tokenFilter.value) {
      return false
    }

    return true
  })
})

// Methods
function reconnect() {
  connect()
}

function isRecentActivity(activity: WhaleActivity): boolean {
  return recentActivityIds.value.has(activity.id)
}

function formatLastUpdate(timestamp: string): string {
  if (!timestamp) return 'Never'

  const diff = Date.now() - new Date(timestamp).getTime()
  const seconds = Math.floor(diff / 1000)

  if (seconds < 60) return `${seconds}s ago`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
  return `${Math.floor(seconds / 3600)}h ago`
}

// Auto-scroll to bottom when new activity arrives
watch(liveWhaleActivity, (newActivities, oldActivities) => {
  if (!newActivities || !oldActivities) return

  // Find new activities
  const newActivityIds = newActivities
    .slice(0, newActivities.length - oldActivities.length)
    .map(activity => activity.id)

  // Mark as recent for highlighting
  newActivityIds.forEach(id => {
    recentActivityIds.value.add(id)
    // Remove highlight after 3 seconds
    setTimeout(() => {
      recentActivityIds.value.delete(id)
    }, 3000)
  })

  // Auto-scroll if enabled
  if (autoScroll.value && feedContainer.value) {
    nextTick(() => {
      feedContainer.value?.scrollTo({
        top: 0,
        behavior: 'smooth'
      })
    })
  }
}, { deep: true })
</script>

<style scoped>
/* Activity list animations */
.activity-list-enter-active {
  transition: all 0.3s ease-out;
}

.activity-list-enter-from {
  transform: translateX(-20px);
  opacity: 0;
}

.activity-list-leave-active {
  transition: all 0.2s ease-in;
}

.activity-list-leave-to {
  transform: translateX(20px);
  opacity: 0;
}

.activity-list-move {
  transition: transform 0.3s ease;
}

/* Settings slide animation */
.settings-slide-enter-active,
.settings-slide-leave-active {
  transition: all 0.2s ease;
}

.settings-slide-enter-from,
.settings-slide-leave-to {
  transform: translateY(-8px);
  opacity: 0;
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

/* Activity item highlighting */
.activity-item {
  transition: all 0.3s ease;
}
</style>