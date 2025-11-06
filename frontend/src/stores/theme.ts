import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { useLocalStorage, usePreferredDark } from '@vueuse/core'

export type ThemeMode = 'light' | 'dark' | 'system'

export const useThemeStore = defineStore('theme', () => {
  // Get system preference
  const prefersDark = usePreferredDark()

  // Persisted theme setting
  const themeMode = useLocalStorage<ThemeMode>('moby-theme', 'system')

  // Computed actual theme based on mode and system preference
  const isDark = computed(() => {
    if (themeMode.value === 'system') {
      return prefersDark.value
    }
    return themeMode.value === 'dark'
  })

  // Theme colors
  const colors = computed(() => ({
    primary: {
      50: '#f0f9ff',
      100: '#e0f2fe',
      200: '#bae6fd',
      300: '#7dd3fc',
      400: '#38bdf8',
      500: '#0ea5e9',
      600: '#0284c7',
      700: '#0369a1',
      800: '#075985',
      900: '#0c4a6e',
      950: '#082f49',
    },
    whale: {
      50: '#f8fafc',
      100: '#f1f5f9',
      200: '#e2e8f0',
      300: '#cbd5e1',
      400: '#94a3b8',
      500: '#64748b',
      600: '#475569',
      700: '#334155',
      800: '#1e293b',
      900: '#0f172a',
      950: '#020617',
    },
    success: {
      50: '#f0fdf4',
      500: '#22c55e',
      600: '#16a34a',
      700: '#15803d',
    },
    warning: {
      50: '#fffbeb',
      500: '#f59e0b',
      600: '#d97706',
      700: '#b45309',
    },
    error: {
      50: '#fef2f2',
      500: '#ef4444',
      600: '#dc2626',
      700: '#b91c1c',
    },
  }))

  // Current theme configuration
  const currentTheme = computed(() => ({
    mode: themeMode.value,
    isDark: isDark.value,
    colors: colors.value,
    name: isDark.value ? 'moby-dark' : 'moby-light',
  }))

  // Actions
  function setTheme(mode: ThemeMode) {
    themeMode.value = mode
  }

  function toggleTheme() {
    if (themeMode.value === 'system') {
      setTheme(prefersDark.value ? 'light' : 'dark')
    } else {
      setTheme(themeMode.value === 'light' ? 'dark' : 'light')
    }
  }

  // Apply theme to document
  function applyTheme() {
    const html = document.documentElement

    if (isDark.value) {
      html.classList.add('dark')
      html.classList.remove('light')
    } else {
      html.classList.add('light')
      html.classList.remove('dark')
    }

    // Set CSS custom properties
    const root = html.style
    const theme = isDark.value ? 'dark' : 'light'

    // Apply color variables
    Object.entries(colors.value).forEach(([colorName, shades]) => {
      Object.entries(shades).forEach(([shade, value]) => {
        root.setProperty(`--color-${colorName}-${shade}`, value)
      })
    })

    // Set skeleton theme
    root.setProperty('--theme', theme)
  }

  // Watch for theme changes and apply them
  watch([isDark], applyTheme, { immediate: true })

  // System theme change detection
  watch(prefersDark, () => {
    if (themeMode.value === 'system') {
      applyTheme()
    }
  })

  // Theme presets for different trading environments
  const presets = {
    professional: {
      name: 'Professional',
      description: 'Clean, minimal design for serious trading',
      colors: {
        primary: '#0ea5e9',
        accent: '#3b82f6',
        background: isDark.value ? '#0f172a' : '#ffffff',
      },
    },
    whale: {
      name: 'Whale Mode',
      description: 'Premium dark theme for whale traders',
      colors: {
        primary: '#0ea5e9',
        accent: '#8b5cf6',
        background: '#020617',
      },
    },
    terminal: {
      name: 'Terminal',
      description: 'High-contrast terminal-style interface',
      colors: {
        primary: '#00ff00',
        accent: '#ffff00',
        background: '#000000',
      },
    },
  }

  function applyPreset(presetName: keyof typeof presets) {
    const preset = presets[presetName]
    if (!preset) return

    const root = document.documentElement.style
    Object.entries(preset.colors).forEach(([key, value]) => {
      root.setProperty(`--color-preset-${key}`, value)
    })
  }

  // Accessibility features
  const reducedMotion = useLocalStorage('moby-reduced-motion', false)
  const highContrast = useLocalStorage('moby-high-contrast', false)
  const fontSize = useLocalStorage('moby-font-size', 'normal')

  function setReducedMotion(enabled: boolean) {
    reducedMotion.value = enabled
    document.documentElement.classList.toggle('reduce-motion', enabled)
  }

  function setHighContrast(enabled: boolean) {
    highContrast.value = enabled
    document.documentElement.classList.toggle('high-contrast', enabled)
  }

  function setFontSize(size: 'small' | 'normal' | 'large') {
    fontSize.value = size
    document.documentElement.classList.remove('font-small', 'font-normal', 'font-large')
    document.documentElement.classList.add(`font-${size}`)
  }

  // Initialize accessibility settings
  watch([reducedMotion, highContrast, fontSize], ([motion, contrast, size]) => {
    setReducedMotion(motion)
    setHighContrast(contrast)
    setFontSize(size as 'small' | 'normal' | 'large')
  }, { immediate: true })

  return {
    // State
    themeMode,
    isDark,
    colors,
    currentTheme,
    presets,

    // Accessibility
    reducedMotion,
    highContrast,
    fontSize,

    // Actions
    setTheme,
    toggleTheme,
    applyTheme,
    applyPreset,
    setReducedMotion,
    setHighContrast,
    setFontSize,
  }
})