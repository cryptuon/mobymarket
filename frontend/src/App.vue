<template>
  <div
    id="app"
    class="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900"
    :class="{ 'dark': isDark }"
  >
    <!-- Background Effects -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none">
      <div class="absolute -top-40 -right-40 w-80 h-80 bg-moby-500/20 rounded-full blur-3xl"></div>
      <div class="absolute -bottom-40 -left-40 w-80 h-80 bg-purple-500/20 rounded-full blur-3xl"></div>
      <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-cyan-500/10 rounded-full blur-3xl"></div>
    </div>

    <!-- Navigation -->
    <AppNavigation />

    <!-- Main Content -->
    <main class="relative z-10">
      <RouterView v-slot="{ Component, route }">
        <Transition
          :name="route.meta.transition || 'fade'"
          mode="out-in"
          appear
        >
          <component :is="Component" :key="route.path" />
        </Transition>
      </RouterView>
    </main>

    <!-- TODO: Add global components like notifications, loading overlay, etc. -->
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { storeToRefs } from 'pinia'

import AppNavigation from '@components/layout/AppNavigation.vue'

import { useThemeStore } from '@stores/theme'
import { useAppStore } from '@stores/app'
import { useWalletStore } from '@stores/wallet'
import { useMarketStore } from '@stores/market'
import { useNotificationStore } from '@stores/notifications'

// Stores
const themeStore = useThemeStore()
const appStore = useAppStore()
const walletStore = useWalletStore()
const marketStore = useMarketStore()
const notificationStore = useNotificationStore()

const { isDark } = storeToRefs(themeStore)
const { isGlobalLoading } = storeToRefs(appStore)

onMounted(() => {
  // Initialize theme
  themeStore.initializeTheme()

  // Auto-connect wallet if previously connected
  walletStore.autoConnect()

  // Initialize market data
  marketStore.initialize()

  // Initialize sample notifications
  notificationStore.initializeSampleNotifications()
})
</script>

<style>
/* Page transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-left-enter-active,
.slide-left-leave-active {
  transition: all 0.3s ease;
}

.slide-left-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.slide-left-leave-to {
  transform: translateX(-100%);
  opacity: 0;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from {
  transform: translateY(20px);
  opacity: 0;
}

.slide-up-leave-to {
  transform: translateY(-20px);
  opacity: 0;
}

/* Custom scrollbars */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb {
  background: rgba(14, 165, 233, 0.5);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(14, 165, 233, 0.7);
}

/* Global font loading */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&display=swap');
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@300;400;500;600;700&display=swap');
</style>