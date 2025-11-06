<template>
  <!-- Mobile Bottom Sheet -->
  <Teleport to="body">
    <Transition
      name="settings-overlay"
      enter-active-class="transition-opacity duration-300"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-200"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50"
        @click="$emit('close')"
      >
        <Transition
          name="settings-content"
          enter-active-class="transition-transform duration-300 ease-out"
          enter-from-class="transform translate-y-full"
          enter-to-class="transform translate-y-0"
          leave-active-class="transition-transform duration-200 ease-in"
          leave-from-class="transform translate-y-0"
          leave-to-class="transform translate-y-full"
        >
          <div
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
                <div class="flex items-center justify-between">
                  <h3 class="text-xl font-bold text-white">Swap Settings</h3>
                  <button
                    @click="$emit('close')"
                    class="p-2 hover:bg-white/10 rounded-xl transition-colors"
                    aria-label="Close"
                  >
                    <HeroIcon name="XMarkIcon" class="w-6 h-6 text-white/70" />
                  </button>
                </div>
              </div>
            </div>

            <!-- Sheet Content -->
            <div class="overflow-y-auto px-4 pb-safe" style="max-height: calc(90vh - 100px);">
              <div class="space-y-8 py-4">
                <!-- Slippage Tolerance -->
                <div>
                  <h4 class="text-lg font-semibold text-white mb-4">Slippage Tolerance</h4>

                  <!-- Preset Buttons -->
                  <div class="grid grid-cols-4 gap-3 mb-4">
                    <button
                      v-for="preset in slippagePresets"
                      :key="preset"
                      @click="setSlippage(preset)"
                      :class="[
                        'py-3 px-4 rounded-xl text-sm font-semibold transition-all active:scale-95',
                        localSlippage === preset
                          ? 'bg-moby-500/20 text-moby-400 border-2 border-moby-500/50'
                          : 'bg-slate-800/50 text-white border-2 border-slate-600/30 hover:border-slate-500/50'
                      ]"
                    >
                      {{ preset }}%
                    </button>
                  </div>

                  <!-- Custom Input -->
                  <div class="relative">
                    <input
                      v-model="customSlippage"
                      type="number"
                      step="0.1"
                      min="0.1"
                      max="50"
                      placeholder="Custom %"
                      class="w-full bg-slate-800/50 border-2 border-slate-600/30 focus:border-moby-500/50 rounded-xl px-4 py-3 text-white placeholder-white/40 focus:outline-none focus:ring-2 focus:ring-moby-500/20"
                      @input="handleCustomSlippage"
                    />
                    <span class="absolute right-4 top-1/2 transform -translate-y-1/2 text-white/60 text-sm font-medium">
                      %
                    </span>
                  </div>

                  <!-- Warning Messages -->
                  <div class="mt-3">
                    <div v-if="localSlippage < 0.1" class="flex items-center space-x-2 text-red-400 text-sm bg-red-500/10 p-3 rounded-xl">
                      <HeroIcon name="ExclamationTriangleIcon" class="w-5 h-5 flex-shrink-0" />
                      <span>Your transaction may fail with very low slippage</span>
                    </div>
                    <div v-else-if="localSlippage > 5" class="flex items-center space-x-2 text-yellow-400 text-sm bg-yellow-500/10 p-3 rounded-xl">
                      <HeroIcon name="ExclamationTriangleIcon" class="w-5 h-5 flex-shrink-0" />
                      <span>High slippage may result in unfavorable trades</span>
                    </div>
                    <div v-else class="flex items-center space-x-2 text-green-400 text-sm bg-green-500/10 p-3 rounded-xl">
                      <HeroIcon name="CheckCircleIcon" class="w-5 h-5 flex-shrink-0" />
                      <span>Recommended slippage range</span>
                    </div>
                  </div>
                </div>

                <!-- Transaction Deadline -->
                <div>
                  <h4 class="text-lg font-semibold text-white mb-4">Transaction Deadline</h4>
                  <div class="relative">
                    <input
                      v-model="localDeadline"
                      type="number"
                      min="1"
                      max="180"
                      placeholder="20"
                      class="w-full bg-slate-800/50 border-2 border-slate-600/30 focus:border-moby-500/50 rounded-xl px-4 py-3 text-white placeholder-white/40 focus:outline-none focus:ring-2 focus:ring-moby-500/20"
                      @input="handleDeadlineChange"
                    />
                    <span class="absolute right-4 top-1/2 transform -translate-y-1/2 text-white/60 text-sm font-medium">
                      minutes
                    </span>
                  </div>
                  <p class="text-sm text-white/50 mt-2">
                    Your transaction will revert if pending for more than this time.
                  </p>
                </div>

                <!-- Advanced Settings -->
                <div>
                  <h4 class="text-lg font-semibold text-white mb-4">Advanced Options</h4>
                  <div class="space-y-4">
                    <!-- Privacy Mode -->
                    <div class="flex items-center justify-between p-4 bg-slate-800/30 rounded-xl">
                      <div class="flex-1">
                        <div class="text-white font-medium mb-1">Privacy Mode</div>
                        <div class="text-sm text-white/60">
                          Route through privacy pools to hide transaction details
                        </div>
                      </div>
                      <MobileToggle
                        v-model="localUsePrivacy"
                        @update:model-value="handlePrivacyChange"
                      />
                    </div>

                    <!-- MEV Protection -->
                    <div class="flex items-center justify-between p-4 bg-slate-800/30 rounded-xl">
                      <div class="flex-1">
                        <div class="text-white font-medium mb-1">MEV Protection</div>
                        <div class="text-sm text-white/60">
                          Protect against maximal extractable value attacks
                        </div>
                      </div>
                      <MobileToggle
                        v-model="localMevProtection"
                        @update:model-value="handleMevProtectionChange"
                      />
                    </div>

                    <!-- Auto Refresh -->
                    <div class="flex items-center justify-between p-4 bg-slate-800/30 rounded-xl">
                      <div class="flex-1">
                        <div class="text-white font-medium mb-1">Auto Refresh Quotes</div>
                        <div class="text-sm text-white/60">
                          Automatically refresh quotes every 10 seconds
                        </div>
                      </div>
                      <MobileToggle
                        v-model="localAutoRefresh"
                        @update:model-value="handleAutoRefreshChange"
                      />
                    </div>

                    <!-- Sound Notifications -->
                    <div class="flex items-center justify-between p-4 bg-slate-800/30 rounded-xl">
                      <div class="flex-1">
                        <div class="text-white font-medium mb-1">Sound Notifications</div>
                        <div class="text-sm text-white/60">
                          Play sounds for successful transactions
                        </div>
                      </div>
                      <MobileToggle
                        v-model="localSoundEnabled"
                        @update:model-value="handleSoundChange"
                      />
                    </div>
                  </div>
                </div>
              </div>

              <!-- Action Buttons -->
              <div class="sticky bottom-0 bg-gradient-to-t from-slate-900 via-slate-900/95 to-transparent pt-6 pb-safe">
                <div class="space-y-3">
                  <Button
                    @click="saveSettings"
                    variant="whale"
                    size="lg"
                    full
                    class="h-14 text-lg font-semibold"
                  >
                    Save Settings
                  </Button>

                  <Button
                    @click="resetToDefaults"
                    variant="ghost"
                    size="lg"
                    full
                    class="h-12"
                  >
                    Reset to Defaults
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'
import MobileToggle from './MobileToggle.vue'

