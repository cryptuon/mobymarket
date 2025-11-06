<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center space-x-3">
          <HeroIcon name="TrophyIcon" class="w-5 h-5 text-yellow-400" />
          <div>
            <h3 class="text-lg font-semibold text-white">Top Performers</h3>
            <p class="text-xs text-white/60">Best performing assets</p>
          </div>
        </div>

        <select
          v-model="sortBy"
          class="bg-slate-800/50 border border-slate-600/50 rounded-lg px-3 py-1 text-white text-xs focus:outline-none focus:border-moby-500/50"
        >
          <option value="returns">Returns</option>
          <option value="pnl">P&L Amount</option>
          <option value="allocation">Allocation</option>
        </select>
      </div>
    </template>

    <div class="space-y-3">
      <!-- Loading State -->
      <div v-if="loading" class="space-y-3">
        <div v-for="i in 5" :key="i" class="animate-pulse">
          <div class="flex items-center space-x-3 p-3 bg-slate-800/30 rounded-lg">
            <div class="w-8 h-8 bg-slate-700/50 rounded-full"></div>
            <div class="flex-1 space-y-2">
              <div class="h-4 bg-slate-700/50 rounded w-20"></div>
              <div class="h-3 bg-slate-700/50 rounded w-16"></div>
            </div>
            <div class="h-4 bg-slate-700/50 rounded w-16"></div>
          </div>
        </div>
      </div>

      <!-- Performers List -->
      <div v-else class="space-y-2">
        <TransitionGroup
          name="performer-list"
          tag="div"
          class="space-y-2"
        >
          <div
            v-for="(performer, index) in sortedPerformers"
            :key="performer.symbol"
            class="flex items-center space-x-3 p-3 bg-slate-800/30 hover:bg-slate-700/30 rounded-lg transition-all duration-200 cursor-pointer group"
            @click="selectPerformer(performer)"
          >
            <!-- Rank Badge -->
            <div :class="getRankBadgeClass(index + 1)" class="px-2 py-1 rounded-lg text-xs font-bold flex-shrink-0">
              #{{ index + 1 }}
            </div>

            <!-- Asset Info -->
            <div class="flex items-center space-x-3 flex-1 min-w-0">
              <!-- Asset Icon -->
              <div class="relative">
                <img
                  :src="getAssetIcon(performer.symbol)"
                  :alt="performer.symbol"
                  class="w-8 h-8 rounded-full"
                  @error="handleImageError"
                />
                <!-- Performance Indicator -->
                <div
                  :class="getPerformanceIndicator(performer.returns)"
                  class="absolute -bottom-1 -right-1 w-3 h-3 rounded-full border-2 border-slate-800"
                ></div>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center space-x-2">
                  <span class="text-white font-medium">{{ performer.symbol }}</span>
                  <span class="text-xs text-white/60 truncate">{{ performer.name }}</span>
                </div>
                <div class="text-xs text-white/60">
                  {{ performer.allocation.toFixed(1) }}% allocation
                </div>
              </div>
            </div>

            <!-- Performance Metrics -->
            <div class="text-right space-y-1 flex-shrink-0">
              <div :class="['font-bold', getReturnColor(performer.returns)]">
                {{ performer.returns >= 0 ? '+' : '' }}{{ performer.returns.toFixed(1) }}%
              </div>
              <div class="text-xs text-white/60">
                ${{ formatCurrency(performer.pnl) }}
              </div>
            </div>

            <!-- Performance Bar -->
            <div class="w-16 flex-shrink-0">
              <div class="h-2 bg-slate-700/50 rounded-full overflow-hidden">
                <div
                  :class="getPerformanceBarClass(performer.returns)"
                  :style="{ width: `${Math.min(100, Math.abs(performer.returns) * 2)}%` }"
                  class="h-full transition-all duration-500"
                ></div>
              </div>
              <div class="text-xs text-white/60 text-center mt-1">
                {{ getPerformanceRating(performer.returns) }}
              </div>
            </div>
          </div>
        </TransitionGroup>
      </div>

      <!-- Show More Button -->
      <div v-if="!loading && data.length > 5" class="pt-3 border-t border-white/10">
        <Button
          @click="showAll = !showAll"
          variant="ghost"
          size="sm"
          class="w-full"
          :icon-right="showAll ? 'ChevronUpIcon' : 'ChevronDownIcon'"
        >
          {{ showAll ? 'Show Less' : `Show All ${data.length} Assets` }}
        </Button>
      </div>

      <!-- Performance Summary -->
      <div class="pt-4 border-t border-white/10 grid grid-cols-3 gap-4 text-center">
        <div>
          <div class="text-xs text-white/60">Best Performer</div>
          <div class="text-sm font-bold text-green-400">
            {{ bestPerformer?.symbol }} {{ bestPerformer?.returns.toFixed(1) }}%
          </div>
        </div>
        <div>
          <div class="text-xs text-white/60">Avg Return</div>
          <div :class="['text-sm font-bold', getReturnColor(averageReturn)]">
            {{ averageReturn >= 0 ? '+' : '' }}{{ averageReturn.toFixed(1) }}%
          </div>
        </div>
        <div>
          <div class="text-xs text-white/60">Winners</div>
          <div class="text-sm font-bold text-white">
            {{ winnersCount }}/{{ data.length }}
          </div>
        </div>
      </div>
    </div>

    <!-- Asset Detail Modal -->
    <Transition
      name="modal"
      enter-active-class="transition-all duration-200"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition-all duration-150"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div
        v-if="selectedPerformer"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
        @click="selectedPerformer = null"
      >
        <div
          class="bg-slate-800/90 backdrop-blur border border-white/20 rounded-xl p-6 max-w-md w-full"
          @click.stop
        >
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center space-x-3">
              <img
                :src="getAssetIcon(selectedPerformer.symbol)"
                :alt="selectedPerformer.symbol"
                class="w-10 h-10 rounded-full"
              />
              <div>
                <h4 class="text-lg font-semibold text-white">{{ selectedPerformer.symbol }}</h4>
                <p class="text-sm text-white/60">{{ selectedPerformer.name }}</p>
              </div>
            </div>
            <button
              @click="selectedPerformer = null"
              class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
            </button>
          </div>

          <div class="grid grid-cols-2 gap-4 mb-4">
            <div>
              <div class="text-xs text-white/60">Total Return</div>
              <div :class="['text-xl font-bold', getReturnColor(selectedPerformer.returns)]">
                {{ selectedPerformer.returns >= 0 ? '+' : '' }}{{ selectedPerformer.returns.toFixed(2) }}%
              </div>
            </div>
            <div>
              <div class="text-xs text-white/60">P&L Amount</div>
              <div :class="['text-xl font-bold', getReturnColor(selectedPerformer.pnl)]">
                {{ selectedPerformer.pnl >= 0 ? '+' : '' }}${{ formatCurrency(Math.abs(selectedPerformer.pnl)) }}
              </div>
            </div>
            <div>
              <div class="text-xs text-white/60">Portfolio Weight</div>
              <div class="text-xl font-bold text-white">{{ selectedPerformer.allocation.toFixed(1) }}%</div>
            </div>
            <div>
              <div class="text-xs text-white/60">Contribution</div>
              <div class="text-xl font-bold text-white">
                {{ (selectedPerformer.returns * selectedPerformer.allocation / 100).toFixed(2) }}%
              </div>
            </div>
          </div>

          <Button
            @click="selectedPerformer = null"
            variant="primary"
            size="sm"
            class="w-full"
          >
            Close
          </Button>
        </div>
      </div>
    </Transition>
  </Card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import Card from '@components/ui/Card.vue'
