import { ref, computed, onMounted, onUnmounted } from 'vue'

export type Breakpoint = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl'

export interface BreakpointConfig {
  xs: number
  sm: number
  md: number
  lg: number
  xl: number
  '2xl': number
}

// TailwindCSS default breakpoints
const breakpoints: BreakpointConfig = {
  xs: 0,
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536
}

export function useBreakpoints() {
  const windowWidth = ref(0)

  // Computed breakpoint states
  const isXs = computed(() => windowWidth.value >= breakpoints.xs && windowWidth.value < breakpoints.sm)
  const isSm = computed(() => windowWidth.value >= breakpoints.sm && windowWidth.value < breakpoints.md)
  const isMd = computed(() => windowWidth.value >= breakpoints.md && windowWidth.value < breakpoints.lg)
  const isLg = computed(() => windowWidth.value >= breakpoints.lg && windowWidth.value < breakpoints.xl)
  const isXl = computed(() => windowWidth.value >= breakpoints.xl && windowWidth.value < breakpoints['2xl'])
  const is2xl = computed(() => windowWidth.value >= breakpoints['2xl'])

  // Utility computed properties
  const isMobile = computed(() => windowWidth.value < breakpoints.md)
  const isTablet = computed(() => windowWidth.value >= breakpoints.md && windowWidth.value < breakpoints.lg)
  const isDesktop = computed(() => windowWidth.value >= breakpoints.lg)
  const isSmallScreen = computed(() => windowWidth.value < breakpoints.lg)
  const isLargeScreen = computed(() => windowWidth.value >= breakpoints.xl)

  // Current breakpoint
  const currentBreakpoint = computed<Breakpoint>(() => {
    if (is2xl.value) return '2xl'
    if (isXl.value) return 'xl'
    if (isLg.value) return 'lg'
    if (isMd.value) return 'md'
    if (isSm.value) return 'sm'
    return 'xs'
  })

  // Breakpoint comparison utilities
  const greaterOrEqual = (breakpoint: Breakpoint) => {
    return computed(() => windowWidth.value >= breakpoints[breakpoint])
  }

  const smallerThan = (breakpoint: Breakpoint) => {
    return computed(() => windowWidth.value < breakpoints[breakpoint])
  }

  const between = (min: Breakpoint, max: Breakpoint) => {
    return computed(() =>
      windowWidth.value >= breakpoints[min] && windowWidth.value < breakpoints[max]
    )
  }

  // Update window width
  function updateWindowWidth() {
    windowWidth.value = window.innerWidth
  }

  // Responsive value selector
  function useResponsiveValue<T>(values: Partial<Record<Breakpoint, T>>, fallback: T): computed<T> {
    return computed(() => {
      // Check from largest to smallest breakpoint
      const orderedBreakpoints: Breakpoint[] = ['2xl', 'xl', 'lg', 'md', 'sm', 'xs']

      for (const bp of orderedBreakpoints) {
        if (windowWidth.value >= breakpoints[bp] && values[bp] !== undefined) {
          return values[bp]!
        }
      }

      return fallback
    })
  }

  // Responsive class selector
  function useResponsiveClass(classes: Partial<Record<Breakpoint, string>>, fallback = ''): computed<string> {
    return useResponsiveValue(classes, fallback)
  }

  // Device detection
  const deviceType = computed(() => {
    if (isMobile.value) return 'mobile'
    if (isTablet.value) return 'tablet'
    return 'desktop'
  })

  // Orientation detection
  const isLandscape = computed(() => {
    if (typeof window === 'undefined') return false
    return window.innerHeight < window.innerWidth
  })

  const isPortrait = computed(() => !isLandscape.value)

  // Touch device detection
  const isTouchDevice = computed(() => {
    if (typeof window === 'undefined') return false
    return 'ontouchstart' in window || navigator.maxTouchPoints > 0
  })

  // Lifecycle
  onMounted(() => {
    updateWindowWidth()
    window.addEventListener('resize', updateWindowWidth)
  })

  onUnmounted(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('resize', updateWindowWidth)
    }
  })

  return {
    // Window dimensions
    windowWidth,

    // Breakpoint states
    isXs,
    isSm,
    isMd,
    isLg,
    isXl,
    is2xl,

    // Device categories
    isMobile,
    isTablet,
    isDesktop,
    isSmallScreen,
    isLargeScreen,

    // Current breakpoint
    currentBreakpoint,

    // Utilities
    greaterOrEqual,
    smallerThan,
    between,
    useResponsiveValue,
    useResponsiveClass,

    // Device info
    deviceType,
    isLandscape,
    isPortrait,
    isTouchDevice,

    // Raw breakpoints for custom logic
    breakpoints
  }
}