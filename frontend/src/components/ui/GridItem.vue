<template>
  <div :class="itemClass">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface ResponsiveSpan {
  xs?: number | 'auto' | 'full'
  sm?: number | 'auto' | 'full'
  md?: number | 'auto' | 'full'
  lg?: number | 'auto' | 'full'
  xl?: number | 'auto' | 'full'
  '2xl'?: number | 'auto' | 'full'
}

interface ResponsiveStart {
  xs?: number | 'auto'
  sm?: number | 'auto'
  md?: number | 'auto'
  lg?: number | 'auto'
  xl?: number | 'auto'
  '2xl'?: number | 'auto'
}

interface Props {
  // Column span
  colSpan?: number | 'auto' | 'full' | ResponsiveSpan

  // Row span
  rowSpan?: number | 'auto' | 'full' | ResponsiveSpan

  // Column start
  colStart?: number | 'auto' | ResponsiveStart

  // Row start
  rowStart?: number | 'auto' | ResponsiveStart

  // Column end
  colEnd?: number | 'auto' | ResponsiveStart

  // Row end
  rowEnd?: number | 'auto' | ResponsiveStart

  // Order
  order?: number | 'first' | 'last' | 'none'

  // Flex properties (when parent is flex)
  flex?: 'auto' | 'initial' | 'none' | '1'
  flexGrow?: boolean
  flexShrink?: boolean

  // Alignment overrides
  justifySelf?: 'auto' | 'start' | 'center' | 'end' | 'stretch'
  alignSelf?: 'auto' | 'start' | 'center' | 'end' | 'stretch' | 'baseline'
}

const props = defineProps<Props>()

// Helper function to generate responsive classes
function generateResponsiveClasses(
  prop: number | string | ResponsiveSpan | ResponsiveStart | undefined,
  prefix: string
): string[] {
  if (!prop) return []

  const classes: string[] = []

  if (typeof prop === 'object') {
    // Responsive object
    Object.entries(prop).forEach(([breakpoint, value]) => {
      if (value === undefined) return

      let className = ''
      if (breakpoint === 'xs') {
        className = `${prefix}-${value}`
      } else {
        className = `${breakpoint}:${prefix}-${value}`
      }
      classes.push(className)
    })
  } else {
    // Single value
    classes.push(`${prefix}-${prop}`)
  }

  return classes
}

// Computed classes
const itemClass = computed(() => {
  const classes: string[] = []

  // Column span
  classes.push(...generateResponsiveClasses(props.colSpan, 'col-span'))

  // Row span
  classes.push(...generateResponsiveClasses(props.rowSpan, 'row-span'))

  // Column start
  classes.push(...generateResponsiveClasses(props.colStart, 'col-start'))

  // Row start
  classes.push(...generateResponsiveClasses(props.rowStart, 'row-start'))

  // Column end
  classes.push(...generateResponsiveClasses(props.colEnd, 'col-end'))

  // Row end
  classes.push(...generateResponsiveClasses(props.rowEnd, 'row-end'))

  // Order
  if (props.order !== undefined) {
    if (typeof props.order === 'number') {
      classes.push(`order-${props.order}`)
    } else {
      classes.push(`order-${props.order}`)
    }
  }

  // Flex properties
  if (props.flex !== undefined) {
    if (props.flex === 'auto') classes.push('flex-auto')
    else if (props.flex === 'initial') classes.push('flex-initial')
    else if (props.flex === 'none') classes.push('flex-none')
    else if (props.flex === '1') classes.push('flex-1')
  }

  if (props.flexGrow) classes.push('flex-grow')
  if (props.flexShrink) classes.push('flex-shrink')

  // Self alignment
  if (props.justifySelf) {
    const justifyMap = {
      auto: 'justify-self-auto',
      start: 'justify-self-start',
      center: 'justify-self-center',
      end: 'justify-self-end',
      stretch: 'justify-self-stretch'
    }
    classes.push(justifyMap[props.justifySelf])
  }

  if (props.alignSelf) {
    const alignMap = {
      auto: 'self-auto',
      start: 'self-start',
      center: 'self-center',
      end: 'self-end',
      stretch: 'self-stretch',
      baseline: 'self-baseline'
    }
    classes.push(alignMap[props.alignSelf])
  }

  return classes.join(' ')
})
</script>