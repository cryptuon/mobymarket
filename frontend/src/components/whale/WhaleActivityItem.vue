<template>
  <div
    :class="itemClass"
    class="p-4 rounded-xl border transition-all duration-200 cursor-pointer hover:scale-[1.02] group"
    @click="$emit('click', activity)"
  >
    <div class="flex items-center justify-between">
      <!-- Activity Info -->
      <div class="flex items-center space-x-3 flex-1 min-w-0">
        <!-- Activity Type Icon -->
        <div :class="typeIconBackgroundClass" class="w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0">
          <HeroIcon :name="typeIcon" class="w-5 h-5" :class="typeIconColorClass" />
        </div>

        <!-- Details -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center space-x-2 mb-1">
            <span class="text-white font-semibold">{{ activity.amount }} {{ activity.token }}</span>
            <span :class="typeTextClass" class="text-xs font-medium px-2 py-1 rounded-full">
              {{ activityTypeText }}
            </span>
          </div>

          <div class="flex items-center space-x-4 text-xs text-white/60">
            <span>{{ formatAddress(activity.address) }}</span>
            <span>{{ formatTimestamp(activity.timestamp) }}</span>
            <span v-if="activity.blockNumber">Block {{ activity.blockNumber.toLocaleString() }}</span>
          </div>
        </div>
      </div>

      <!-- Value -->
      <div class="text-right flex-shrink-0">
        <div class="text-lg font-bold text-white">${{ formatValue(activity.usdValue) }}</div>
        <div class="text-xs text-white/60">USD Value</div>
      </div>
    </div>

    <!-- Additional Info (expandable) -->
    <div v-if="showDetails" class="mt-4 pt-4 border-t border-white/10 space-y-2">
      <div class="grid grid-cols-2 gap-4 text-xs">
        <div>
          <span class="text-white/60">Transaction Hash:</span>
          <div class="text-white font-mono break-all">{{ activity.txHash }}</div>
        </div>
        <div v-if="activity.dex">
          <span class="text-white/60">DEX:</span>
          <div class="text-white">{{ activity.dex }}</div>
        </div>
        <div v-if="activity.priceImpact">
          <span class="text-white/60">Price Impact:</span>
          <div :class="[
            'font-medium',
            activity.priceImpact > 5 ? 'text-red-400' :
            activity.priceImpact > 2 ? 'text-yellow-400' : 'text-green-400'
          ]">
            {{ activity.priceImpact.toFixed(2) }}%
          </div>
        </div>
        <div v-if="activity.strategy">
          <span class="text-white/60">Strategy:</span>
          <div class="text-white">{{ activity.strategy }}</div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="flex items-center space-x-2 pt-2">
        <Button
          @click.stop="copyTxHash"
          variant="outline"
          size="sm"
          icon-left="ClipboardIcon"
        >
          Copy Hash
        </Button>
        <Button
          @click.stop="viewOnExplorer"
          variant="outline"
          size="sm"
          icon-left="ArrowTopRightOnSquareIcon"
        >
          View
        </Button>
      </div>
    </div>

    <!-- Expand/Collapse Button -->
    <button
      @click.stop="showDetails = !showDetails"
      class="w-full flex items-center justify-center mt-3 py-2 text-xs text-white/40 hover:text-white/70 transition-colors group"
    >
      <span>{{ showDetails ? 'Less' : 'More' }} details</span>
      <HeroIcon
        name="ChevronDownIcon"
        class="w-4 h-4 ml-1 transition-transform"
        :class="{ 'rotate-180': showDetails }"
      />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useNotificationStore } from '@/stores/notifications'
import type { WhaleActivity } from '@/types'

interface Props {
  activity: WhaleActivity
  highlight?: boolean
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  highlight: false,
  compact: false
})

const emit = defineEmits<{
  click: [activity: WhaleActivity]
}>()

const notificationStore = useNotificationStore()
const showDetails = ref(false)

// Computed properties
const itemClass = computed(() => [
  'bg-slate-800/30 hover:bg-slate-700/30 border-slate-600/30 hover:border-slate-500/50',
  {
    'ring-2 ring-moby-500/50 border-moby-500/30': props.highlight,
    'animate-pulse-once': props.highlight
  }
])

const typeIcon = computed(() => {
  switch (props.activity.type) {
    case 'buy': return 'ArrowTrendingUpIcon'
    case 'sell': return 'ArrowTrendingDownIcon'
    case 'transfer': return 'ArrowsRightLeftIcon'
    default: return 'CurrencyDollarIcon'
  }
})

const typeIconBackgroundClass = computed(() => {
  switch (props.activity.type) {
    case 'buy': return 'bg-green-500/20'
    case 'sell': return 'bg-red-500/20'
    case 'transfer': return 'bg-blue-500/20'
    default: return 'bg-white/10'
  }
})

const typeIconColorClass = computed(() => {
  switch (props.activity.type) {
    case 'buy': return 'text-green-400'
    case 'sell': return 'text-red-400'
    case 'transfer': return 'text-blue-400'
    default: return 'text-white/70'
  }
})

const typeTextClass = computed(() => {
  switch (props.activity.type) {
    case 'buy': return 'bg-green-500/20 text-green-400'
    case 'sell': return 'bg-red-500/20 text-red-400'
    case 'transfer': return 'bg-blue-500/20 text-blue-400'
    default: return 'bg-white/10 text-white/70'
  }
})

const activityTypeText = computed(() => {
  switch (props.activity.type) {
    case 'buy': return 'BUY'
    case 'sell': return 'SELL'
    case 'transfer': return 'TRANSFER'
    default: return 'ACTIVITY'
  }
})

// Methods
function formatAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`
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

async function copyTxHash() {
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
  const explorerUrls: Record<string, string> = {
    ethereum: 'https://etherscan.io',
    polygon: 'https://polygonscan.com',
    arbitrum: 'https://arbiscan.io',
    optimism: 'https://optimistic.etherscan.io',
    base: 'https://basescan.org'
  }

  // Default to Ethereum if chain not specified
  const explorerUrl = explorerUrls[props.activity.chain?.toLowerCase() || 'ethereum']
  const url = `${explorerUrl}/tx/${props.activity.txHash}`

  window.open(url, '_blank', 'noopener,noreferrer')
}
</script>

<style scoped>
/* Highlight animation */
@keyframes pulse-once {
  0% {
    transform: scale(1);
    box-shadow: 0 0 0 0 rgba(14, 165, 233, 0.7);
  }
  50% {
    transform: scale(1.02);
    box-shadow: 0 0 0 10px rgba(14, 165, 233, 0);
  }
  100% {
    transform: scale(1);
    box-shadow: 0 0 0 0 rgba(14, 165, 233, 0);
  }
}

.animate-pulse-once {
  animation: pulse-once 0.6s ease-out;
}

/* Hover scale effect */
.hover\:scale-\[1\.02\]:hover {
  transform: scale(1.02);
}
</style>