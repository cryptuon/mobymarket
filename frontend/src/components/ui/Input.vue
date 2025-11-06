<template>
  <div :class="wrapperClass">
    <!-- Label -->
    <label
      v-if="label"
      :for="inputId"
      class="block text-sm font-medium text-white mb-2"
    >
      {{ label }}
      <span v-if="required" class="text-red-400 ml-1">*</span>
    </label>

    <!-- Input Container -->
    <div class="relative">
      <!-- Icon Left -->
      <div
        v-if="iconLeft"
        class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none"
      >
        <HeroIcon :name="iconLeft" :class="iconClass" />
      </div>

      <!-- Input Element -->
      <input
        :id="inputId"
        ref="inputRef"
        :type="computedType"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :autocomplete="autocomplete"
        :min="min"
        :max="max"
        :step="step"
        :pattern="pattern"
        :class="inputClass"
        v-bind="$attrs"
        @input="handleInput"
        @blur="handleBlur"
        @focus="handleFocus"
      />

      <!-- Icon Right / Actions -->
      <div class="absolute inset-y-0 right-0 flex items-center">
        <!-- Clear Button -->
        <button
          v-if="clearable && modelValue && !disabled"
          @click="clearInput"
          class="p-2 text-white/40 hover:text-white/70 transition-colors"
          type="button"
          aria-label="Clear input"
        >
          <HeroIcon name="XMarkIcon" class="w-4 h-4" />
        </button>

        <!-- Password Toggle -->
        <button
          v-if="type === 'password'"
          @click="togglePasswordVisibility"
          class="p-2 text-white/40 hover:text-white/70 transition-colors"
          type="button"
          :aria-label="showPassword ? 'Hide password' : 'Show password'"
        >
          <HeroIcon :name="showPassword ? 'EyeSlashIcon' : 'EyeIcon'" class="w-4 h-4" />
        </button>

        <!-- Icon Right -->
        <div
          v-if="iconRight"
          class="pr-3 flex items-center pointer-events-none"
        >
          <HeroIcon :name="iconRight" :class="iconClass" />
        </div>

        <!-- Loading Spinner -->
        <div
          v-if="loading"
          class="pr-3 flex items-center"
        >
          <div class="animate-spin rounded-full h-4 w-4 border-2 border-white/20 border-t-white/60"></div>
        </div>
      </div>
    </div>

    <!-- Help Text -->
    <p
      v-if="helpText && !error"
      class="mt-2 text-sm text-white/60"
    >
      {{ helpText }}
    </p>

    <!-- Error Message -->
    <p
      v-if="error"
      class="mt-2 text-sm text-red-400 flex items-center"
    >
      <HeroIcon name="ExclamationCircleIcon" class="w-4 h-4 mr-1" />
      {{ error }}
    </p>

    <!-- Success Message -->
    <p
      v-if="success"
      class="mt-2 text-sm text-green-400 flex items-center"
    >
      <HeroIcon name="CheckCircleIcon" class="w-4 h-4 mr-1" />
      {{ success }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'

import HeroIcon from './HeroIcon.vue'

interface Props {
  // Core
  modelValue?: string | number
  type?: 'text' | 'email' | 'password' | 'number' | 'tel' | 'url' | 'search'
  placeholder?: string
  label?: string
  helpText?: string

  // Validation
  required?: boolean
  error?: string
  success?: string

  // Appearance
  size?: 'sm' | 'md' | 'lg'
  variant?: 'default' | 'whale'
  full?: boolean

  // Icons
  iconLeft?: string
  iconRight?: string

  // State
  disabled?: boolean
  readonly?: boolean
  loading?: boolean
  clearable?: boolean

  // Number specific
  min?: number
  max?: number
  step?: number

  // Validation
  pattern?: string

  // HTML attributes
  autocomplete?: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text',
  size: 'md',
  variant: 'default',
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
  focus: [event: FocusEvent]
  blur: [event: FocusEvent]
  clear: []
}>()

const inputRef = ref<HTMLInputElement>()
const showPassword = ref(false)
const isFocused = ref(false)

// Generate unique ID for label association
const inputId = computed(() => `input-${Math.random().toString(36).substr(2, 9)}`)

// Password visibility toggle
const computedType = computed(() => {
  if (props.type === 'password' && showPassword.value) {
    return 'text'
  }
  return props.type
})

// Classes
const wrapperClass = computed(() => [
  props.full ? 'w-full' : ''
])

const inputClass = computed(() => [
  // Base styles
  'block w-full bg-slate-800/50 backdrop-blur-sm border text-white placeholder-white/40 focus:outline-none focus:ring-2 transition-all duration-200',

  // Size classes
  {
    'px-3 py-2 text-sm': props.size === 'sm',
    'px-4 py-2.5 text-sm': props.size === 'md',
    'px-5 py-3 text-base': props.size === 'lg',
  },

  // Padding adjustments for icons
  {
    'pl-10': props.iconLeft && props.size === 'sm',
    'pl-11': props.iconLeft && props.size === 'md',
    'pl-12': props.iconLeft && props.size === 'lg',
    'pr-10': props.iconRight || props.clearable || props.type === 'password' || props.loading,
  },

  // Border radius
  {
    'rounded-md': props.size === 'sm',
    'rounded-lg': props.size === 'md' || props.size === 'lg',
  },

  // Variant styles
  {
    // Default
    'border-slate-600/50 focus:border-moby-500/50 focus:ring-moby-500/20': props.variant === 'default' && !props.error && !props.success,

    // Whale
    'border-moby-500/30 focus:border-moby-400/70 focus:ring-moby-400/30 shadow-glow': props.variant === 'whale' && !props.error && !props.success,
  },

  // State classes
  {
    'border-red-500/50 focus:border-red-500/70 focus:ring-red-500/20': props.error,
    'border-green-500/50 focus:border-green-500/70 focus:ring-green-500/20': props.success,
    'opacity-50 cursor-not-allowed': props.disabled,
    'bg-slate-700/30': props.readonly,
  }
])

const iconClass = computed(() => [
  'text-white/40',
  props.size === 'sm' ? 'w-4 h-4' : 'w-5 h-5'
])

// Methods
function handleInput(event: Event) {
  const target = event.target as HTMLInputElement
  let value: string | number = target.value

  if (props.type === 'number' && value !== '') {
    value = parseFloat(value)
  }

  emit('update:modelValue', value)
}

function handleFocus(event: FocusEvent) {
  isFocused.value = true
  emit('focus', event)
}

function handleBlur(event: FocusEvent) {
  isFocused.value = false
  emit('blur', event)
}

function togglePasswordVisibility() {
  showPassword.value = !showPassword.value
  nextTick(() => {
    inputRef.value?.focus()
  })
}

function clearInput() {
  emit('update:modelValue', '')
  emit('clear')
  nextTick(() => {
    inputRef.value?.focus()
  })
}

// Expose focus method
function focus() {
  inputRef.value?.focus()
}

function blur() {
  inputRef.value?.blur()
}

defineExpose({
  focus,
  blur,
})
</script>

<style scoped>
.shadow-glow {
  box-shadow: 0 0 0 1px rgba(14, 165, 233, 0.1);
}

input:focus.shadow-glow {
  box-shadow: 0 0 0 1px rgba(14, 165, 233, 0.3), 0 0 20px rgba(14, 165, 233, 0.1);
}
</style>