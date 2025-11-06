import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/HomeView.vue'),
    meta: {
      title: 'Moby Market - Premium Whale Trading Platform',
      description: 'Professional trading platform for whale traders with privacy, cross-chain, and MEV protection',
      transition: 'fade',
    },
  },
  {
    path: '/trade',
    name: 'Trade',
    component: () => import('@/views/trading/TradeView.vue'),
    meta: {
      title: 'Trade - Moby Market',
      description: 'Execute whale trades with optimal routing and privacy protection',
      requiresWallet: true,
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Trade' },
      ],
    },
  },
  {
    path: '/portfolio',
    name: 'Portfolio',
    component: () => import('@/views/portfolio/PortfolioView.vue'),
    meta: {
      title: 'Portfolio - Moby Market',
      description: 'Track your whale portfolio performance and analytics',
      requiresWallet: true,
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Portfolio' },
      ],
    },
    children: [
      {
        path: '',
        name: 'PortfolioOverview',
        component: () => import('@/views/portfolio/PortfolioOverview.vue'),
      },
      {
        path: 'positions',
        name: 'Positions',
        component: () => import('@/views/portfolio/PositionsView.vue'),
        meta: {
          title: 'Positions - Portfolio',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Portfolio', to: '/portfolio' },
            { label: 'Positions' },
          ],
        },
      },
      {
        path: 'history',
        name: 'TradingHistory',
        component: () => import('@/views/portfolio/TradingHistoryView.vue'),
        meta: {
          title: 'Trading History - Portfolio',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Portfolio', to: '/portfolio' },
            { label: 'History' },
          ],
        },
      },
      {
        path: 'analytics',
        name: 'PortfolioAnalytics',
        component: () => import('@/views/portfolio/AnalyticsView.vue'),
        meta: {
          title: 'Analytics - Portfolio',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Portfolio', to: '/portfolio' },
            { label: 'Analytics' },
          ],
        },
      },
    ],
  },
  {
    path: '/whale-intelligence',
    name: 'WhaleIntelligence',
    component: () => import('@/views/whale/WhaleIntelligenceView.vue'),
    meta: {
      title: 'Whale Intelligence - Moby Market',
      description: 'Real-time whale activity tracking and market intelligence',
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Whale Intelligence' },
      ],
    },
    children: [
      {
        path: '',
        name: 'WhaleOverview',
        component: () => import('@/views/whale/WhaleOverview.vue'),
      },
      {
        path: 'activity',
        name: 'WhaleActivity',
        component: () => import('@/views/whale/WhaleActivityView.vue'),
        meta: {
          title: 'Whale Activity - Intelligence',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Whale Intelligence', to: '/whale-intelligence' },
            { label: 'Activity' },
          ],
        },
      },
      {
        path: 'migration',
        name: 'WhaleMigration',
        component: () => import('@/views/whale/MigrationView.vue'),
        meta: {
          title: 'Migration Patterns - Intelligence',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Whale Intelligence', to: '/whale-intelligence' },
            { label: 'Migration' },
          ],
        },
      },
      {
        path: 'alerts',
        name: 'WhaleAlerts',
        component: () => import('@/views/whale/AlertsView.vue'),
        meta: {
          title: 'Alerts - Intelligence',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Whale Intelligence', to: '/whale-intelligence' },
            { label: 'Alerts' },
          ],
        },
      },
    ],
  },
  {
    path: '/yield',
    name: 'Yield',
    component: () => import('@/views/yield/YieldView.vue'),
    meta: {
      title: 'Yield Optimization - Moby Market',
      description: 'Maximize returns with automated yield farming and optimization',
      requiresWallet: true,
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Yield' },
      ],
    },
    children: [
      {
        path: '',
        name: 'YieldOverview',
        component: () => import('@/views/yield/YieldOverview.vue'),
      },
      {
        path: 'opportunities',
        name: 'YieldOpportunities',
        component: () => import('@/views/yield/OpportunitiesView.vue'),
        meta: {
          title: 'Opportunities - Yield',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Yield', to: '/yield' },
            { label: 'Opportunities' },
          ],
        },
      },
      {
        path: 'positions',
        name: 'YieldPositions',
        component: () => import('@/views/yield/YieldPositionsView.vue'),
        meta: {
          title: 'Positions - Yield',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Yield', to: '/yield' },
            { label: 'Positions' },
          ],
        },
      },
    ],
  },
  {
    path: '/governance',
    name: 'Governance',
    component: () => import('@/views/governance/GovernanceView.vue'),
    meta: {
      title: 'Governance - Moby Market',
      description: 'Participate in protocol governance and earn rewards',
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Governance' },
      ],
    },
    children: [
      {
        path: '',
        name: 'GovernanceOverview',
        component: () => import('@/views/governance/GovernanceOverview.vue'),
      },
      {
        path: 'proposals',
        name: 'Proposals',
        component: () => import('@/views/governance/ProposalsView.vue'),
        meta: {
          title: 'Proposals - Governance',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Governance', to: '/governance' },
            { label: 'Proposals' },
          ],
        },
      },
      {
        path: 'voting',
        name: 'Voting',
        component: () => import('@/views/governance/VotingView.vue'),
        meta: {
          title: 'Voting - Governance',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Governance', to: '/governance' },
            { label: 'Voting' },
          ],
        },
      },
    ],
  },
  {
    path: '/analytics',
    name: 'Analytics',
    component: () => import('@/views/analytics/AnalyticsView.vue'),
    meta: {
      title: 'Market Analytics - Moby Market',
      description: 'Advanced market analytics and trading insights',
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Analytics' },
      ],
    },
    children: [
      {
        path: '',
        name: 'AnalyticsOverview',
        component: () => import('@/views/analytics/AnalyticsOverview.vue'),
      },
      {
        path: 'market',
        name: 'MarketAnalytics',
        component: () => import('@/views/analytics/MarketAnalyticsView.vue'),
        meta: {
          title: 'Market Analytics',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Analytics', to: '/analytics' },
            { label: 'Market' },
          ],
        },
      },
      {
        path: 'performance',
        name: 'PerformanceAnalytics',
        component: () => import('@/views/analytics/PerformanceView.vue'),
        meta: {
          title: 'Performance Analytics',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Analytics', to: '/analytics' },
            { label: 'Performance' },
          ],
        },
      },
    ],
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/settings/SettingsView.vue'),
    meta: {
      title: 'Settings - Moby Market',
      description: 'Configure your trading preferences and account settings',
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Settings' },
      ],
    },
    children: [
      {
        path: '',
        name: 'GeneralSettings',
        component: () => import('@/views/settings/GeneralSettings.vue'),
      },
      {
        path: 'trading',
        name: 'TradingSettings',
        component: () => import('@/views/settings/TradingSettings.vue'),
        meta: {
          title: 'Trading Settings',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Settings', to: '/settings' },
            { label: 'Trading' },
          ],
        },
      },
      {
        path: 'privacy',
        name: 'PrivacySettings',
        component: () => import('@/views/settings/PrivacySettings.vue'),
        meta: {
          title: 'Privacy Settings',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Settings', to: '/settings' },
            { label: 'Privacy' },
          ],
        },
      },
      {
        path: 'notifications',
        name: 'NotificationSettings',
        component: () => import('@/views/settings/NotificationSettings.vue'),
        meta: {
          title: 'Notification Settings',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Settings', to: '/settings' },
            { label: 'Notifications' },
          ],
        },
      },
      {
        path: 'api',
        name: 'APISettings',
        component: () => import('@/views/settings/APISettings.vue'),
        meta: {
          title: 'API Settings',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Settings', to: '/settings' },
            { label: 'API' },
          ],
        },
      },
    ],
  },
  {
    path: '/help',
    name: 'Help',
    component: () => import('@/views/help/HelpView.vue'),
    meta: {
      title: 'Help & Documentation - Moby Market',
      description: 'Learn how to use Moby Market for optimal whale trading',
      transition: 'slide-left',
      breadcrumbs: [
        { label: 'Home', to: '/' },
        { label: 'Help' },
      ],
    },
    children: [
      {
        path: '',
        name: 'HelpOverview',
        component: () => import('@/views/help/HelpOverview.vue'),
      },
      {
        path: 'getting-started',
        name: 'GettingStarted',
        component: () => import('@/views/help/GettingStarted.vue'),
        meta: {
          title: 'Getting Started',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Help', to: '/help' },
            { label: 'Getting Started' },
          ],
        },
      },
      {
        path: 'trading-guide',
        name: 'TradingGuide',
        component: () => import('@/views/help/TradingGuide.vue'),
        meta: {
          title: 'Trading Guide',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Help', to: '/help' },
            { label: 'Trading Guide' },
          ],
        },
      },
      {
        path: 'api-docs',
        name: 'APIDocs',
        component: () => import('@/views/help/APIDocsView.vue'),
        meta: {
          title: 'API Documentation',
          breadcrumbs: [
            { label: 'Home', to: '/' },
            { label: 'Help', to: '/help' },
            { label: 'API Docs' },
          ],
        },
      },
    ],
  },
  // Error pages
  {
    path: '/404',
    name: 'NotFound',
    component: () => import('@/views/error/NotFoundView.vue'),
    meta: {
      title: 'Page Not Found - Moby Market',
      layout: 'minimal',
    },
  },
  {
    path: '/500',
    name: 'ServerError',
    component: () => import('@/views/error/ServerErrorView.vue'),
    meta: {
      title: 'Server Error - Moby Market',
      layout: 'minimal',
    },
  },
  {
    path: '/maintenance',
    name: 'Maintenance',
    component: () => import('@/views/error/MaintenanceView.vue'),
    meta: {
      title: 'Maintenance - Moby Market',
      layout: 'minimal',
    },
  },
  // Catch-all route
  {
    path: '/:pathMatch(.*)*',
    redirect: '/404',
  },
]

export default routes