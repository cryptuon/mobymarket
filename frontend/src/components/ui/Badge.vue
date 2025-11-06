<template>
  <span :class="badgeClass">
    <slot />
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'whale'
  size?: 'sm' | 'md' | 'lg'
  rounded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'sm',
  rounded: true,
})

const badgeClass = computed(() => [
  'inline-flex items-center justify-center font-medium',
  // Size classes
  {
    'px-2 py-0.5 text-xs': props.size === 'sm',
    'px-3 py-1 text-sm': props.size === 'md',
    'px-4 py-1.5 text-base': props.size === 'lg',
  },
  // Rounded classes
  {
    'rounded-full': props.rounded && props.size === 'sm',
    'rounded-lg': props.rounded && props.size === 'md',
    'rounded-xl': props.rounded && props.size === 'lg',
    'rounded-md': !props.rounded,
  },
  // Variant classes
  {
    'bg-moby-500/20 text-moby-400 border border-moby-500/30': props.variant === 'primary',
    'bg-whale-500/20 text-whale-300 border border-whale-500/30': props.variant === 'secondary',
    'bg-green-500/20 text-green-400 border border-green-500/30': props.variant === 'success',
    'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30': props.variant === 'warning',
    'bg-red-500/20 text-red-400 border border-red-500/30': props.variant === 'error',
    'bg-blue-500/20 text-blue-400 border border-blue-500/30': props.variant === 'info',
    'bg-gradient-to-r from-moby-500 to-moby-600 text-white shadow-glow': props.variant === 'whale',
  },
])
</script>