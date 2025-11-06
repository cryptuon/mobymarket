<template>
  <Teleport to="body">
    <Transition
      name="modal-overlay"
      enter-active-class="transition-opacity duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
        @click="closeModal"
      >
        <Transition
          name="modal-content"
          enter-active-class="transition-all duration-300 ease-out"
          enter-from-class="transform scale-95 opacity-0"
          enter-to-class="transform scale-100 opacity-100"
          leave-active-class="transition-all duration-200 ease-in"
          leave-from-class="transform scale-100 opacity-100"
          leave-to-class="transform scale-95 opacity-0"
        >
          <Card
            v-if="isOpen"
            variant="glass"
            size="lg"
            class="w-full max-w-md mx-auto"
            @click.stop
          >
            <template #header>
              <div class="flex items-center justify-between w-full">
                <div>
                  <h2 class="text-xl font-bold text-white">Connect Wallet</h2>
                  <p class="text-sm text-white/60 mt-1">
                    Choose your preferred wallet to connect to Moby Market
                  </p>
                </div>
                <button
                  @click="closeModal"
                  class="p-2 hover:bg-white/10 rounded-lg transition-colors"
                  aria-label="Close modal"
                >
                  <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
                </button>
              </div>
            </template>

            <div class="space-y-3">
              <!-- Installed Wallets -->
              <div v-if="installedProviders.length > 0">
                <h3 class="text-sm font-medium text-white/80 mb-3">Available Wallets</h3>
                <div class="space-y-2">
                  <button
                    v-for="provider in installedProviders"
                    :key="provider.name"
                    @click="connectProvider(provider.name)"
                    :disabled="isConnecting"
                    class="w-full flex items-center justify-between p-4 bg-slate-800/50 hover:bg-slate-700/50 border border-slate-600/50 hover:border-slate-500/50 rounded-xl transition-all group disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <div class="flex items-center space-x-3">
                      <div class="w-8 h-8 rounded-lg bg-white/10 flex items-center justify-center">
                        <img
                          :src="provider.icon"
                          :alt="provider.name"
                          class="w-6 h-6"
                          @error="handleImageError"
                        />
                      </div>
                      <div class="text-left">
                        <div class="font-medium text-white">{{ provider.name }}</div>
                        <div class="text-xs text-green-400">Installed</div>
                      </div>
                    </div>

                    <div class="flex items-center">
                      <div
                        v-if="isConnecting && connectingProvider === provider.name"
                        class="animate-spin rounded-full h-4 w-4 border-2 border-white/20 border-t-white"
                      ></div>
                      <HeroIcon
                        v-else
                        name="ChevronRightIcon"
                        class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors"
                      />
                    </div>
                  </button>
                </div>
              </div>

              <!-- Not Installed Wallets -->
              <div v-if="notInstalledProviders.length > 0" class="mt-6">
                <h3 class="text-sm font-medium text-white/80 mb-3">Get a Wallet</h3>
                <div class="space-y-2">
                  <button
                    v-for="provider in notInstalledProviders"
                    :key="provider.name"
                    @click="openInstallLink(provider.name)"
                    class="w-full flex items-center justify-between p-4 bg-slate-800/30 hover:bg-slate-700/30 border border-slate-600/30 hover:border-slate-500/30 rounded-xl transition-all group"
                  >
                    <div class="flex items-center space-x-3">
                      <div class="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center">
                        <img
                          :src="provider.icon"
                          :alt="provider.name"
                          class="w-6 h-6 opacity-60"
                          @error="handleImageError"
                        />
                      </div>
                      <div class="text-left">
                        <div class="font-medium text-white/80">{{ provider.name }}</div>
                        <div class="text-xs text-white/50">Not installed</div>
                      </div>
                    </div>

                    <HeroIcon
                      name="ArrowTopRightOnSquareIcon"
                      class="w-4 h-4 text-white/40 group-hover:text-white/70 transition-colors"
                    />
                  </button>
                </div>
              </div>
            </div>

            <template #footer>
              <div class="text-center">
                <p class="text-xs text-white/50">
                  By connecting a wallet, you agree to Moby Market's
                  <a href="/terms" class="text-moby-400 hover:text-moby-300 underline">
                    Terms of Service
                  </a>
                  and acknowledge that you have read and understand the risks.
                </p>
              </div>
            </template>
          </Card>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

import Card from '@components/ui/Card.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

import { useWallet } from '@/composables/useWallet'
import { web3Provider } from '@/services/web3Provider'
import type { WalletProvider } from '@/types'

interface Props {
  isOpen: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  connected: [providerName: string]
}>()

const { connectWallet } = useWallet()

const availableProviders = ref<WalletProvider[]>([])
const isConnecting = ref(false)
const connectingProvider = ref<string>('')

// Computed properties
const installedProviders = computed(() =>
  availableProviders.value.filter(p => p.isInstalled)
)

const notInstalledProviders = computed(() =>
  availableProviders.value.filter(p => !p.isInstalled)
)

// Methods
async function connectProvider(providerName: string) {
  if (isConnecting.value) return

  isConnecting.value = true
  connectingProvider.value = providerName

  try {
    await connectWallet(providerName.toLowerCase())
    emit('connected', providerName)
    closeModal()
  } catch (error) {
    console.error('Failed to connect wallet:', error)
  } finally {
    isConnecting.value = false
    connectingProvider.value = ''
  }
}

function openInstallLink(providerName: string) {
  const installUrls: Record<string, string> = {
    'MetaMask': 'https://metamask.io/download/',
    'Coinbase Wallet': 'https://wallet.coinbase.com/',
    'Rainbow': 'https://rainbow.me/',
    'Trust Wallet': 'https://trustwallet.com/',
    'WalletConnect': 'https://walletconnect.com/'
  }

  const url = installUrls[providerName]
  if (url) {
    window.open(url, '_blank', 'noopener,noreferrer')
  }
}

function closeModal() {
  emit('close')
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  // Fallback to a default wallet icon
  img.src = '/icons/wallet-default.svg'
}

// Handle escape key
function handleEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.isOpen) {
    closeModal()
  }
}

// Lifecycle
onMounted(() => {
  availableProviders.value = web3Provider.getAvailableProviders()
  document.addEventListener('keydown', handleEscape)
})

// Cleanup
onUnmounted(() => {
  document.removeEventListener('keydown', handleEscape)
})
</script>

<style scoped>
/* Modal animations */
.modal-overlay-enter-active,
.modal-overlay-leave-active {
  transition: opacity 0.3s ease;
}

.modal-overlay-enter-from,
.modal-overlay-leave-to {
  opacity: 0;
}

.modal-content-enter-active {
  transition: all 0.3s ease-out;
}

.modal-content-leave-active {
  transition: all 0.2s ease-in;
}

.modal-content-enter-from,
.modal-content-leave-to {
  transform: scale(0.95);
  opacity: 0;
}
</style>