import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Notification, AppState } from '@/types'

export const useAppStore = defineStore('app', () => {
  // State
  const isGlobalLoading = ref(false)
  const loadingMessage = ref<string>()
  const notifications = ref<Notification[]>([])
  const sidebarOpen = ref(false)
  const sidebarPinned = ref(true)

  // Modal state
  const modalOpen = ref(false)
  const modalComponent = ref<any>(null)
  const modalProps = ref<Record<string, any>>({})

  // Computed
  const activeNotifications = computed(() =>
    notifications.value.filter(n => !n.duration ||
      Date.now() - n.timestamp < n.duration
    )
  )

  const unreadNotificationsCount = computed(() =>
    activeNotifications.value.length
  )

  // Actions
  function setGlobalLoading(loading: boolean, message?: string) {
    isGlobalLoading.value = loading
    loadingMessage.value = message
  }

  function addNotification(notification: Omit<Notification, 'id' | 'timestamp'>) {
    const id = crypto.randomUUID()
    const timestamp = Date.now()

    const newNotification: Notification = {
      id,
      timestamp,
      duration: notification.duration || 5000, // 5 seconds default
      ...notification,
    }

    notifications.value.unshift(newNotification)

    // Auto-remove after duration
    if (newNotification.duration) {
      setTimeout(() => {
        removeNotification(id)
      }, newNotification.duration)
    }

    return id
  }

  function removeNotification(id: string) {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index > -1) {
      notifications.value.splice(index, 1)
    }
  }

  function clearAllNotifications() {
    notifications.value = []
  }

  function toggleSidebar() {
    sidebarOpen.value = !sidebarOpen.value
  }

  function setSidebarOpen(open: boolean) {
    sidebarOpen.value = open
  }

  function setSidebarPinned(pinned: boolean) {
    sidebarPinned.value = pinned
    if (pinned) {
      sidebarOpen.value = true
    }
  }

  function openModal(component: any, props: Record<string, any> = {}) {
    modalComponent.value = component
    modalProps.value = props
    modalOpen.value = true
  }

  function closeModal() {
    modalOpen.value = false
    modalComponent.value = null
    modalProps.value = {}
  }

  // Notification helpers
  function notifySuccess(title: string, message: string, duration?: number) {
    return addNotification({
      type: 'success',
      title,
      message,
      duration,
    })
  }

  function notifyError(title: string, message: string, duration?: number) {
    return addNotification({
      type: 'error',
      title,
      message,
      duration: duration || 10000, // Errors stay longer
    })
  }

  function notifyWarning(title: string, message: string, duration?: number) {
    return addNotification({
      type: 'warning',
      title,
      message,
      duration,
    })
  }

  function notifyInfo(title: string, message: string, duration?: number) {
    return addNotification({
      type: 'info',
      title,
      message,
      duration,
    })
  }

  // Whale-specific notifications
  function notifyWhaleActivity(activity: {
    amount: string
    token: string
    type: string
  }) {
    return addNotification({
      type: 'info',
      title: '🐋 Whale Activity Detected',
      message: `${activity.amount} ${activity.token} ${activity.type}`,
      duration: 8000,
    })
  }

  function notifyTradeComplete(trade: {
    amount: string
    token: string
    profit?: string
  }) {
    return addNotification({
      type: 'success',
      title: '✅ Trade Completed',
      message: `${trade.amount} ${trade.token} executed${trade.profit ? ` • Profit: ${trade.profit}` : ''}`,
      duration: 6000,
    })
  }

  function notifyArbitrageOpportunity(opportunity: {
    pair: string
    profit: string
    timeLeft: number
  }) {
    return addNotification({
      type: 'warning',
      title: '⚡ Arbitrage Opportunity',
      message: `${opportunity.profit}% profit on ${opportunity.pair} • ${opportunity.timeLeft}min left`,
      duration: 15000, // Stay visible longer for time-sensitive opportunities
    })
  }

  return {
    // State
    isGlobalLoading,
    loadingMessage,
    notifications,
    sidebarOpen,
    sidebarPinned,
    modalOpen,
    modalComponent,
    modalProps,

    // Computed
    activeNotifications,
    unreadNotificationsCount,

    // Actions
    setGlobalLoading,
    addNotification,
    removeNotification,
    clearAllNotifications,
    toggleSidebar,
    setSidebarOpen,
    setSidebarPinned,
    openModal,
    closeModal,

    // Notification helpers
    notifySuccess,
    notifyError,
    notifyWarning,
    notifyInfo,
    notifyWhaleActivity,
    notifyTradeComplete,
    notifyArbitrageOpportunity,
  }
})