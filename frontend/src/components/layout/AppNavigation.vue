<template>
  <nav class="fixed top-0 left-0 right-0 z-50 bg-glass-dark border-b border-white/10 backdrop-blur-xl">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center justify-between h-16">
        <!-- Logo & Brand -->
        <div class="flex items-center space-x-4">
          <RouterLink
            to="/"
            class="flex items-center space-x-3 hover:opacity-80 transition-opacity"
          >
            <div class="w-8 h-8 bg-gradient-to-br from-moby-400 to-moby-600 rounded-lg flex items-center justify-center">
              <span class="text-white font-bold text-lg">🐋</span>
            </div>
            <div class="hidden sm:block">
              <h1 class="text-xl font-bold text-gradient-primary">Moby Market</h1>
              <p class="text-xs text-white/60 -mt-1">Whale Trading Platform</p>
            </div>
          </RouterLink>
        </div>

        <!-- Desktop Navigation -->
        <div class="hidden lg:flex items-center space-x-1">
          <NavItem
            v-for="item in mainNavItems"
            :key="item.name"
            :to="item.to"
            :icon="item.icon"
            :badge="item.badge"
          >
            {{ item.name }}
          </NavItem>
        </div>

        <!-- Right Side Actions -->
        <div class="flex items-center space-x-4">
          <!-- Market Status -->
          <MarketStatusIndicator class="hidden md:block" />

          <!-- Notifications -->
          <NotificationBell />

          <!-- Wallet Connection -->
          <WalletConnector />

          <!-- Theme Toggle -->
          <ThemeToggle />

          <!-- Mobile Menu Button -->
          <button
            @click="toggleMobileMenu"
            class="lg:hidden p-2 rounded-lg text-white/70 hover:text-white hover:bg-white/10 transition-all"
            :aria-expanded="isMobileMenuOpen"
            aria-label="Toggle navigation menu"
          >
            <HeroIcon :name="isMobileMenuOpen ? 'XMarkIcon' : 'Bars3Icon'" class="w-6 h-6" />
          </button>
        </div>
      </div>
    </div>

    <!-- Mobile Navigation Menu -->
    <Transition
      name="mobile-menu"
      enter-active-class="transition-all duration-300"
      enter-from-class="transform -translate-y-full opacity-0"
      enter-to-class="transform translate-y-0 opacity-100"
      leave-active-class="transition-all duration-200"
      leave-from-class="transform translate-y-0 opacity-100"
      leave-to-class="transform -translate-y-full opacity-0"
    >
      <div
        v-if="isMobileMenuOpen"
        class="lg:hidden bg-slate-900/95 backdrop-blur-xl border-b border-white/10"
      >
        <div class="px-4 py-6 space-y-3">
          <MobileNavItem
            v-for="item in allNavItems"
            :key="item.name"
            :to="item.to"
            :icon="item.icon"
            :badge="item.badge"
            @click="closeMobileMenu"
          >
            {{ item.name }}
          </MobileNavItem>

          <!-- Mobile-only items -->
          <div class="pt-4 border-t border-white/10">
            <MobileNavItem to="/settings" icon="CogIcon">
              Settings
            </MobileNavItem>
            <MobileNavItem to="/help" icon="QuestionMarkCircleIcon">
              Help
            </MobileNavItem>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Mobile Menu Overlay -->
    <Transition
      name="overlay"
      enter-active-class="transition-opacity duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isMobileMenuOpen"
        class="fixed inset-0 bg-black/50 backdrop-blur-sm lg:hidden"
        @click="closeMobileMenu"
      />
    </Transition>
  </nav>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'

import NavItem from './NavItem.vue'
import MobileNavItem from './MobileNavItem.vue'
import MarketStatusIndicator from '@components/ui/MarketStatusIndicator.vue'
import NotificationBell from '@components/ui/NotificationBell.vue'
import WalletConnector from '@components/wallet/WalletConnector.vue'
import ThemeToggle from '@components/ui/ThemeToggle.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useAppStore } from '@stores/app'

// Stores
const appStore = useAppStore()
const { unreadNotificationsCount } = storeToRefs(appStore)

// Mobile menu state
const isMobileMenuOpen = ref(false)

// Navigation items
const mainNavItems = [
  {
    name: 'Trade',
    to: '/trade',
    icon: 'ArrowsRightLeftIcon',
  },
  {
    name: 'Portfolio',
    to: '/portfolio',
    icon: 'ChartPieIcon',
  },
  {
    name: 'Whale Intel',
    to: '/whale-intelligence',
    icon: 'EyeIcon',
    badge: computed(() => unreadNotificationsCount.value > 0 ? 'new' : undefined),
  },
  {
    name: 'Yield',
    to: '/yield',
    icon: 'CurrencyDollarIcon',
  },
  {
    name: 'Analytics',
    to: '/analytics',
    icon: 'ChartBarIcon',
  },
]

const secondaryNavItems = [
  {
    name: 'Governance',
    to: '/governance',
    icon: 'BuildingLibraryIcon',
  },
]

const allNavItems = [...mainNavItems, ...secondaryNavItems]

// Mobile menu actions
function toggleMobileMenu() {
  isMobileMenuOpen.value = !isMobileMenuOpen.value
}

function closeMobileMenu() {
  isMobileMenuOpen.value = false
}

// Close mobile menu on route change
const route = useRoute()
watch(() => route.path, () => {
  closeMobileMenu()
})

// Close mobile menu on escape key
function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && isMobileMenuOpen.value) {
    closeMobileMenu()
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
/* Navigation styling */
.router-link-active {
  @apply text-moby-400;
}

.router-link-exact-active {
  @apply text-moby-300;
}

/* Mobile menu animations */
.mobile-menu-enter-active,
.mobile-menu-leave-active {
  transition: all 0.3s ease;
}

.mobile-menu-enter-from,
.mobile-menu-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}

/* Overlay animations */
.overlay-enter-active,
.overlay-leave-active {
  transition: opacity 0.3s ease;
}

.overlay-enter-from,
.overlay-leave-to {
  opacity: 0;
}
</style>