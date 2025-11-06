<template>
  <button
    @click="toggleTheme"
    class="p-2 rounded-lg text-white/70 hover:text-white hover:bg-white/10 transition-all duration-200"
    :aria-label="`Switch to ${nextTheme} theme`"
  >
    <Transition
      name="theme-icon"
      mode="out-in"
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="transform rotate-180 scale-0"
      enter-to-class="transform rotate-0 scale-100"
      leave-active-class="transition-all duration-300 ease-in"
      leave-from-class="transform rotate-0 scale-100"
      leave-to-class="transform -rotate-180 scale-0"
    >
      <HeroIcon
        v-if="currentTheme === 'dark'"
        key="moon"
        name="MoonIcon"
        class="w-6 h-6"
      />
      <HeroIcon
        v-else-if="currentTheme === 'light'"
        key="sun"
        name="SunIcon"
        class="w-6 h-6"
      />
      <HeroIcon
        v-else
        key="computer"
        name="ComputerDesktopIcon"
        class="w-6 h-6"
      />
    </Transition>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'

import HeroIcon from '@components/ui/HeroIcon.vue'

import { useThemeStore } from '@stores/theme'

const themeStore = useThemeStore()
const { currentTheme } = storeToRefs(themeStore)

// Computed properties
const nextTheme = computed(() => {
  switch (currentTheme.value) {
    case 'light':
      return 'dark'
    case 'dark':
      return 'system'
    case 'system':
      return 'light'
    default:
      return 'dark'
  }
})

// Methods
function toggleTheme() {
  const themes = ['light', 'dark', 'system'] as const
  const currentIndex = themes.indexOf(currentTheme.value)
  const nextIndex = (currentIndex + 1) % themes.length
  themeStore.setTheme(themes[nextIndex])
}
</script>

<style scoped>
.theme-icon-enter-active,
.theme-icon-leave-active {
  transition: all 0.3s ease;
}

.theme-icon-enter-from {
  transform: rotate(180deg) scale(0);
  opacity: 0;
}

.theme-icon-leave-to {
  transform: rotate(-180deg) scale(0);
  opacity: 0;
}
</style>