<template>
  <div class="relative">
    <!-- Connected State -->
    <div
      v-if="isConnected"
      class="flex items-center space-x-3 bg-glass-light backdrop-blur-md border border-white/20 rounded-xl px-4 py-2"
    >
      <!-- Wallet Avatar -->
      <div class="w-8 h-8 bg-gradient-to-r from-moby-400 to-moby-600 rounded-lg flex items-center justify-center">
        <span class="text-white text-xs font-bold">{{ walletInitials }}</span>
      </div>

      <!-- Wallet Info -->
      <div class="hidden sm:block">
        <div class="text-sm font-medium text-white">{{ formattedAddress }}</div>
        <div class="text-xs text-white/60">{{ formattedBalance }} ETH</div>
      </div>

      <!-- Network Badge -->
      <Badge :variant="networkBadgeVariant" size="sm">
        {{ networkName }}
      </Badge>

      <!-- Dropdown Toggle -->
      <button
        @click="toggleDropdown"
        class="p-1 hover:bg-white/10 rounded-lg transition-colors"
        :aria-expanded="isDropdownOpen"
        aria-label="Wallet menu"
      >
        <HeroIcon name="ChevronDownIcon" class="w-4 h-4 text-white/70" />
      </button>
    </div>

    <!-- Disconnected State -->
    <button
      v-else
      @click="showWalletModal = true"
      :disabled="isConnecting"
      class="flex items-center space-x-2 bg-gradient-to-r from-moby-500 to-moby-600 hover:from-moby-600 hover:to-moby-700 text-white px-6 py-2 rounded-xl font-medium transition-all duration-200 shadow-glow disabled:opacity-50"
    >
      <HeroIcon name="WalletIcon" class="w-5 h-5" />
      <span>{{ isConnecting ? 'Connecting...' : 'Connect Wallet' }}</span>
    </button>

    <!-- Wallet Dropdown Menu -->
    <Transition
      name="dropdown"
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div
        v-if="isConnected && isDropdownOpen"
        class="absolute right-0 top-full mt-2 w-64 bg-slate-900/95 backdrop-blur-xl border border-white/20 rounded-xl shadow-xl z-50"
      >
        <div class="p-4 border-b border-white/10">
          <div class="flex items-center space-x-3">
            <div class="w-10 h-10 bg-gradient-to-r from-moby-400 to-moby-600 rounded-lg flex items-center justify-center">
              <span class="text-white font-bold">{{ walletInitials }}</span>
            </div>
            <div>
              <div class="text-sm font-medium text-white">{{ formattedAddress }}</div>
              <div class="text-xs text-white/60">{{ networkName }}</div>
            </div>
          </div>
        </div>

        <div class="p-2">
          <button
            @click="copyAddress"
            class="w-full flex items-center space-x-3 px-3 py-2 text-sm text-white/80 hover:text-white hover:bg-white/10 rounded-lg transition-colors"
          >
            <HeroIcon name="ClipboardIcon" class="w-4 h-4" />
            <span>Copy Address</span>
          </button>

          <button
            @click="viewOnExplorer"
            class="w-full flex items-center space-x-3 px-3 py-2 text-sm text-white/80 hover:text-white hover:bg-white/10 rounded-lg transition-colors"
          >
            <HeroIcon name="ArrowTopRightOnSquareIcon" class="w-4 h-4" />
            <span>View on Explorer</span>
          </button>

          <div class="border-t border-white/10 my-2"></div>

          <button
            @click="switchNetwork"
            class="w-full flex items-center space-x-3 px-3 py-2 text-sm text-white/80 hover:text-white hover:bg-white/10 rounded-lg transition-colors"
          >
            <HeroIcon name="ArrowPathIcon" class="w-4 h-4" />
            <span>Switch Network</span>
          </button>

          <button
            @click="disconnectWallet"
            class="w-full flex items-center space-x-3 px-3 py-2 text-sm text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-lg transition-colors"
          >
            <HeroIcon name="ArrowRightOnRectangleIcon" class="w-4 h-4" />
            <span>Disconnect</span>
          </button>
        </div>
      </div>
    </Transition>

    <!-- Click Outside Handler -->
    <div
      v-if="isDropdownOpen"
      class="fixed inset-0 z-40"
      @click="closeDropdown"
    ></div>

    <!-- Wallet Selection Modal -->
    <WalletSelectionModal
      :is-open="showWalletModal"
      @close="showWalletModal = false"
      @connected="handleWalletConnected"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

import HeroIcon from '@components/ui/HeroIcon.vue'
import Badge from '@components/ui/Badge.vue'
import WalletSelectionModal from './WalletSelectionModal.vue'

import { useWallet } from '@/composables/useWallet'

const {
  isConnected,
  isConnecting,
  address,
  chainId,
  balance,
  formattedAddress,
  networkName,
  disconnectWallet
} = useWallet()

const isDropdownOpen = ref(false)
const showWalletModal = ref(false)

// Computed properties
const walletInitials = computed(() => {
  if (!address.value) return 'W'
  return address.value.slice(2, 4).toUpperCase()
})

const formattedBalance = computed(() => {
  if (!balance.value) return '0.00'
  return parseFloat(balance.value).toFixed(4)
})

const networkBadgeVariant = computed(() => {
  switch (chainId.value) {
    case 1: return 'primary'
    case 137: return 'info'
    case 42161: return 'success'
    case 10: return 'error'
    case 8453: return 'whale'
    default: return 'secondary'
  }
})

// Methods
function handleWalletConnected(providerName: string) {
  showWalletModal.value = false
  // Notification is handled by the useWallet composable
}

function toggleDropdown() {
  isDropdownOpen.value = !isDropdownOpen.value
}

function closeDropdown() {
  isDropdownOpen.value = false
}

async function copyAddress() {
  if (!address.value) return

  try {
    await navigator.clipboard.writeText(address.value)
    notificationStore.addNotification({
      type: 'success',
      title: 'Address Copied',
      message: 'Wallet address copied to clipboard'
    })
    closeDropdown()
  } catch (error) {
    notificationStore.addNotification({
      type: 'error',
      title: 'Copy Failed',
      message: 'Failed to copy address to clipboard'
    })
  }
}

function viewOnExplorer() {
  if (!address.value || !chainId.value) return

  const explorerUrls: Record<number, string> = {
    1: 'https://etherscan.io',
    137: 'https://polygonscan.com',
    42161: 'https://arbiscan.io',
    10: 'https://optimistic.etherscan.io',
    8453: 'https://basescan.org'
  }

  const explorerUrl = explorerUrls[chainId.value]
  if (explorerUrl) {
    window.open(`${explorerUrl}/address/${address.value}`, '_blank')
    closeDropdown()
  }
}

async function switchNetwork() {
  try {
    // This would typically open a network selection modal
    // For now, we'll just show a notification
    notificationStore.addNotification({
      type: 'info',
      title: 'Network Switching',
      message: 'Network switching coming soon'
    })
    closeDropdown()
  } catch (error) {
    notificationStore.addNotification({
      type: 'error',
      title: 'Network Switch Failed',
      message: 'Failed to switch network'
    })
  }
}

// Handle escape key
function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && isDropdownOpen.value) {
    closeDropdown()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleEscape)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscape)
})
</script>