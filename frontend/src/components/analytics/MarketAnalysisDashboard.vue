<template>
  <div class="space-y-6">
    <!-- Market Overview Cards -->
    <Grid :cols="{ xs: 2, lg: 4 }" gap="6">
      <GridItem>
        <MetricCard
          title="Market Cap"
          :value="formatCurrency(marketData.totalMarketCap)"
          :change="marketData.marketCapChange"
          icon="GlobeAltIcon"
          color="blue"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="24h Volume"
          :value="formatCurrency(marketData.volume24h)"
          :change="marketData.volumeChange"
          icon="ArrowsRightLeftIcon"
          color="green"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="BTC Dominance"
          :value="`${marketData.btcDominance}%`"
          :change="marketData.dominanceChange"
          icon="ChartPieIcon"
          color="orange"
        />
      </GridItem>
      <GridItem>
        <MetricCard
          title="Fear & Greed"
          :value="marketData.fearGreedIndex"
          :change="marketData.fearGreedChange"
          icon="FaceSmileIcon"
          :color="getFearGreedColor(marketData.fearGreedIndex)"
        />
      </GridItem>
    </Grid>

    <!-- Market Charts -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <MarketOverviewChart
          :data="marketChartData"
          :time-range="timeRange"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <SectorPerformanceChart
          :data="sectorData"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Market Sentiment & Indicators -->
    <Grid :cols="{ xs: 1, lg: 3 }" gap="6">
      <GridItem>
        <MarketSentimentCard
          :data="sentimentData"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <TechnicalIndicatorsCard
          :data="technicalIndicators"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <OnChainMetricsCard
          :data="onChainMetrics"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Market Movers & News -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <TopMoversCard
          :gainers="topGainers"
          :losers="topLosers"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <MarketNewsCard
          :news="marketNews"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- DeFi & NFT Metrics -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <DeFiMetricsCard
          :data="defiMetrics"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <NFTMetricsCard
          :data="nftMetrics"
          :loading="loading"
        />
      </GridItem>
    </Grid>

    <!-- Correlation & Volatility Analysis -->
    <Grid :cols="{ xs: 1, lg: 2 }" gap="6">
      <GridItem>
        <CorrelationHeatmap
          :data="correlationMatrix"
          :loading="loading"
        />
      </GridItem>
      <GridItem>
        <VolatilityChart
          :data="volatilityData"
          :time-range="timeRange"
          :loading="loading"
        />
      </GridItem>
    </Grid>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Grid from '@components/ui/Grid.vue'
import GridItem from '@components/ui/GridItem.vue'
import MetricCard from '@components/dashboard/MetricCard.vue'
import MarketOverviewChart from './charts/MarketOverviewChart.vue'
import SectorPerformanceChart from './charts/SectorPerformanceChart.vue'
import MarketSentimentCard from './cards/MarketSentimentCard.vue'
import TechnicalIndicatorsCard from './cards/TechnicalIndicatorsCard.vue'
import OnChainMetricsCard from './cards/OnChainMetricsCard.vue'
import TopMoversCard from './cards/TopMoversCard.vue'
import MarketNewsCard from './cards/MarketNewsCard.vue'
import DeFiMetricsCard from './cards/DeFiMetricsCard.vue'
import NFTMetricsCard from './cards/NFTMetricsCard.vue'
import CorrelationHeatmap from './charts/CorrelationHeatmap.vue'
import VolatilityChart from './charts/VolatilityChart.vue'

interface Props {
  data: any
  timeRange: string
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

// Market data
const marketData = ref({
  totalMarketCap: 2450000000000,
  marketCapChange: 3.2,
  volume24h: 125000000000,
  volumeChange: -8.5,
  btcDominance: 52.3,
  dominanceChange: 0.8,
  fearGreedIndex: 68,
  fearGreedChange: 5
})

// Market chart data
const marketChartData = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    return {
      timestamp: date.toISOString(),
      btc: 65000 + Math.random() * 10000,
      eth: 3200 + Math.random() * 800,
      totalMarketCap: 2400000000000 + Math.random() * 200000000000,
      volume: 100000000000 + Math.random() * 50000000000
    }
  })
})

// Sector performance data
const sectorData = ref([
  { sector: 'Layer 1', performance: 12.5, volume: 45000000000, marketCap: 1200000000000 },
  { sector: 'DeFi', performance: -3.8, volume: 28000000000, marketCap: 180000000000 },
  { sector: 'GameFi', performance: 25.3, volume: 5200000000, marketCap: 35000000000 },
  { sector: 'Metaverse', performance: -12.1, volume: 3800000000, marketCap: 28000000000 },
  { sector: 'NFTs', performance: 8.7, volume: 2100000000, marketCap: 15000000000 },
  { sector: 'Infrastructure', performance: 5.2, volume: 8500000000, marketCap: 65000000000 }
])

// Sentiment data
const sentimentData = ref({
  fearGreedIndex: 68,
  socialSentiment: 72,
  newssentiment: 65,
  tradingSentiment: 78,
  indicators: [
    { name: 'RSI', value: 45, status: 'neutral' },
    { name: 'MACD', value: 0.8, status: 'bullish' },
    { name: 'Bollinger Bands', value: 0.3, status: 'neutral' },
    { name: 'Stoch RSI', value: 25, status: 'oversold' }
  ]
})

// Technical indicators
const technicalIndicators = ref({
  btc: {
    rsi: 45.2,
    macd: 850.5,
    bb_upper: 68500,
    bb_lower: 63200,
    sma_20: 65800,
    sma_50: 64200,
    volume_sma: 28500000000
  },
  eth: {
    rsi: 52.8,
    macd: 125.3,
    bb_upper: 3450,
    bb_lower: 3150,
    sma_20: 3280,
    sma_50: 3180,
    volume_sma: 15200000000
  }
})

