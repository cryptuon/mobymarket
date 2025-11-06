<template>
  <RouterLink
    :to="to"
    :class="mobileNavItemClass"
    class="flex items-center space-x-3 px-4 py-3 rounded-lg font-medium transition-all duration-200"
  >
    <HeroIcon v-if="icon" :name="icon" class="w-6 h-6 flex-shrink-0" />
    <span class="flex-1">
      <slot />
    </span>
    <Badge v-if="badge" :variant="badgeVariant" size="sm">
      {{ badge }}
    </Badge>
    <HeroIcon name="ChevronRightIcon" class="w-4 h-4 text-white/40" />
  </RouterLink>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import HeroIcon from '@components/ui/HeroIcon.vue'
import Badge from '@components/ui/Badge.vue'

interface Props {
  to: string
  icon?: string
  badge?: string | number
  badgeVariant?: 'primary' | 'success' | 'warning' | 'error' | 'info'
}

const props = withDefaults(defineProps<Props>(), {
  badgeVariant: 'primary',
})

const route = useRoute()

const isActive = computed(() => {
  if (props.to === '/') {
    return route.path === '/'
  }
  return route.path.startsWith(props.to)
})

const mobileNavItemClass = computed(() => [
  'text-white/80 hover:text-white hover:bg-white/10',
  {
    'text-moby-400 bg-moby-500/10 border-l-2 border-l-moby-400': isActive.value,
  },
])
</script>