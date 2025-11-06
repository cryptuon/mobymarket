<template>
  <component :is="iconComponent" v-bind="$attrs" />
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'

interface Props {
  name: string
  outline?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  outline: false,
})

const iconComponent = computed(() => {
  const iconType = props.outline ? 'outline' : 'solid'

  return defineAsyncComponent(() =>
    import(`@heroicons/vue/24/${iconType}/index.js`).then((module) => {
      const icon = module[props.name]
      if (!icon) {
        console.warn(`Icon "${props.name}" not found in heroicons/${iconType}`)
        return module.QuestionMarkCircleIcon || (() => null)
      }
      return icon
    }).catch(() => {
      console.error(`Failed to load icon "${props.name}"`)
      return () => null
    })
  )
})
</script>