// On-chain metrics
const onChainMetrics = ref({
  activeAddresses: 987654,
  transactions: 1250000,
  hashRate: '245.5 EH/s',
  difficulty: '35.6T',
  gasPrice: 25.8,
  tvl: 85200000000,
  stakingRatio: 0.157,
  whaleActivity: 847
})

// Top movers
const topGainers = ref([
  { symbol: 'RNDR', name: 'Render Token', price: 8.45, change: 35.7 },
  { symbol: 'FET', name: 'Fetch.ai', price: 2.18, change: 28.3 },
  { symbol: 'OCEAN', name: 'Ocean Protocol', price: 0.87, change: 24.9 },
  { symbol: 'AGIX', name: 'SingularityNET', price: 0.65, change: 22.1 },
  { symbol: 'TAO', name: 'Bittensor', price: 425.80, change: 18.5 }
])

const topLosers = ref([
  { symbol: 'LUNA', name: 'Terra Classic', price: 0.000085, change: -18.9 },
  { symbol: 'LUNC', name: 'Terra Luna Classic', price: 0.000072, change: -15.2 },
  { symbol: 'FTT', name: 'FTX Token', price: 1.85, change: -12.8 },
  { symbol: 'CEL', name: 'Celsius', price: 0.32, change: -11.5 },
  { symbol: 'VOYG', name: 'Voyager Token', price: 0.045, change: -9.7 }
])

// Market news
const marketNews = ref([
  {
    id: '1',
    title: 'Bitcoin ETF Inflows Reach Record High',
    summary: 'Institutional adoption continues to drive BTC demand with over $2B in weekly inflows.',
    source: 'CoinDesk',
    timestamp: new Date(Date.now() - 1800000).toISOString(),
    sentiment: 'bullish'
  },
  {
    id: '2',
    title: 'Ethereum Layer 2 Solutions See 300% Growth',
    summary: 'Arbitrum and Optimism lead the scaling revolution with record TVL and transaction volumes.',
    source: 'The Block',
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    sentiment: 'bullish'
  },
  {
    id: '3',
    title: 'Regulatory Clarity Expected This Quarter',
    summary: 'SEC signals potential framework for crypto regulation, markets respond positively.',
    source: 'Bloomberg',
    timestamp: new Date(Date.now() - 7200000).toISOString(),
    sentiment: 'neutral'
  }
])

// DeFi metrics
const defiMetrics = ref({
  totalTVL: 85200000000,
  tvlChange: 12.5,
  topProtocols: [
    { name: 'Uniswap', tvl: 8500000000, change: 5.2 },
    { name: 'Aave', tvl: 7200000000, change: 8.7 },
    { name: 'Curve', tvl: 4800000000, change: -2.1 },
    { name: 'MakerDAO', tvl: 4200000000, change: 3.8 }
  ],
  yields: {
    avgAPY: 8.5,
    topYield: 145.2,
    stablecoinYield: 4.2
  }
})

// NFT metrics
const nftMetrics = ref({
  totalVolume: 2100000000,
  volumeChange: -15.3,
  floorPrices: [
    { collection: 'Bored Ape Yacht Club', floor: 45.2, change: -8.5 },
    { collection: 'CryptoPunks', floor: 85.7, change: -12.1 },
    { collection: 'Azuki', floor: 12.8, change: 5.3 },
    { collection: 'Clone X', floor: 8.2, change: -3.7 }
  ],
  marketplaces: [
    { name: 'OpenSea', volume: 1250000000, share: 59.5 },
    { name: 'Blur', volume: 520000000, share: 24.8 },
    { name: 'LooksRare', volume: 180000000, share: 8.6 },
    { name: 'X2Y2', volume: 150000000, share: 7.1 }
  ]
})

// Correlation matrix
const correlationMatrix = ref([
  ['BTC', 1.00, 0.85, 0.65, 0.42, 0.35, 0.28],
  ['ETH', 0.85, 1.00, 0.78, 0.55, 0.48, 0.39],
  ['BNB', 0.65, 0.78, 1.00, 0.62, 0.51, 0.44],
  ['ADA', 0.42, 0.55, 0.62, 1.00, 0.73, 0.58],
  ['SOL', 0.35, 0.48, 0.51, 0.73, 1.00, 0.82],
  ['AVAX', 0.28, 0.39, 0.44, 0.58, 0.82, 1.00]
])

// Volatility data
const volatilityData = computed(() => {
  const days = getTimeRangeDays(props.timeRange)
  return Array.from({ length: days }, (_, i) => {
    const date = new Date()
    date.setDate(date.getDate() - (days - i - 1))

    return {
      timestamp: date.toISOString(),
      btc: 0.3 + Math.random() * 0.5,
      eth: 0.35 + Math.random() * 0.6,
      altcoins: 0.5 + Math.random() * 0.8,
      market: 0.25 + Math.random() * 0.4
    }
  })
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e12) return `$${(amount / 1e12).toFixed(2)}T`
  if (amount >= 1e9) return `$${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `$${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `$${(amount / 1e3).toFixed(2)}K`
  return `$${amount.toFixed(2)}`
}

function getFearGreedColor(index: number): 'red' | 'orange' | 'yellow' | 'green' {
  if (index <= 25) return 'red'
  if (index <= 50) return 'orange'
  if (index <= 75) return 'yellow'
  return 'green'
}

function getTimeRangeDays(range: string): number {
  switch (range) {
    case '24h': return 1
    case '7d': return 7
    case '30d': return 30
    case '90d': return 90
    case '1y': return 365
    default: return 30
  }
}
</script>