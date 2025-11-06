<template>
  <RouterLink
    :to="to"
    :class="navItemClass"
    class="group relative px-4 py-2 rounded-lg font-medium transition-all duration-200 flex items-center space-x-2"
  >
    <HeroIcon v-if="icon" :name="icon" class="w-5 h-5" />
    <span>
      <slot />
    </span>
    <Badge v-if="badge" :variant="badgeVariant" class="ml-2">
      {{ badge }}
    </Badge>

    <!-- Active indicator -->
    <div
      v-if="isActive"
      class="absolute bottom-0 left-1/2 transform -translate-x-1/2 w-1 h-1 bg-moby-400 rounded-full"
    />
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

const navItemClass = computed(() => [
  'text-white/70 hover:text-white hover:bg-white/10',
  {
    'text-moby-400 bg-moby-500/10': isActive.value,
  },
])
</script>