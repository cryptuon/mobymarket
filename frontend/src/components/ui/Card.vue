<template>
  <component
    :is="tag"
    :to="to"
    :href="href"
    :class="cardClass"
    v-bind="$attrs"
  >
    <!-- Header -->
    <div
      v-if="$slots.header || title || subtitle || headerActions"
      :class="headerClass"
    >
      <div v-if="title || subtitle" class="flex-1 min-w-0">
        <!-- Icon + Title -->
        <div v-if="title" class="flex items-center space-x-3">
          <div
            v-if="icon"
            :class="[
              'flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center',
              iconBackground
            ]"
          >
            <HeroIcon :name="icon" class="w-5 h-5" :class="iconColor" />
          </div>
          <h3 :class="titleClass">{{ title }}</h3>
          <Badge v-if="badge" :variant="badgeVariant" size="sm">
            {{ badge }}
          </Badge>
        </div>

        <!-- Subtitle -->
        <p v-if="subtitle" :class="subtitleClass">
          {{ subtitle }}
        </p>
      </div>

      <!-- Header Slot -->
      <div v-if="$slots.header" class="flex-1">
        <slot name="header" />
      </div>

      <!-- Header Actions -->
      <div v-if="$slots.headerActions || headerActions" class="flex-shrink-0">
        <slot name="headerActions">
          <div class="flex items-center space-x-2">
            <component
              v-for="action in headerActions"
              :key="action.label"
              :is="action.component || 'button'"
              v-bind="action.props"
              @click="action.onClick"
              class="p-2 text-white/60 hover:text-white hover:bg-white/10 rounded-lg transition-all"
            >
              <HeroIcon v-if="action.icon" :name="action.icon" class="w-4 h-4" />
              <span v-if="action.label" class="sr-only">{{ action.label }}</span>
            </component>
          </div>
        </slot>
      </div>
    </div>

    <!-- Content -->
    <div v-if="$slots.default" :class="contentClass">
      <slot />
    </div>

    <!-- Footer -->
    <div
      v-if="$slots.footer"
      :class="footerClass"
    >
      <slot name="footer" />
    </div>

    <!-- Loading Overlay -->
    <div
      v-if="loading"
      class="absolute inset-0 bg-slate-900/50 backdrop-blur-sm rounded-lg flex items-center justify-center"
    >
      <div class="flex items-center space-x-3 text-white">
        <div class="animate-spin rounded-full h-6 w-6 border-2 border-white/20 border-t-white"></div>
        <span v-if="loadingText" class="text-sm">{{ loadingText }}</span>
      </div>
    </div>
  </component>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

import HeroIcon from './HeroIcon.vue'
import Badge from './Badge.vue'

interface HeaderAction {
  icon?: string
  label: string
  onClick: () => void
  component?: string
  props?: Record<string, any>
}

interface Props {
  // Content
  title?: string
  subtitle?: string
  icon?: string
  badge?: string | number
  badgeVariant?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'whale'

  // Appearance
  variant?: 'default' | 'glass' | 'whale' | 'premium' | 'outline'
  size?: 'sm' | 'md' | 'lg'
  rounded?: boolean
  shadow?: boolean
  glow?: boolean

  // Layout
  padding?: 'none' | 'sm' | 'md' | 'lg'
  full?: boolean

  // State
  loading?: boolean
  loadingText?: string
  disabled?: boolean
  clickable?: boolean

  // Navigation
  to?: string
  href?: string

  // Actions
  headerActions?: HeaderAction[]
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  size: 'md',
  padding: 'md',
  rounded: true,
  shadow: true,
  badgeVariant: 'primary',
})

// Determine component tag
const tag = computed(() => {
  if (props.to) return RouterLink
  if (props.href) return 'a'
  if (props.clickable) return 'button'
  return 'div'
})

