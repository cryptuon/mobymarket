// Global type definitions for Moby Market

import type { Ref } from 'vue'
import type { RouteLocationNormalized } from 'vue-router'

// ===== WALLET & WEB3 TYPES =====

export interface WalletInfo {
  address: string
  chainId: number
  isConnected: boolean
  balance?: string
  walletType?: string
  ensName?: string
}

export interface WalletProvider {
  name: string
  isInstalled: boolean
  icon: string
  connector: any
}

export interface ChainInfo {
  id: number
  name: string
  symbol: string
  rpcUrl: string
  blockExplorer: string
  isTestnet: boolean
}

export interface TokenInfo {
  address: string
  symbol: string
  name: string
  decimals: number
  logoURI?: string
  chainId: number
}

// ===== TRADING TYPES =====

export interface TradingPair {
  tokenIn: TokenInfo
  tokenOut: TokenInfo
  symbol: string // e.g., "ETH/USDC"
}

export interface Quote {
  id: string
  tokenIn: string
  tokenOut: string
  amountIn: string
  amountOut: string
  priceImpact: number
  gasEstimate: string
  route: QuoteRoute[]
  validUntil: number
  timestamp: number
}

export interface QuoteRoute {
  dex: string
  percentage: number
  amountIn: string
  amountOut: string
  priceImpact: number
  gasEstimate: string
}

export interface TradeRequest {
  tokenIn: string
  tokenOut: string
  amountIn: string
  slippageTolerance: number
  deadline?: number
  recipient?: string
  usePrivacy?: boolean
  mevProtection?: boolean
}

export interface TradeResult {
  id: string
  hash: string
  status: 'pending' | 'confirmed' | 'failed'
  amountIn: string
  amountOut: string
  actualSlippage: number
  gasUsed: string
  gasPrice: string
  timestamp: number
  route: ExecutedRoute[]
}

export interface ExecutedRoute {
  dex: string
  amountIn: string
  amountOut: string
  gasUsed: string
}

// ===== PORTFOLIO TYPES =====

export interface Portfolio {
  totalValue: string // USD value
  assets: PortfolioAsset[]
  chains: ChainPortfolio[]
  performance: PortfolioPerformance
}

export interface PortfolioAsset {
  token: TokenInfo
  balance: string
  value: string // USD value
  percentage: number
  priceChange24h: number
  chains: Array<{
    chainId: number
    balance: string
    value: string
  }>
}

export interface ChainPortfolio {
  chainId: number
  chainName: string
  totalValue: string
  assetCount: number
  assets: PortfolioAsset[]
}

export interface PortfolioPerformance {
  totalReturn: number
  totalReturnUSD: string
  return24h: number
  return7d: number
  return30d: number
  return1y: number
  winRate: number
  totalTrades: number
  profitableTrades: number
}

// ===== WHALE TRACKING TYPES =====

export interface WhaleActivity {
  id: string
  address: string // anonymized
  type: 'buy' | 'sell' | 'transfer'
  token: string
  amount: string
  usdValue: number
  timestamp: string
  txHash: string
  blockNumber: number
  tokenIn?: string
  tokenOut?: string
  dex?: string
  chain?: string
  priceImpact?: number
  strategy?: string
}

export interface WhaleAlert {
  id: string
  type: 'large_trade' | 'accumulation' | 'distribution' | 'migration'
  whale: string // anonymized ID
  amount: string
  token: string
  description: string
  timestamp: number
  severity: 'low' | 'medium' | 'high' | 'critical'
}

export interface WhaleMigration {
  fromChain: string
  toChain: string
  amount: string
  whaleCount: number
  timeframe: string
  trend: 'increasing' | 'decreasing' | 'stable'
}

// ===== ANALYTICS TYPES =====

export interface MarketData {
  totalVolume24h: string
  totalLiquidity: string
  whaleActivityScore: number
  marketVolatility: number
  sentiment: 'bullish' | 'bearish' | 'neutral'
  topPairs: Array<{
    pair: string
    volume24h: string
    priceChange24h: number
  }>
}

export interface PriceData {
  token: string
  price: number
  priceChange24h: number
  volume24h: string
  marketCap?: string
  timestamp: number
}

export interface TokenPrice {
  symbol: string
  name: string
  price: number
  change24h: number
  volume24h: number
  marketCap: number
  lastUpdated: string
}

export interface ChartData {
  timestamp: number
  price: number
  volume: number
  high: number
  low: number
  open: number
  close: number
}

// ===== UI COMPONENT TYPES =====

export type NotificationType = 'success' | 'error' | 'warning' | 'info'
export type NotificationCategory = 'whale_activity' | 'trading' | 'price_alert' | 'system'

export interface Notification {
  id: string
  type: NotificationType
  category: NotificationCategory
  title: string
  message: string
  timestamp: string
  isRead: boolean
  actionUrl?: string
  data?: Record<string, any>
  duration?: number
  actions?: NotificationAction[]
}

export interface NotificationAction {
  label: string
  action: () => void
  style?: 'primary' | 'secondary' | 'danger'
}

