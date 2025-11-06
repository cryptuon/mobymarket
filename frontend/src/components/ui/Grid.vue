<template>
  <div :class="gridClass">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Breakpoint } from '@/composables/useBreakpoints'

interface ResponsiveColumns {
  xs?: number
  sm?: number
  md?: number
  lg?: number
  xl?: number
  '2xl'?: number
}

interface Props {
  // Column configuration
  cols?: number | ResponsiveColumns

  // Gap configuration
  gap?: number | string
  gapX?: number | string
  gapY?: number | string

  // Alignment
  justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'
  align?: 'start' | 'center' | 'end' | 'stretch' | 'baseline'

  // Flow
  flow?: 'row' | 'col' | 'row-dense' | 'col-dense'

  // Auto sizing
  autoRows?: 'auto' | 'min' | 'max' | 'fr' | string
  autoCols?: 'auto' | 'min' | 'max' | 'fr' | string

  // Layout type
  type?: 'grid' | 'flex'
}

const props = withDefaults(defineProps<Props>(), {
  cols: 12,
  gap: 4,
  justify: 'start',
  align: 'stretch',
  flow: 'row',
  type: 'grid'
})

// Computed classes
const gridClass = computed(() => {
  const classes: string[] = []

  if (props.type === 'grid') {
    classes.push('grid')

    // Handle columns
    if (typeof props.cols === 'number') {
      classes.push(`grid-cols-${props.cols}`)
    } else if (typeof props.cols === 'object') {
      // Responsive columns
      Object.entries(props.cols).forEach(([breakpoint, cols]) => {
        if (breakpoint === 'xs') {
          classes.push(`grid-cols-${cols}`)
        } else {
          classes.push(`${breakpoint}:grid-cols-${cols}`)
        }
      })
    }

    // Grid flow
    if (props.flow !== 'row') {
      classes.push(`grid-flow-${props.flow}`)
    }

    // Auto rows/cols
    if (props.autoRows !== undefined) {
      if (props.autoRows === 'auto') classes.push('auto-rows-auto')
      else if (props.autoRows === 'min') classes.push('auto-rows-min')
      else if (props.autoRows === 'max') classes.push('auto-rows-max')
      else if (props.autoRows === 'fr') classes.push('auto-rows-fr')
      else classes.push(`auto-rows-[${props.autoRows}]`)
    }

    if (props.autoCols !== undefined) {
      if (props.autoCols === 'auto') classes.push('auto-cols-auto')
      else if (props.autoCols === 'min') classes.push('auto-cols-min')
      else if (props.autoCols === 'max') classes.push('auto-cols-max')
      else if (props.autoCols === 'fr') classes.push('auto-cols-fr')
      else classes.push(`auto-cols-[${props.autoCols}]`)
    }
  } else {
    classes.push('flex', 'flex-wrap')
  }

  // Gap classes
  if (props.gapX !== undefined || props.gapY !== undefined) {
    if (props.gapX !== undefined) {
      classes.push(typeof props.gapX === 'number' ? `gap-x-${props.gapX}` : `gap-x-[${props.gapX}]`)
    }
    if (props.gapY !== undefined) {
      classes.push(typeof props.gapY === 'number' ? `gap-y-${props.gapY}` : `gap-y-[${props.gapY}]`)
    }
  } else if (props.gap !== undefined) {
    classes.push(typeof props.gap === 'number' ? `gap-${props.gap}` : `gap-[${props.gap}]`)
  }

  // Justify content
  if (props.justify !== 'start') {
    const justifyMap = {
      center: 'justify-center',
      end: 'justify-end',
      between: 'justify-between',
      around: 'justify-around',
      evenly: 'justify-evenly'
    }
    classes.push(justifyMap[props.justify])
  }

  // Align items
  if (props.align !== 'stretch') {
    const alignMap = {
      start: 'items-start',
      center: 'items-center',
      end: 'items-end',
      baseline: 'items-baseline'
    }
    classes.push(alignMap[props.align])
  }

  return classes.join(' ')
})
</script>