// Card classes
const cardClass = computed(() => [
  // Base styles
  'relative overflow-hidden transition-all duration-200',

  // Interactive styles
  {
    'cursor-pointer hover:scale-[1.02] focus:outline-none focus:ring-2 focus:ring-moby-500/50': props.clickable || props.to || props.href,
    'opacity-50 cursor-not-allowed': props.disabled,
  },

  // Size classes
  {
    'text-sm': props.size === 'sm',
    'text-base': props.size === 'md',
    'text-lg': props.size === 'lg',
  },

  // Width
  {
    'w-full': props.full,
  },

  // Border radius
  {
    'rounded-lg': props.rounded && props.size === 'sm',
    'rounded-xl': props.rounded && props.size === 'md',
    'rounded-2xl': props.rounded && props.size === 'lg',
    'rounded-none': !props.rounded,
  },

  // Shadow
  {
    'shadow-lg': props.shadow && props.variant !== 'outline',
    'shadow-xl': props.shadow && props.size === 'lg',
  },

  // Glow effect
  {
    'shadow-glow': props.glow,
  },

  // Variant styles
  {
    // Default
    'bg-slate-800/50 backdrop-blur-sm border border-slate-700/50': props.variant === 'default',

    // Glass
    'bg-glass-dark backdrop-blur-xl border border-white/10': props.variant === 'glass',

    // Whale
    'bg-gradient-to-br from-slate-800/50 to-moby-900/20 backdrop-blur-sm border border-moby-500/20 shadow-glow': props.variant === 'whale',

    // Premium
    'bg-gradient-to-br from-slate-800/80 to-slate-900/80 backdrop-blur-xl border border-gradient-to-r border-moby-500/30 shadow-glow-lg': props.variant === 'premium',

    // Outline
    'bg-transparent border-2 border-slate-600/50 hover:border-slate-500/50': props.variant === 'outline',
  }
])

// Header classes
const headerClass = computed(() => [
  'flex items-start justify-between',
  {
    'p-3': props.padding === 'sm',
    'p-4': props.padding === 'md',
    'p-6': props.padding === 'lg',
    'border-b border-white/10': true, // Always show header border
  }
])

// Content classes
const contentClass = computed(() => [
  {
    'p-3': props.padding === 'sm',
    'p-4': props.padding === 'md',
    'p-6': props.padding === 'lg',
  }
])

// Footer classes
const footerClass = computed(() => [
  'border-t border-white/10',
  {
    'p-3': props.padding === 'sm',
    'p-4': props.padding === 'md',
    'p-6': props.padding === 'lg',
  }
])

// Title classes
const titleClass = computed(() => [
  'font-semibold text-white',
  {
    'text-sm': props.size === 'sm',
    'text-base': props.size === 'md',
    'text-lg': props.size === 'lg',
  }
])

// Subtitle classes
const subtitleClass = computed(() => [
  'text-white/60 mt-1',
  {
    'text-xs': props.size === 'sm',
    'text-sm': props.size === 'md' || props.size === 'lg',
  }
])

// Icon styling
const iconBackground = computed(() => {
  switch (props.variant) {
    case 'whale':
      return 'bg-moby-500/20'
    case 'premium':
      return 'bg-gradient-to-r from-moby-500/20 to-purple-500/20'
    default:
      return 'bg-white/10'
  }
})

const iconColor = computed(() => {
  switch (props.variant) {
    case 'whale':
    case 'premium':
      return 'text-moby-400'
    default:
      return 'text-white/70'
  }
})
</script>

<style scoped>
.shadow-glow {
  box-shadow: 0 0 20px rgba(14, 165, 233, 0.1);
}

.shadow-glow-lg {
  box-shadow: 0 0 30px rgba(14, 165, 233, 0.15), 0 0 60px rgba(147, 51, 234, 0.1);
}

/* Gradient border for premium variant */
.border-gradient-to-r {
  border-image: linear-gradient(to right, rgba(14, 165, 233, 0.3), rgba(147, 51, 234, 0.3)) 1;
}

/* Hover effects */
.hover\:scale-\[1\.02\]:hover {
  transform: scale(1.02);
}
</style>