<template>
  <div>
    <!-- Token Selection Button -->
    <button
      @click="openTokenSheet"
      :class="buttonClass"
      class="flex items-center justify-between w-full p-4 rounded-xl transition-all duration-200 active:scale-98"
      :disabled="disabled"
    >
      <div class="flex items-center space-x-3">
        <!-- Token Icon -->
        <div v-if="selectedToken" class="w-10 h-10 rounded-full overflow-hidden bg-white/10 flex-shrink-0">
          <img
            :src="selectedToken.logoURI"
            :alt="selectedToken.symbol"
            class="w-full h-full object-cover"
            @error="handleImageError"
          />
        </div>
        <div v-else class="w-10 h-10 rounded-full bg-white/10 flex items-center justify-center flex-shrink-0">
          <HeroIcon name="QuestionMarkCircleIcon" class="w-6 h-6 text-white/50" />
        </div>

        <!-- Token Info -->
        <div class="text-left flex-1 min-w-0">
          <div class="font-semibold text-white text-lg">
            {{ selectedToken?.symbol || 'Select Token' }}
          </div>
          <div v-if="selectedToken" class="text-sm text-white/60 truncate">
            {{ selectedToken.name }}
          </div>
        </div>
      </div>

      <HeroIcon
        name="ChevronDownIcon"
        class="w-5 h-5 text-white/60 flex-shrink-0"
      />
    </button>

    <!-- Mobile Bottom Sheet -->
    <Teleport to="body">
      <Transition
        name="sheet-overlay"
        enter-active-class="transition-opacity duration-300"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition-opacity duration-200"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
      >
        <div
          v-if="isOpen"
          class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50"
          @click="closeTokenSheet"
        >
          <Transition
            name="sheet-content"
            enter-active-class="transition-transform duration-300 ease-out"
            enter-from-class="transform translate-y-full"
            enter-to-class="transform translate-y-0"
            leave-active-class="transition-transform duration-200 ease-in"
            leave-from-class="transform translate-y-0"
            leave-to-class="transform translate-y-full"
          >
            <div
              v-if="isOpen"
              class="absolute bottom-0 left-0 right-0 bg-slate-900 rounded-t-3xl max-h-[90vh] overflow-hidden"
              @click.stop
            >
              <!-- Sheet Header -->
              <div class="sticky top-0 bg-slate-900 border-b border-white/10 z-10">
                <!-- Handle -->
                <div class="flex justify-center pt-3 pb-2">
                  <div class="w-12 h-1 bg-white/20 rounded-full"></div>
                </div>

                <!-- Header Content -->
                <div class="px-4 pb-4">
                  <div class="flex items-center justify-between mb-4">
                    <h3 class="text-xl font-bold text-white">Select Token</h3>
                    <button
                      @click="closeTokenSheet"
                      class="p-2 hover:bg-white/10 rounded-xl transition-colors"
                      aria-label="Close"
                    >
                      <HeroIcon name="XMarkIcon" class="w-6 h-6 text-white/70" />
                    </button>
                  </div>

                  <!-- Search Input -->
                  <div class="relative">
                    <HeroIcon
                      name="MagnifyingGlassIcon"
                      class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-white/40"
                    />
                    <input
                      v-model="searchQuery"
                      type="text"
                      placeholder="Search tokens..."
                      class="w-full bg-slate-800/50 border border-slate-600/50 rounded-xl pl-10 pr-4 py-3 text-white placeholder-white/40 focus:outline-none focus:border-moby-500/50 focus:ring-2 focus:ring-moby-500/20"
                      @input="handleSearch"
                    />
                    <button
                      v-if="searchQuery"
                      @click="clearSearch"
                      class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 hover:bg-white/10 rounded-lg transition-colors"
                    >
                      <HeroIcon name="XMarkIcon" class="w-4 h-4 text-white/40" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- Sheet Content -->
              <div class="overflow-y-auto px-4 pb-safe" style="max-height: calc(90vh - 140px);">
                <!-- Popular Tokens -->
                <div v-if="!searchQuery" class="mb-6">
                  <h4 class="text-sm font-semibold text-white/80 mb-3 px-1">Popular</h4>
                  <div class="grid grid-cols-2 gap-3">
                    <button
                      v-for="token in popularTokens"
                      :key="token.address"
                      @click="selectToken(token)"
                      class="flex items-center space-x-3 p-3 bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30 hover:border-slate-500/50 rounded-xl transition-all active:scale-95"
                    >
                      <img
                        :src="token.logoURI"
                        :alt="token.symbol"
                        class="w-8 h-8 rounded-full flex-shrink-0"
                        @error="handleImageError"
                      />
                      <div class="text-left flex-1 min-w-0">
                        <div class="text-white font-medium truncate">{{ token.symbol }}</div>
                        <div class="text-xs text-white/60 truncate">{{ token.name }}</div>
                      </div>
                    </button>
                  </div>
                </div>

                <!-- Token List -->
                <div class="space-y-2">
                  <h4 v-if="!searchQuery" class="text-sm font-semibold text-white/80 mb-3 px-1">All Tokens</h4>

                  <!-- No Results -->
                  <div v-if="filteredTokens.length === 0" class="text-center py-12">
                    <HeroIcon name="MagnifyingGlassIcon" class="w-16 h-16 text-white/20 mx-auto mb-4" />
                    <p class="text-white/60 text-lg mb-2">No tokens found</p>
                    <p class="text-white/40 text-sm">Try adjusting your search or import a custom token</p>
                  </div>

                  <!-- Token Items -->
                  <button
                    v-for="token in filteredTokens"
                    :key="token.address"
                    @click="selectToken(token)"
                    :disabled="token.address === excludeToken"
                    class="w-full flex items-center justify-between p-4 hover:bg-slate-800/30 rounded-xl transition-all active:scale-98 disabled:opacity-50 disabled:cursor-not-allowed group"
                  >
                    <div class="flex items-center space-x-4">
                      <img
                        :src="token.logoURI"
                        :alt="token.symbol"
                        class="w-12 h-12 rounded-full flex-shrink-0"
                        @error="handleImageError"
                      />
                      <div class="text-left flex-1 min-w-0">
                        <div class="text-white font-semibold text-lg">{{ token.symbol }}</div>
                        <div class="text-white/60 text-sm truncate">{{ token.name }}</div>
                      </div>
                    </div>

                    <div v-if="getTokenBalance(token)" class="text-right flex-shrink-0">
                      <div class="text-white font-medium">{{ formatBalance(getTokenBalance(token)) }}</div>
                      <div class="text-xs text-white/50">Balance</div>
                    </div>
                  </button>
                </div>

                <!-- Import Token Section -->
                <div class="mt-8 pt-6 border-t border-white/10">
                  <button
                    @click="showImportToken"
                    class="w-full flex items-center justify-center space-x-2 p-4 border-2 border-dashed border-white/20 hover:border-moby-500/50 rounded-xl transition-all group"
                  >
                    <HeroIcon name="PlusIcon" class="w-5 h-5 text-white/60 group-hover:text-moby-400" />
                    <span class="text-white/60 group-hover:text-moby-400 font-medium">Import Custom Token</span>
                  </button>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import HeroIcon from '@components/ui/HeroIcon.vue'

