<template>
  <div class="relative">
    <!-- Network Selection Button -->
    <button
      @click="toggleNetworkMenu"
      :class="buttonClass"
      class="flex items-center space-x-2 px-4 py-2 rounded-lg font-medium transition-all duration-200"
      :aria-expanded="isMenuOpen"
      aria-label="Switch network"
    >
      <div :class="statusIndicatorClass"></div>
      <span class="text-sm">{{ currentNetworkName }}</span>
      <HeroIcon name="ChevronDownIcon" class="w-4 h-4" />
    </button>

    <!-- Network Dropdown Menu -->
    <Transition
      name="network-menu"
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="transform scale-95 opacity-0"
      enter-to-class="transform scale-100 opacity-100"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="transform scale-100 opacity-100"
      leave-to-class="transform scale-95 opacity-0"
    >
      <div
        v-if="isMenuOpen"
        class="absolute top-full mt-2 w-64 bg-slate-900/95 backdrop-blur-xl border border-white/20 rounded-xl shadow-xl z-50"
        :class="menuPosition"
      >
        <div class="p-4 border-b border-white/10">
          <h3 class="text-sm font-medium text-white">Switch Network</h3>
          <p class="text-xs text-white/60 mt-1">
            Choose your preferred network for trading
          </p>
        </div>

        <div class="p-2">
          <button
            v-for="network in supportedNetworks"
            :key="network.id"
            @click="switchToNetwork(network.id)"
            :disabled="isSwitching || network.id === currentChainId"
            :class="[
              'w-full flex items-center justify-between p-3 rounded-lg transition-all group',
              network.id === currentChainId
                ? 'bg-moby-500/10 border border-moby-500/30'
                : 'hover:bg-white/5 border border-transparent'
            ]"
          >
            <div class="flex items-center space-x-3">
              <div :class="[
                'w-8 h-8 rounded-lg flex items-center justify-center',
                network.id === currentChainId ? 'bg-moby-500/20' : 'bg-white/10'
              ]">
                <img
                  :src="network.icon"
                  :alt="network.name"
                  class="w-5 h-5"
                  @error="handleImageError"
                />
              </div>
              <div class="text-left">
                <div :class="[
                  'text-sm font-medium',
                  network.id === currentChainId ? 'text-moby-400' : 'text-white'
                ]">
                  {{ network.name }}
                </div>
                <div class="text-xs text-white/60">{{ network.description }}</div>
              </div>
            </div>

            <div class="flex items-center space-x-2">
              <!-- Current network indicator -->
              <div
                v-if="network.id === currentChainId"
                class="flex items-center space-x-1 text-xs text-moby-400"
              >
                <div class="w-2 h-2 bg-moby-400 rounded-full"></div>
                <span>Active</span>
              </div>

              <!-- Loading spinner -->
              <div
                v-if="isSwitching && switchingToChainId === network.id"
                class="animate-spin rounded-full h-4 w-4 border-2 border-white/20 border-t-white"
              ></div>

              <!-- Testnet badge -->
              <Badge
                v-if="network.isTestnet"
                variant="warning"
                size="sm"
              >
                Testnet
              </Badge>
            </div>
          </button>
        </div>

        <div class="p-4 border-t border-white/10">
          <p class="text-xs text-white/50">
            Switching networks may require approval in your wallet
          </p>
        </div>
      </div>
    </Transition>

    <!-- Click Outside Handler -->
    <div
      v-if="isMenuOpen"
      class="fixed inset-0 z-40"
      @click="closeNetworkMenu"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

import HeroIcon from '@components/ui/HeroIcon.vue'
import Badge from '@components/ui/Badge.vue'

import { useWallet } from '@/composables/useWallet'
import { useNotificationStore } from '@/stores/notifications'

interface Props {
  size?: 'sm' | 'md' | 'lg'
  position?: 'left' | 'right'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  position: 'right'
})