export interface TableColumn<T = any> {
  key: keyof T | string
  label: string
  sortable?: boolean
  width?: string
  align?: 'left' | 'center' | 'right'
  formatter?: (value: any, row: T) => string
  component?: any
}

export interface SelectOption {
  value: string | number
  label: string
  disabled?: boolean
  icon?: string
  description?: string
}

// ===== STORE TYPES =====

export interface AppState {
  isLoading: boolean
  loadingMessage?: string
  notifications: Notification[]
  theme: 'light' | 'dark' | 'system'
  sidebar: {
    isOpen: boolean
    isPinned: boolean
  }
  modal: {
    isOpen: boolean
    component?: any
    props?: Record<string, any>
  }
}

export interface WalletState {
  wallet: WalletInfo | null
  isConnecting: boolean
  supportedChains: ChainInfo[]
  currentChain: ChainInfo | null
  recentTransactions: TradeResult[]
}

export interface TradingState {
  quotes: Record<string, Quote>
  activeTrades: TradeResult[]
  tradingHistory: TradeResult[]
  favorites: TradingPair[]
  defaultSlippage: number
  defaultDeadline: number
  preferences: {
    usePrivacy: boolean
    mevProtection: boolean
    autoRefresh: boolean
    soundEnabled: boolean
  }
}

export interface PortfolioState {
  portfolio: Portfolio | null
  isLoading: boolean
  lastUpdated?: number
  watchlist: string[]
  priceAlerts: PriceAlert[]
}

export interface PriceAlert {
  id: string
  token: string
  type: 'above' | 'below'
  price: number
  isActive: boolean
  createdAt: number
}

// ===== API TYPES =====

export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
  timestamp: number
  requestId?: string
}

export interface PaginatedResponse<T = any> extends ApiResponse<T[]> {
  pagination: {
    page: number
    limit: number
    total: number
    hasNext: boolean
    hasPrev: boolean
  }
}

export interface WebSocketMessage<T = any> {
  type: string
  data: T
  timestamp: number
  id?: string
}

// ===== ROUTE TYPES =====

export interface RouteMetaData {
  title?: string
  description?: string
  requiresAuth?: boolean
  requiresWallet?: boolean
  transition?: string
  layout?: string
  breadcrumbs?: Breadcrumb[]
}

export interface Breadcrumb {
  label: string
  to?: string
  icon?: string
}

// ===== COMPOSABLE RETURN TYPES =====

export interface UseAsyncStateReturn<T> {
  state: Ref<T | null>
  isLoading: Ref<boolean>
  error: Ref<Error | null>
  execute: () => Promise<T>
  refresh: () => Promise<T>
}

export interface UseWebSocketReturn {
  isConnected: Ref<boolean>
  isConnecting: Ref<boolean>
  send: (message: any) => void
  close: () => void
  reconnect: () => void
  lastMessage: Ref<any>
}

export interface UseQueryReturn<T> {
  data: Ref<T | undefined>
  isLoading: Ref<boolean>
  isError: Ref<boolean>
  error: Ref<Error | null>
  refetch: () => Promise<void>
}

// ===== UTILITY TYPES =====

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P]
}

export type RequiredKeys<T, K extends keyof T> = T & Required<Pick<T, K>>

export type OptionalKeys<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>

export type Timestamp = number

export type Address = string

export type TokenAmount = string

export type PercentageChange = number

// ===== FORM TYPES =====

export interface FormField {
  name: string
  label: string
  type: 'text' | 'number' | 'email' | 'password' | 'select' | 'textarea'
  placeholder?: string
  required?: boolean
  disabled?: boolean
  options?: SelectOption[]
  validation?: {
    min?: number
    max?: number
    pattern?: RegExp
    custom?: (value: any) => string | null
  }
}

export interface FormState {
  values: Record<string, any>
  errors: Record<string, string>
  touched: Record<string, boolean>
  isValid: boolean
  isDirty: boolean
  isSubmitting: boolean
}

// ===== THEME TYPES =====

export interface ThemeColors {
  primary: string
  secondary: string
  success: string
  warning: string
  error: string
  surface: string
  background: string
  text: string
}

export interface ThemeConfig {
  name: string
  colors: {
    light: ThemeColors
    dark: ThemeColors
  }
  borderRadius: string
  fontFamily: {
    sans: string
    mono: string
  }
}

// ===== ERROR TYPES =====

export interface ErrorInfo {
  code: string
  message: string
  details?: any
  timestamp: number
  userMessage?: string
  recoverable?: boolean
}

export class AppError extends Error {
  code: string
  details?: any
  userMessage?: string
  recoverable: boolean

  constructor(message: string, code: string, details?: any, userMessage?: string, recoverable = true) {
    super(message)
    this.name = 'AppError'
    this.code = code
    this.details = details
    this.userMessage = userMessage
    this.recoverable = recoverable
  }
}

// Re-export commonly used Vue types
export type { Ref, ComputedRef, WatchStopHandle } from 'vue'
export type { RouteLocationNormalized, Router } from 'vue-router'