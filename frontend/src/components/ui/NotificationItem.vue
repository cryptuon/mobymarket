<template>
  <div
    @click="$emit('click', notification)"
    :class="[
      'p-4 cursor-pointer transition-all duration-200 hover:bg-white/5',
      !notification.isRead ? 'bg-moby-500/5 border-l-2 border-l-moby-500' : ''
    ]"
  >
    <div class="flex items-start space-x-3">
      <!-- Notification Icon -->
      <div :class="[
        'flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center',
        iconBackgroundClass
      ]">
        <HeroIcon :name="notificationIcon" class="w-5 h-5" :class="iconColorClass" />
      </div>

      <!-- Notification Content -->
      <div class="flex-1 min-w-0">
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <h4 class="text-sm font-medium text-white truncate">
              {{ notification.title }}
            </h4>
            <p class="text-sm text-white/70 mt-1 line-clamp-2">
              {{ notification.message }}
            </p>

            <!-- Additional Data -->
            <div v-if="notification.data" class="mt-2 space-y-1">
              <!-- Whale Activity Data -->
              <div v-if="notification.category === 'whale_activity'" class="flex items-center space-x-4 text-xs">
                <span class="text-white/60">Amount:</span>
                <span class="text-moby-400 font-medium">{{ formatAmount(notification.data.amount) }}</span>
                <span class="text-white/60">Token:</span>
                <span class="text-white">{{ notification.data.token }}</span>
              </div>

              <!-- Price Alert Data -->
              <div v-else-if="notification.category === 'price_alert'" class="flex items-center space-x-4 text-xs">
                <span class="text-white/60">Price:</span>
                <span :class="[
                  'font-medium',
                  notification.data.direction === 'up' ? 'text-green-400' : 'text-red-400'
                ]">
                  ${{ formatPrice(notification.data.price) }}
                </span>
                <span class="text-white/60">Change:</span>
                <span :class="[
                  'font-medium',
                  notification.data.direction === 'up' ? 'text-green-400' : 'text-red-400'
                ]">
                  {{ notification.data.direction === 'up' ? '+' : '' }}{{ notification.data.change }}%
                </span>
              </div>

              <!-- Trading Data -->
              <div v-else-if="notification.category === 'trading'" class="flex items-center space-x-4 text-xs">
                <span class="text-white/60">Status:</span>
                <Badge :variant="getStatusVariant(notification.data.status)" size="sm">
                  {{ notification.data.status }}
                </Badge>
                <span v-if="notification.data.txHash" class="text-white/60">
                  Tx: {{ shortenHash(notification.data.txHash) }}
                </span>
              </div>
            </div>

            <!-- Timestamp -->
            <div class="flex items-center justify-between mt-2">
              <time class="text-xs text-white/50">
                {{ formatTimestamp(notification.timestamp) }}
              </time>

              <!-- Action Buttons -->
              <div class="flex items-center space-x-2">
                <button
                  v-if="notification.actionUrl"
                  class="text-xs text-moby-400 hover:text-moby-300 transition-colors"
                >
                  View Details
                </button>
                <button
                  @click.stop="$emit('dismiss', notification)"
                  class="p-1 text-white/40 hover:text-white/70 hover:bg-white/10 rounded transition-all"
                  aria-label="Dismiss notification"
                >
                  <HeroIcon name="XMarkIcon" class="w-3 h-3" />
                </button>
              </div>
            </div>
          </div>

          <!-- Unread Indicator -->
          <div
            v-if="!notification.isRead"
            class="flex-shrink-0 w-2 h-2 bg-moby-400 rounded-full mt-1"
          ></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import HeroIcon from '@components/ui/HeroIcon.vue'
import Badge from '@components/ui/Badge.vue'

import type { Notification } from '@/types'

interface Props {
  notification: Notification
}

const props = defineProps<Props>()

defineEmits<{
  click: [notification: Notification]
  dismiss: [notification: Notification]
}>()

// Computed properties
const notificationIcon = computed(() => {
  switch (props.notification.category) {
    case 'whale_activity':
      return 'EyeIcon'
    case 'trading':
      return props.notification.type === 'success' ? 'CheckCircleIcon' :
             props.notification.type === 'error' ? 'XCircleIcon' : 'ArrowsRightLeftIcon'
    case 'price_alert':
      return props.notification.data?.direction === 'up' ? 'ArrowTrendingUpIcon' : 'ArrowTrendingDownIcon'
    case 'system':
      return 'InformationCircleIcon'
    default:
      return 'BellIcon'
  }
})

const iconColorClass = computed(() => {
  switch (props.notification.type) {
    case 'success':
      return 'text-green-400'
    case 'error':
      return 'text-red-400'
    case 'warning':
      return 'text-yellow-400'
    case 'info':
      return 'text-blue-400'
    default:
      return 'text-white'
  }
})

const iconBackgroundClass = computed(() => {
  switch (props.notification.type) {
    case 'success':
      return 'bg-green-500/20'
    case 'error':
      return 'bg-red-500/20'
    case 'warning':
      return 'bg-yellow-500/20'
    case 'info':
      return 'bg-blue-500/20'
    default:
      return 'bg-white/10'
  }
})

// Methods
function formatAmount(amount: string | number): string {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount
  if (num >= 1e9) return `${(num / 1e9).toFixed(2)}B`
  if (num >= 1e6) return `${(num / 1e6).toFixed(2)}M`
  if (num >= 1e3) return `${(num / 1e3).toFixed(2)}K`
  return num.toFixed(2)
}

function formatPrice(price: string | number): string {
  const num = typeof price === 'string' ? parseFloat(price) : price
  return num.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6
  })
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`

  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
  })
}

function shortenHash(hash: string): string {
  return `${hash.slice(0, 6)}...${hash.slice(-4)}`
}

function getStatusVariant(status: string): 'success' | 'error' | 'warning' | 'info' {
  switch (status.toLowerCase()) {
    case 'completed':
    case 'success':
      return 'success'
    case 'failed':
    case 'error':
      return 'error'
    case 'pending':
    case 'processing':
      return 'warning'
    default:
      return 'info'
  }
}
</script>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>