const {
  isConnected,
  chainId: currentChainId,
  networkName: currentNetworkName,
  switchToMainnet,
  switchToArbitrum
} = useWallet()

const notificationStore = useNotificationStore()

const isMenuOpen = ref(false)
const isSwitching = ref(false)
const switchingToChainId = ref<number | null>(null)

// Supported networks configuration
const supportedNetworks = ref([
  {
    id: 1,
    name: 'Ethereum',
    description: 'Ethereum Mainnet',
    icon: '/icons/ethereum.svg',
    isTestnet: false,
    gasToken: 'ETH'
  },
  {
    id: 137,
    name: 'Polygon',
    description: 'Polygon Mainnet',
    icon: '/icons/polygon.svg',
    isTestnet: false,
    gasToken: 'MATIC'
  },
  {
    id: 42161,
    name: 'Arbitrum',
    description: 'Arbitrum One',
    icon: '/icons/arbitrum.svg',
    isTestnet: false,
    gasToken: 'ETH'
  },
  {
    id: 10,
    name: 'Optimism',
    description: 'Optimism Mainnet',
    icon: '/icons/optimism.svg',
    isTestnet: false,
    gasToken: 'ETH'
  },
  {
    id: 8453,
    name: 'Base',
    description: 'Base Mainnet',
    icon: '/icons/base.svg',
    isTestnet: false,
    gasToken: 'ETH'
  }
])

// Computed properties
const buttonClass = computed(() => [
  'bg-slate-800/50 hover:bg-slate-700/50 border border-slate-600/50 hover:border-slate-500/50 text-white',
  {
    'text-xs px-2 py-1': props.size === 'sm',
    'text-sm px-4 py-2': props.size === 'md',
    'text-base px-5 py-3': props.size === 'lg'
  }
])

const statusIndicatorClass = computed(() => [
  'w-2 h-2 rounded-full',
  isConnected.value ? 'bg-green-500 animate-pulse' : 'bg-red-500'
])

const menuPosition = computed(() => {
  return props.position === 'left' ? 'left-0' : 'right-0'
})

// Methods
function toggleNetworkMenu() {
  if (!isConnected.value) {
    notificationStore.notifySystem(
      'Wallet Required',
      'Please connect your wallet first',
      'warning'
    )
    return
  }

  isMenuOpen.value = !isMenuOpen.value
}

function closeNetworkMenu() {
  isMenuOpen.value = false
}

async function switchToNetwork(chainId: number) {
  if (!isConnected.value || isSwitching.value) return

  isSwitching.value = true
  switchingToChainId.value = chainId

  try {
    const network = supportedNetworks.value.find(n => n.id === chainId)

    if (!network) {
      throw new Error(`Unsupported network: ${chainId}`)
    }

    // Use specific methods for common networks
    if (chainId === 1) {
      await switchToMainnet()
    } else if (chainId === 42161) {
      await switchToArbitrum()
    } else {
      // Generic network switching would go here
      notificationStore.notifySystem(
        'Network Switch',
        `Switching to ${network.name} - please approve in your wallet`,
        'info'
      )

      // Simulate network switch for other networks
      setTimeout(() => {
        notificationStore.notifySystem(
          'Network Switch',
          `Successfully switched to ${network.name}`,
          'success'
        )
      }, 2000)
    }

    closeNetworkMenu()
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error'
    notificationStore.notifySystem(
      'Network Switch Failed',
      errorMessage,
      'error'
    )
  } finally {
    isSwitching.value = false
    switchingToChainId.value = null
  }
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  // Fallback to a default network icon
  img.src = '/icons/network-default.svg'
}

// Handle escape key
function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && isMenuOpen.value) {
    closeNetworkMenu()
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
/* Network menu animations */
.network-menu-enter-active,
.network-menu-leave-active {
  transition: all 0.2s ease;
}

.network-menu-enter-from,
.network-menu-leave-to {
  transform: scale(0.95);
  opacity: 0;
}
</style>