import Button from '@components/ui/Button.vue'
import HeroIcon from '@components/ui/HeroIcon.vue'

interface Performer {
  symbol: string
  name: string
  pnl: number
  returns: number
  allocation: number
}

interface Props {
  data: Performer[]
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const sortBy = ref('returns')
const showAll = ref(false)
const selectedPerformer = ref<Performer | null>(null)

// Computed properties
const sortedPerformers = computed(() => {
  const sorted = [...props.data].sort((a, b) => {
    switch (sortBy.value) {
      case 'pnl':
        return b.pnl - a.pnl
      case 'allocation':
        return b.allocation - a.allocation
      default:
        return b.returns - a.returns
    }
  })

  return showAll.value ? sorted : sorted.slice(0, 5)
})

const bestPerformer = computed(() => {
  return props.data.reduce((best, current) =>
    current.returns > best.returns ? current : best, props.data[0]
  )
})

const averageReturn = computed(() => {
  if (props.data.length === 0) return 0
  const sum = props.data.reduce((total, performer) => total + performer.returns, 0)
  return sum / props.data.length
})

const winnersCount = computed(() => {
  return props.data.filter(performer => performer.returns > 0).length
})

// Methods
function formatCurrency(amount: number): string {
  if (amount >= 1e9) return `${(amount / 1e9).toFixed(2)}B`
  if (amount >= 1e6) return `${(amount / 1e6).toFixed(2)}M`
  if (amount >= 1e3) return `${(amount / 1e3).toFixed(2)}K`
  return amount.toFixed(0)
}

function getRankBadgeClass(rank: number): string {
  if (rank === 1) return 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30'
  if (rank === 2) return 'bg-gray-400/20 text-gray-300 border border-gray-400/30'
  if (rank === 3) return 'bg-orange-500/20 text-orange-400 border border-orange-500/30'
  return 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
}

function getAssetIcon(symbol: string): string {
  const iconMap: Record<string, string> = {
    ETH: '/tokens/eth.svg',
    BTC: '/tokens/btc.svg',
    UNI: '/tokens/uni.svg',
    AAVE: '/tokens/aave.svg',
    COMP: '/tokens/comp.svg',
    SUSHI: '/tokens/sushi.svg'
  }
  return iconMap[symbol] || '/tokens/default.svg'
}

function handleImageError(event: Event) {
  const img = event.target as HTMLImageElement
  img.src = '/tokens/default.svg'
}

function getPerformanceIndicator(returns: number): string {
  if (returns >= 20) return 'bg-green-400 animate-pulse'
  if (returns >= 0) return 'bg-green-400'
  if (returns >= -10) return 'bg-yellow-400'
  return 'bg-red-400'
}

function getReturnColor(returns: number): string {
  return returns >= 0 ? 'text-green-400' : 'text-red-400'
}

function getPerformanceBarClass(returns: number): string {
  if (returns >= 20) return 'bg-green-400'
  if (returns >= 10) return 'bg-green-500'
  if (returns >= 0) return 'bg-blue-400'
  if (returns >= -10) return 'bg-yellow-400'
  return 'bg-red-400'
}

function getPerformanceRating(returns: number): string {
  if (returns >= 20) return 'Excellent'
  if (returns >= 10) return 'Great'
  if (returns >= 0) return 'Good'
  if (returns >= -10) return 'Poor'
  return 'Bad'
}

function selectPerformer(performer: Performer) {
  selectedPerformer.value = performer
}
</script>

<style scoped>
/* List animations */
.performer-list-enter-active {
  transition: all 0.3s ease-out;
}

.performer-list-enter-from {
  transform: translateX(-20px);
  opacity: 0;
}

.performer-list-leave-active {
  transition: all 0.2s ease-in;
}

.performer-list-leave-to {
  transform: translateX(20px);
  opacity: 0;
}

.performer-list-move {
  transition: transform 0.3s ease;
}

/* Modal animations */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

/* Performance bar animations */
.transition-all {
  transition: all 0.5s ease;
}
</style>