import { useTradingStore } from '@/stores/trading'
import type { TokenInfo } from '@/types'

interface Props {
  modelValue?: string
  excludeToken?: string
  disabled?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'token-selected': [token: TokenInfo]
}>()

const tradingStore = useTradingStore()
const { popularTokens } = tradingStore

const isOpen = ref(false)
const searchQuery = ref('')

// Mock token balances
const tokenBalances = ref<Record<string, string>>({
  '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2': '2.5432',
  '0xA0b86a33E6847d7b1e7bCd15F6FdE5d2b9FC1234': '1250.00',
  '0xdAC17F958D2ee523a2206206994597C13D831ec7': '500.00',
  '0x6B175474E89094C44Da98b954EedeAC495271d0F': '750.25',
})

// Extended token list
const allTokens = ref<TokenInfo[]>([
  ...popularTokens,
  {
    address: '0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984',
    symbol: 'UNI',
    name: 'Uniswap',
    decimals: 18,
    chainId: 1,
    logoURI: '/tokens/uni.svg'
  },
  {
    address: '0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9',
    symbol: 'AAVE',
    name: 'Aave',
    decimals: 18,
    chainId: 1,
    logoURI: '/tokens/aave.svg'
  }
])

// Computed properties
const selectedToken = computed(() => {
  if (!props.modelValue) return null
  return allTokens.value.find(token => token.address === props.modelValue)
})

const buttonClass = computed(() => [
  'bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30 hover:border-slate-500/50',
  {
    'opacity-50 cursor-not-allowed': props.disabled
  }
])

const filteredTokens = computed(() => {
  if (!searchQuery.value) return allTokens.value

  const query = searchQuery.value.toLowerCase()
  return allTokens.value.filter(token =>
    token.symbol.toLowerCase().includes(query) ||
    token.name.toLowerCase().includes(query) ||
    token.address.toLowerCase().includes(query)
  )
})

// Methods
function openTokenSheet() {
  if (props.disabled) return
  isOpen.value = true
  // Prevent body scroll on mobile
  document.body.style.overflow = 'hidden'
}

function closeTokenSheet() {
  isOpen.value = false
  searchQuery.value = ''
  // Restore body scroll
  document.body.style.overflow = ''
}

function selectToken(token: TokenInfo) {
  emit('update:modelValue', token.address)
  emit('token-selected', token)
  closeTokenSheet()
}

function handleSearch(event: Event) {
  const target = event.target as HTMLInputElement
  searchQuery.value = target.value
}

function clearSearch() {
  searchQuery.value = ''
}

function getTokenBalance(token: TokenInfo): string | null {
  return tokenBalances.value[token.address] || null
}

function formatBalance(balance: string | null): string {
  if (!balance) return '0'
  const num = parseFloat(balance)
  if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(2)}K`
  if (num >= 1) return num.toFixed(4)
  return num.toFixed(6)
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  img.src = '/tokens/default.svg'
}

function showImportToken() {
  // Would open import token modal
  console.log('Import custom token')
}
</script>

<style scoped>
/* Active scale effect */
.active\:scale-95:active {
  transform: scale(0.95);
}

.active\:scale-98:active {
  transform: scale(0.98);
}

/* Safe area support */
.pb-safe {
  padding-bottom: env(safe-area-inset-bottom);
}

/* Sheet animations */
.sheet-overlay-enter-active,
.sheet-overlay-leave-active {
  transition: opacity 0.3s ease;
}

.sheet-overlay-enter-from,
.sheet-overlay-leave-to {
  opacity: 0;
}

.sheet-content-enter-active {
  transition: transform 0.3s ease-out;
}

.sheet-content-leave-active {
  transition: transform 0.2s ease-in;
}

.sheet-content-enter-from,
.sheet-content-leave-to {
  transform: translateY(100%);
}
</style>