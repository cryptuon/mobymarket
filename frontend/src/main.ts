import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import { VueQueryPlugin } from '@tanstack/vue-query'

import App from './App.vue'
import routes from './router/routes'

// Styles
import './assets/styles/main.css'
import '@skeletonlabs/skeleton/themes/theme-moby-dark.css'
import '@skeletonlabs/skeleton/styles/skeleton.css'
import './assets/styles/app.postcss'

// Initialize stores
const pinia = createPinia()

// Initialize router
const router = createRouter({
  history: createWebHistory(),
  routes,
})

// Create app
const app = createApp(App)

// Install plugins
app.use(pinia)
app.use(router)
app.use(VueQueryPlugin, {
  queryClientConfig: {
    defaultOptions: {
      queries: {
        staleTime: 30_000, // 30 seconds
        cacheTime: 5 * 60 * 1000, // 5 minutes
        retry: 2,
        refetchOnWindowFocus: false,
      },
    },
  },
})

// Global error handler
app.config.errorHandler = (error, instance, info) => {
  console.error('Global error:', error, info)
  // TODO: Send to error tracking service
}

// Mount app
app.mount('#app')