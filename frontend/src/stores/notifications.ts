import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { Notification, NotificationCategory, NotificationType } from '@/types'

export const useNotificationStore = defineStore('notifications', () => {
  // State
  const notifications = ref<Notification[]>([])
  const maxNotifications = ref(100) // Limit stored notifications
  const hasLiveActivity = ref(false)

  // Getters
  const unreadCount = computed(() =>
    notifications.value.filter(n => !n.isRead).length
  )

  const unreadNotifications = computed(() =>
    notifications.value.filter(n => !n.isRead)
  )

  const recentNotifications = computed(() =>
    notifications.value.slice(0, 10)
  )

  const whaleActivityNotifications = computed(() =>
    notifications.value.filter(n => n.category === 'whale_activity')
  )

  const tradingNotifications = computed(() =>
    notifications.value.filter(n => n.category === 'trading')
  )

  const priceAlertNotifications = computed(() =>
    notifications.value.filter(n => n.category === 'price_alert')
  )

  const systemNotifications = computed(() =>
    notifications.value.filter(n => n.category === 'system')
  )

  // Actions
  function addNotification(notification: Omit<Notification, 'id' | 'timestamp' | 'isRead'>): string {
    const id = generateId()
    const newNotification: Notification = {
      ...notification,
      id,
      timestamp: new Date().toISOString(),
      isRead: false,
    }

    // Add to beginning of array (most recent first)
    notifications.value.unshift(newNotification)

    // Limit stored notifications
    if (notifications.value.length > maxNotifications.value) {
      notifications.value = notifications.value.slice(0, maxNotifications.value)
    }

    // Set live activity indicator
    if (notification.category === 'whale_activity' || notification.category === 'trading') {
      hasLiveActivity.value = true
      // Clear live activity after 30 seconds
      setTimeout(() => {
        hasLiveActivity.value = false
      }, 30000)
    }

    return id
  }

  function markAsRead(id: string): void {
    const notification = notifications.value.find(n => n.id === id)
    if (notification) {
      notification.isRead = true
    }
  }

  function markAllAsRead(): void {
    notifications.value.forEach(n => {
      n.isRead = true
    })
  }

  function dismissNotification(id: string): void {
    const index = notifications.value.findIndex(n => n.id === id)
    if (index !== -1) {
      notifications.value.splice(index, 1)
    }
  }

  function clearAllNotifications(): void {
    notifications.value = []
  }

  function clearReadNotifications(): void {
    notifications.value = notifications.value.filter(n => !n.isRead)
  }

  // Notification helpers
  function notifyWhaleActivity(data: {
    amount: string
    token: string
    type: 'buy' | 'sell' | 'transfer'
    address?: string
    txHash?: string
  }): string {
    const actionText = {
      buy: 'purchased',
      sell: 'sold',
      transfer: 'transferred'
    }

    return addNotification({
      type: 'info',
      category: 'whale_activity',
      title: '🐋 Whale Activity Detected',
      message: `Large ${actionText[data.type]} of ${data.amount} ${data.token}`,
      data: {
        amount: data.amount,
        token: data.token,
        type: data.type,
        address: data.address,
        txHash: data.txHash
      },
      actionUrl: data.txHash ? `/whale-intelligence/activity?tx=${data.txHash}` : '/whale-intelligence'
    })
  }

  function notifyTrade(data: {
    status: 'pending' | 'completed' | 'failed'
    tokenIn: string
    tokenOut: string
    amountIn: string
    amountOut?: string
    txHash?: string
    error?: string
  }): string {
    const type: NotificationType = data.status === 'completed' ? 'success' :
                                  data.status === 'failed' ? 'error' : 'info'

    const titles = {
      pending: 'Trade Pending',
      completed: 'Trade Completed',
      failed: 'Trade Failed'
    }

    const messages = {
      pending: `Swapping ${data.amountIn} ${data.tokenIn} for ${data.tokenOut}`,
      completed: `Successfully swapped ${data.amountIn} ${data.tokenIn} for ${data.amountOut} ${data.tokenOut}`,
      failed: `Failed to swap ${data.amountIn} ${data.tokenIn}: ${data.error || 'Unknown error'}`
    }

    return addNotification({
      type,
      category: 'trading',
      title: titles[data.status],
      message: messages[data.status],
      data: {
        status: data.status,
        tokenIn: data.tokenIn,
        tokenOut: data.tokenOut,
        amountIn: data.amountIn,
        amountOut: data.amountOut,
        txHash: data.txHash,
        error: data.error
      },
      actionUrl: data.txHash ? `/portfolio/history?tx=${data.txHash}` : '/portfolio'
    })
  }

  function notifyPriceAlert(data: {
    token: string
    price: number
    targetPrice: number
    direction: 'up' | 'down'
    change: number
  }): string {
    const directionText = data.direction === 'up' ? 'above' : 'below'
    const emoji = data.direction === 'up' ? '📈' : '📉'

    return addNotification({
      type: data.direction === 'up' ? 'success' : 'warning',
      category: 'price_alert',
      title: `${emoji} Price Alert: ${data.token}`,
      message: `${data.token} is now ${directionText} your target of $${data.targetPrice.toLocaleString()}`,
      data: {
        token: data.token,
        price: data.price,
        targetPrice: data.targetPrice,
        direction: data.direction,
        change: data.change
      },
      actionUrl: `/analytics/market?token=${data.token}`
    })
  }

  function notifySystem(
    title: string,
    message: string,
    type: NotificationType = 'info',
    actionUrl?: string
  ): string {
    return addNotification({
      type,
      category: 'system',
      title,
      message,
      actionUrl
    })
  }

  // Utility functions
  function generateId(): string {
    return `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
  }

  function getNotificationsByCategory(category: NotificationCategory): Notification[] {
    return notifications.value.filter(n => n.category === category)
  }

  function getNotificationsByType(type: NotificationType): Notification[] {
    return notifications.value.filter(n => n.type === type)
  }

  // Initialize with some sample notifications for development
  function initializeSampleNotifications(): void {
    if (notifications.value.length > 0) return // Don't add if already have notifications

    // Add some sample notifications
    notifyWhaleActivity({
      amount: '1,500 ETH',
      token: 'ETH',
      type: 'buy',
      address: '0x123...abc',
      txHash: '0xdef456...'
    })

    notifyPriceAlert({
      token: 'ETH',
      price: 3250,
      targetPrice: 3200,
      direction: 'up',
      change: 2.5
    })

    notifyTrade({
      status: 'completed',
      tokenIn: 'USDC',
      tokenOut: 'ETH',
      amountIn: '10,000',
      amountOut: '3.1',
      txHash: '0x789abc...'
    })

    notifySystem(
      'Welcome to Moby Market',
      'Your whale trading platform is ready. Start by connecting your wallet.',
      'info',
      '/help/getting-started'
    )
  }

  return {
    // State
    notifications,
    maxNotifications,
    hasLiveActivity,

    // Getters
    unreadCount,
    unreadNotifications,
    recentNotifications,
    whaleActivityNotifications,
    tradingNotifications,
    priceAlertNotifications,
    systemNotifications,

    // Actions
    addNotification,
    markAsRead,
    markAllAsRead,
    dismissNotification,
    clearAllNotifications,
    clearReadNotifications,

    // Helpers
    notifyWhaleActivity,
    notifyTrade,
    notifyPriceAlert,
    notifySystem,
    getNotificationsByCategory,
    getNotificationsByType,
    initializeSampleNotifications,
  }
})