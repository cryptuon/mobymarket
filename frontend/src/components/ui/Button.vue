<template>
  <component
    :is="tag"
    :to="to"
    :href="href"
    :type="tag === 'button' ? type : undefined"
    :disabled="disabled || loading"
    :class="buttonClass"
    v-bind="$attrs"
  >
    <!-- Loading Spinner -->
    <div
      v-if="loading"
      class="animate-spin rounded-full border-2 border-transparent border-t-current mr-2"
      :class="spinnerSizeClass"
    ></div>

    <!-- Icon (Left) -->
    <HeroIcon
      v-if="iconLeft && !loading"
      :name="iconLeft"
      :class="iconSizeClass"
      class="mr-2"
    />

    <!-- Button Content -->
    <span v-if="$slots.default || label">
      <slot>{{ label }}</slot>
    </span>

    <!-- Icon (Right) -->
    <HeroIcon
      v-if="iconRight"
      :name="iconRight"
      :class="iconSizeClass"
      class="ml-2"
    />

    <!-- Badge -->
    <Badge
      v-if="badge"
      :variant="badgeVariant"
      size="sm"
      class="ml-2"
    >
      {{ badge }}
    </Badge>
  </component>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

import HeroIcon from './HeroIcon.vue'
import Badge from './Badge.vue'

interface Props {
  // Content
  label?: string
  iconLeft?: string
  iconRight?: string
  badge?: string | number

  // Appearance
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'whale' | 'ghost' | 'outline'
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  rounded?: boolean
  full?: boolean

  // State
  loading?: boolean
  disabled?: boolean

  // Navigation
  to?: string
  href?: string

  // Button specific
  type?: 'button' | 'submit' | 'reset'

  // Badge
  badgeVariant?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'whale'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  type: 'button',
  rounded: true,
  badgeVariant: 'primary',
})

// Determine component tag
const tag = computed(() => {
  if (props.to) return RouterLink
  if (props.href) return 'a'
  return 'button'
})

// Button classes
const buttonClass = computed(() => [
  // Base styles
  'inline-flex items-center justify-center font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-moby-500/50 disabled:opacity-50 disabled:cursor-not-allowed',

  // Size classes
  {
    'px-2 py-1 text-xs': props.size === 'xs',
    'px-3 py-1.5 text-sm': props.size === 'sm',
    'px-4 py-2 text-sm': props.size === 'md',
    'px-6 py-2.5 text-base': props.size === 'lg',
    'px-8 py-3 text-lg': props.size === 'xl',
  },

  // Width
  {
    'w-full': props.full,
  },

  // Rounded classes
  {
    'rounded': props.rounded && props.size === 'xs',
    'rounded-md': props.rounded && props.size === 'sm',
    'rounded-lg': props.rounded && (props.size === 'md' || props.size === 'lg'),
    'rounded-xl': props.rounded && props.size === 'xl',
    'rounded-none': !props.rounded,
  },

  // Variant classes
  {
    // Primary
    'bg-gradient-to-r from-moby-500 to-moby-600 hover:from-moby-600 hover:to-moby-700 text-white shadow-glow hover:shadow-glow-lg': props.variant === 'primary',

    // Secondary
    'bg-slate-700/50 hover:bg-slate-600/50 text-white border border-slate-600/50 hover:border-slate-500/50': props.variant === 'secondary',

    // Success
    'bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 text-white': props.variant === 'success',

    // Warning
    'bg-gradient-to-r from-yellow-500 to-yellow-600 hover:from-yellow-600 hover:to-yellow-700 text-white': props.variant === 'warning',

    // Error
    'bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 text-white': props.variant === 'error',

    // Info
    'bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white': props.variant === 'info',

    // Whale (Special gradient)
    'bg-gradient-to-r from-moby-400 via-blue-500 to-purple-600 hover:from-moby-500 hover:via-blue-600 hover:to-purple-700 text-white shadow-glow-xl': props.variant === 'whale',

    // Ghost
    'bg-transparent hover:bg-white/5 text-white/80 hover:text-white border border-transparent': props.variant === 'ghost',

    // Outline
    'bg-transparent hover:bg-moby-500/10 text-moby-400 hover:text-moby-300 border border-moby-500/30 hover:border-moby-500/50': props.variant === 'outline',
  }
])

// Icon size classes
const iconSizeClass = computed(() => {
  switch (props.size) {
    case 'xs': return 'w-3 h-3'
    case 'sm': return 'w-4 h-4'
    case 'md': return 'w-5 h-5'
    case 'lg': return 'w-5 h-5'
    case 'xl': return 'w-6 h-6'
    default: return 'w-5 h-5'
  }
})

// Spinner size classes
const spinnerSizeClass = computed(() => {
  switch (props.size) {
    case 'xs': return 'w-3 h-3'
    case 'sm': return 'w-4 h-4'
    case 'md': return 'w-4 h-4'
    case 'lg': return 'w-5 h-5'
    case 'xl': return 'w-6 h-6'
    default: return 'w-4 h-4'
  }
})
</script>

<style scoped>
/* Custom glow effects */
.shadow-glow {
  box-shadow: 0 0 20px rgba(14, 165, 233, 0.3);
}

.shadow-glow-lg {
  box-shadow: 0 0 30px rgba(14, 165, 233, 0.4);
}

.shadow-glow-xl {
  box-shadow: 0 0 40px rgba(14, 165, 233, 0.5), 0 0 80px rgba(147, 51, 234, 0.3);
}

/* Hover animations */
.transition-all {
  transition-property: all;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
}
</style>