import { useTradingStore } from '@/stores/trading'
import { useNotificationStore } from '@/stores/notifications'

const emit = defineEmits<{
  close: []
}>()

const tradingStore = useTradingStore()
const notificationStore = useNotificationStore()

// Local state
const localSlippage = ref<number>(0.5)
const localDeadline = ref<number>(20)
const localUsePrivacy = ref<boolean>(false)
const localMevProtection = ref<boolean>(true)
const localAutoRefresh = ref<boolean>(true)
const localSoundEnabled = ref<boolean>(true)

const customSlippage = ref<string>('')

const slippagePresets = [0.1, 0.5, 1.0, 3.0]

// Methods
function setSlippage(value: number) {
  localSlippage.value = value
  customSlippage.value = ''
}

function handleCustomSlippage(event: Event) {
  const target = event.target as HTMLInputElement
  const value = target.value
  const numValue = parseFloat(value)

  if (!isNaN(numValue) && numValue >= 0.1 && numValue <= 50) {
    localSlippage.value = numValue
  }
}

function handleDeadlineChange(event: Event) {
  const target = event.target as HTMLInputElement
  const value = target.value
  const numValue = parseInt(value)

  if (!isNaN(numValue) && numValue >= 1 && numValue <= 180) {
    localDeadline.value = numValue
  }
}

function handlePrivacyChange(value: boolean) {
  localUsePrivacy.value = value
}

function handleMevProtectionChange(value: boolean) {
  localMevProtection.value = value
}

function handleAutoRefreshChange(value: boolean) {
  localAutoRefresh.value = value
}

function handleSoundChange(value: boolean) {
  localSoundEnabled.value = value
}

function resetToDefaults() {
  localSlippage.value = 0.5
  localDeadline.value = 20
  localUsePrivacy.value = false
  localMevProtection.value = true
  localAutoRefresh.value = true
  localSoundEnabled.value = true
  customSlippage.value = ''

  notificationStore.notifySystem(
    'Settings Reset',
    'All settings have been reset to defaults',
    'info'
  )
}

function saveSettings() {
  // Update trading store
  tradingStore.setSlippage(localSlippage.value)
  tradingStore.setDeadline(localDeadline.value)
  tradingStore.updatePreferences({
    usePrivacy: localUsePrivacy.value,
    mevProtection: localMevProtection.value,
    autoRefresh: localAutoRefresh.value,
    soundEnabled: localSoundEnabled.value
  })

  notificationStore.notifySystem(
    'Settings Saved',
    'Your swap settings have been updated',
    'success'
  )

  emit('close')
}

// Load current settings
onMounted(() => {
  localSlippage.value = tradingStore.defaultSlippage
  localDeadline.value = tradingStore.defaultDeadline
  localUsePrivacy.value = tradingStore.usePrivacy
  localMevProtection.value = tradingStore.mevProtection
  localAutoRefresh.value = tradingStore.autoRefresh
  localSoundEnabled.value = tradingStore.soundEnabled

  // Prevent body scroll when modal is open
  document.body.style.overflow = 'hidden'

  // Cleanup on unmount
  return () => {
    document.body.style.overflow = ''
  }
})
</script>

<style scoped>
/* Safe area support */
.pb-safe {
  padding-bottom: env(safe-area-inset-bottom);
}

/* Active scale effect */
.active\:scale-95:active {
  transform: scale(0.95);
}

/* Settings sheet animations */
.settings-overlay-enter-active,
.settings-overlay-leave-active {
  transition: opacity 0.3s ease;
}

.settings-overlay-enter-from,
.settings-overlay-leave-to {
  opacity: 0;
}

.settings-content-enter-active {
  transition: transform 0.3s ease-out;
}

.settings-content-leave-active {
  transition: transform 0.2s ease-in;
}

.settings-content-enter-from,
.settings-content-leave-to {
  transform: translateY(100%);
}
</style>