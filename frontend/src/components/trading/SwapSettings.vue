<template>
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
        class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
        @click="$emit('close')"
      >
        <Transition
          name="settings-content"
          enter-active-class="transition-all duration-300 ease-out"
          enter-from-class="transform scale-95 opacity-0"
          enter-to-class="transform scale-100 opacity-100"
          leave-active-class="transition-all duration-200 ease-in"
          leave-from-class="transform scale-100 opacity-100"
          leave-to-class="transform scale-95 opacity-0"
        >
          <Card
            variant="glass"
            size="lg"
            class="w-full max-w-lg mx-auto"
            @click.stop
          >
            <template #header>
              <div class="flex items-center justify-between w-full">
                <h3 class="text-xl font-bold text-white">Swap Settings</h3>
                <button
                  @click="$emit('close')"
                  class="p-2 hover:bg-white/10 rounded-lg transition-colors"
                  aria-label="Close settings"
                >
                  <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
                </button>
              </div>
            </template>

            <div class="space-y-6">
              <!-- Slippage Tolerance -->
              <div>
                <label class="block text-sm font-medium text-white mb-3">
                  Slippage Tolerance
                </label>
                <div class="space-y-3">
                  <!-- Preset Buttons -->
                  <div class="flex space-x-2">
                    <button
                      v-for="preset in slippagePresets"
                      :key="preset"
                      @click="setSlippage(preset)"
                      :class="[
                        'px-3 py-2 rounded-lg text-sm font-medium transition-all',
                        localSlippage === preset
                          ? 'bg-moby-500/20 text-moby-400 border border-moby-500/30'
                          : 'bg-slate-800/50 text-white/80 border border-slate-600/50 hover:border-slate-500/50'
                      ]"
                    >
                      {{ preset }}%
                    </button>
                  </div>

                  <!-- Custom Input -->
                  <div class="flex items-center space-x-2">
                    <Input
                      v-model="customSlippage"
                      type="number"
                      step="0.1"
                      min="0.1"
                      max="50"
                      placeholder="Custom"
                      class="flex-1"
                      @input="handleCustomSlippage"
                    />
                    <span class="text-white/60 text-sm">%</span>
                  </div>

                  <!-- Warning Messages -->
                  <div v-if="localSlippage < 0.1" class="text-red-400 text-xs flex items-center space-x-1">
                    <HeroIcon name="ExclamationTriangleIcon" class="w-4 h-4" />
                    <span>Your transaction may fail</span>
                  </div>
                  <div v-else-if="localSlippage > 5" class="text-yellow-400 text-xs flex items-center space-x-1">
                    <HeroIcon name="ExclamationTriangleIcon" class="w-4 h-4" />
                    <span>Your transaction may be frontrun</span>
                  </div>
                  <div v-else class="text-green-400 text-xs flex items-center space-x-1">
                    <HeroIcon name="CheckCircleIcon" class="w-4 h-4" />
                    <span>Recommended slippage range</span>
                  </div>
                </div>
              </div>

              <!-- Transaction Deadline -->
              <div>
                <label class="block text-sm font-medium text-white mb-3">
                  Transaction Deadline
                </label>
                <div class="flex items-center space-x-2">
                  <Input
                    v-model="localDeadline"
                    type="number"
                    min="1"
                    max="180"
                    placeholder="20"
                    class="flex-1"
                    @input="handleDeadlineChange"
                  />
                  <span class="text-white/60 text-sm">minutes</span>
                </div>
                <p class="text-xs text-white/50 mt-2">
                  Your transaction will revert if it is pending for more than this long.
                </p>
              </div>

              <!-- Advanced Settings -->
              <div>
                <h4 class="text-sm font-medium text-white mb-3">Advanced Settings</h4>
                <div class="space-y-4">
                  <!-- Privacy Mode -->
                  <div class="flex items-center justify-between">
                    <div>
                      <div class="text-sm font-medium text-white">Privacy Mode</div>
                      <div class="text-xs text-white/60">
                        Route through privacy pools to hide transaction details
                      </div>
                    </div>
                    <Toggle
                      v-model="localUsePrivacy"
                      @update:model-value="handlePrivacyChange"
                    />
                  </div>

                  <!-- MEV Protection -->
                  <div class="flex items-center justify-between">
                    <div>
                      <div class="text-sm font-medium text-white">MEV Protection</div>
                      <div class="text-xs text-white/60">
                        Protect against maximal extractable value attacks
                      </div>
                    </div>
                    <Toggle
                      v-model="localMevProtection"
                      @update:model-value="handleMevProtectionChange"
                    />
                  </div>

                  <!-- Auto Refresh Quotes -->
                  <div class="flex items-center justify-between">
                    <div>
                      <div class="text-sm font-medium text-white">Auto Refresh Quotes</div>
                      <div class="text-xs text-white/60">
                        Automatically refresh quotes every 10 seconds
                      </div>
                    </div>
                    <Toggle
                      v-model="localAutoRefresh"
                      @update:model-value="handleAutoRefreshChange"
                    />
                  </div>

                  <!-- Sound Notifications -->
                  <div class="flex items-center justify-between">
                    <div>
                      <div class="text-sm font-medium text-white">Sound Notifications</div>
                      <div class="text-xs text-white/60">
                        Play sounds for successful transactions
                      </div>
                    </div>
                    <Toggle
                      v-model="localSoundEnabled"
                      @update:model-value="handleSoundChange"
                    />
                  </div>
                </div>
              </div>
            </div>

            <template #footer>
              <div class="flex items-center justify-between">
                <button
                  @click="resetToDefaults"
                  class="text-sm text-white/60 hover:text-white transition-colors"
                >
                  Reset to Defaults
                </button>
                <Button @click="saveSettings" variant="primary">
                  Save Settings
                </Button>
              </div>
            </template>
          </Card>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

import Card from '@components/ui/Card.vue'
import Input from '@components/ui/Input.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'
import Toggle from '@components/ui/Toggle.vue'

import { useTradingStore } from '@/stores/trading'
import { useNotificationStore } from '@/stores/notifications'

const emit = defineEmits<{
  close: []
}>()

const tradingStore = useTradingStore()
const notificationStore = useNotificationStore()

// Local state for settings
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

function handleCustomSlippage(value: string) {
  const numValue = parseFloat(value)
  if (!isNaN(numValue) && numValue >= 0.1 && numValue <= 50) {
    localSlippage.value = numValue
  }
}

function handleDeadlineChange(value: string) {
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
})
</script>

<style scoped>
/* Settings modal animations */
.settings-overlay-enter-active,
.settings-overlay-leave-active {
  transition: opacity 0.3s ease;
}

.settings-overlay-enter-from,
.settings-overlay-leave-to {
  opacity: 0;
}

.settings-content-enter-active {
  transition: all 0.3s ease-out;
}

.settings-content-leave-active {
  transition: all 0.2s ease-in;
}

.settings-content-enter-from,
.settings-content-leave-to {
  transform: scale(0.95);
  opacity: 0;
}
</style>