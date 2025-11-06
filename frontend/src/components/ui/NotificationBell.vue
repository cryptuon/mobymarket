<template>
  <div class="relative">
    <!-- Notification Bell Button -->
    <button
      @click="toggleNotifications"
      class="relative p-2 rounded-lg text-white/70 hover:text-white hover:bg-white/10 transition-all duration-200"
      :class="{ 'text-moby-400': hasUnread }"
      :aria-expanded="isOpen"
      aria-label="Notifications"
    >
      <HeroIcon :name="hasUnread ? 'BellAlertIcon' : 'BellIcon'" class="w-6 h-6" />

      <!-- Unread Badge -->
      <div
        v-if="unreadCount > 0"
        class="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white text-xs font-bold rounded-full flex items-center justify-center shadow-lg"
      >
        {{ unreadCount > 99 ? '99+' : unreadCount }}
      </div>

      <!-- Live Activity Indicator -->
      <div
        v-if="hasLiveActivity"
        class="absolute top-0 right-0 w-3 h-3 bg-green-400 rounded-full animate-pulse border-2 border-slate-900"
      ></div>
    </button>

    <!-- Notifications Dropdown -->
    <Transition
      name="notifications"
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div
        v-if="isOpen"
        class="absolute right-0 top-full mt-2 w-80 bg-slate-900/95 backdrop-blur-xl border border-white/20 rounded-xl shadow-xl z-50 max-h-96 overflow-hidden"
      >
        <!-- Header -->
        <div class="p-4 border-b border-white/10">
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-semibold text-white">Notifications</h3>
            <div class="flex items-center space-x-2">
              <button
                v-if="unreadCount > 0"
                @click="markAllAsRead"
                class="text-sm text-moby-400 hover:text-moby-300 transition-colors"
              >
                Mark all read
              </button>
              <button
                @click="openSettings"
                class="p-1 hover:bg-white/10 rounded-lg transition-colors"
                aria-label="Notification settings"
              >
                <HeroIcon name="Cog6ToothIcon" class="w-4 h-4 text-white/70" />
              </button>
            </div>
          </div>

          <!-- Filter Tabs -->
          <div class="flex space-x-1 mt-3">
            <button
              v-for="filter in filters"
              :key="filter.value"
              @click="activeFilter = filter.value"
              :class="[
                'px-3 py-1 text-xs font-medium rounded-lg transition-all',
                activeFilter === filter.value
                  ? 'bg-moby-500/20 text-moby-400 border border-moby-500/30'
                  : 'text-white/60 hover:text-white hover:bg-white/10'
              ]"
            >
              {{ filter.label }}
              <span v-if="filter.count > 0" class="ml-1 text-xs opacity-70">
                ({{ filter.count }})
              </span>
            </button>
          </div>
        </div>

        <!-- Notifications List -->
        <div class="max-h-80 overflow-y-auto">
          <div v-if="filteredNotifications.length === 0" class="p-6 text-center">
            <HeroIcon name="BellIcon" class="w-12 h-12 text-white/30 mx-auto mb-2" />
            <p class="text-white/60 text-sm">No notifications</p>
          </div>

          <div v-else class="divide-y divide-white/10">
            <NotificationItem
              v-for="notification in filteredNotifications"
              :key="notification.id"
              :notification="notification"
              @click="handleNotificationClick"
              @dismiss="dismissNotification"
            />
          </div>
        </div>

        <!-- Footer -->
        <div class="p-4 border-t border-white/10">
          <button
            @click="viewAllNotifications"
            class="w-full text-center text-sm text-moby-400 hover:text-moby-300 transition-colors"
          >
            View all notifications
          </button>
        </div>
      </div>
    </Transition>

    <!-- Click Outside Handler -->
    <div
      v-if="isOpen"
      class="fixed inset-0 z-40"
      @click="closeNotifications"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'

import HeroIcon from '@components/ui/HeroIcon.vue'
import NotificationItem from './NotificationItem.vue'

import { useNotificationStore } from '@stores/notifications'
import type { Notification } from '@/types'

const router = useRouter()
const notificationStore = useNotificationStore()

const {
  notifications,
  unreadCount,
  hasLiveActivity
} = storeToRefs(notificationStore)

const isOpen = ref(false)
const activeFilter = ref<string>('all')

// Filter options
const filters = computed(() => [
  {
    label: 'All',
    value: 'all',
    count: notifications.value.length
  },
  {
    label: 'Whale Activity',
    value: 'whale',
    count: notifications.value.filter(n => n.category === 'whale_activity').length
  },
  {
    label: 'Trading',
    value: 'trading',
    count: notifications.value.filter(n => n.category === 'trading').length
  },
  {
    label: 'Price Alerts',
    value: 'price',
    count: notifications.value.filter(n => n.category === 'price_alert').length
  },
  {
    label: 'System',
    value: 'system',
    count: notifications.value.filter(n => n.category === 'system').length
  }
])

// Computed properties
const hasUnread = computed(() => unreadCount.value > 0)

const filteredNotifications = computed(() => {
  if (activeFilter.value === 'all') {
    return notifications.value.slice(0, 20) // Limit to 20 most recent
  }
  return notifications.value
    .filter(n => n.category === activeFilter.value)
    .slice(0, 20)
})

// Methods
function toggleNotifications() {
  isOpen.value = !isOpen.value
}

function closeNotifications() {
  isOpen.value = false
}

function markAllAsRead() {
  notificationStore.markAllAsRead()
}

function dismissNotification(notification: Notification) {
  notificationStore.dismissNotification(notification.id)
}

function handleNotificationClick(notification: Notification) {
  notificationStore.markAsRead(notification.id)

  // Navigate based on notification type
  if (notification.actionUrl) {
    router.push(notification.actionUrl)
    closeNotifications()
  }
}

function openSettings() {
  router.push('/settings/notifications')
  closeNotifications()
}

function viewAllNotifications() {
  router.push('/notifications')
  closeNotifications()
}

// Handle escape key
function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    closeNotifications()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleEscape)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscape)
})
</script>

<style scoped>
/* Custom scrollbar for notifications list */
.max-h-80::-webkit-scrollbar {
  width: 4px;
}

.max-h-80::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.max-h-80::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 2px;
}

.max-h-80::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.5);
}
</style>