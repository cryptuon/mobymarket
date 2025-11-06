<template>
  <button
    :class="toggleClass"
    class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-moby-500/50 focus:ring-offset-2 focus:ring-offset-slate-900 disabled:opacity-50 disabled:cursor-not-allowed"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    @click="toggle"
  >
    <span class="sr-only">{{ label || 'Toggle' }}</span>
    <span
      :class="knobClass"
      class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
    />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  modelValue: boolean
  disabled?: boolean
  label?: string
  size?: 'sm' | 'md' | 'lg'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md'
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
  },
  // Size variants
  {
    'h-5 w-9': props.size === 'sm',
    'h-6 w-11': props.size === 'md',
    'h-7 w-13': props.size === 'lg',
  }
])

const knobClass = computed(() => [
  {
    // Position based on state
    'translate-x-5': props.modelValue && props.size === 'md',
    'translate-x-0': !props.modelValue && props.size === 'md',

    'translate-x-4': props.modelValue && props.size === 'sm',
    'translate-x-6': props.modelValue && props.size === 'lg',
  },
  // Size variants for knob
  {
    'h-4 w-4': props.size === 'sm',
    'h-5 w-5': props.size === 'md',
    'h-6 w-6': props.size === 'lg',
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
/* Ensure proper width for large toggle */
.w-13 {
  width: 3.25rem;
}
</style>