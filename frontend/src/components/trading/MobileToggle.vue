<template>
  <button
    :class="toggleClass"
    class="relative inline-flex h-8 w-14 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-moby-500/50 focus:ring-offset-2 focus:ring-offset-slate-900 disabled:opacity-50 disabled:cursor-not-allowed active:scale-95"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    @click="toggle"
  >
    <span class="sr-only">{{ label || 'Toggle' }}</span>
    <span
      :class="knobClass"
      class="pointer-events-none inline-block h-7 w-7 transform rounded-full bg-white shadow-lg ring-0 transition duration-200 ease-in-out"
    />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  modelValue: boolean
  disabled?: boolean
  label?: string
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

// Computed properties
const toggleClass = computed(() => [
  {
    // Enabled state
    'bg-moby-500': props.modelValue && !props.disabled,
    'bg-slate-600': !props.modelValue && !props.disabled,

    // Disabled state
    'bg-moby-300': props.modelValue && props.disabled,
    'bg-slate-500': !props.modelValue && props.disabled,
  }
])

const knobClass = computed(() => [
  {
    // Position based on state
    'translate-x-6': props.modelValue,
    'translate-x-0': !props.modelValue,
  }
])

// Methods
function toggle() {
  if (!props.disabled) {
    emit('update:modelValue', !props.modelValue)
  }
}
</script>

<style scoped>
/* Active scale effect for mobile interaction */
.active\:scale-95:active {
  transform: scale(0.95);
}
</style>