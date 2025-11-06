<template>
  <Card variant="glass">
    <template #header>
      <div class="flex items-center space-x-3">
        <HeroIcon name="FireIcon" class="w-5 h-5 text-orange-400" />
        <div>
          <h3 class="text-lg font-semibold text-white">Activity Heatmap</h3>
          <p class="text-xs text-white/60">Whale activity by time</p>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <!-- Time Labels (Hours) -->
      <div class="flex items-center">
        <div class="w-12 text-xs text-white/40"></div>
        <div class="flex-1 grid grid-cols-12 gap-1 text-xs text-white/40">
          <div v-for="hour in [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22]" :key="hour" class="text-center">
            {{ hour.toString().padStart(2, '0') }}
          </div>
        </div>
      </div>

      <!-- Heatmap Grid -->
      <div class="space-y-1">
        <div v-for="dayData in data" :key="dayData.day" class="flex items-center">
          <!-- Day Label -->
          <div class="w-12 text-xs text-white/60 font-medium">{{ dayData.day }}</div>

          <!-- Hour Cells -->
          <div class="flex-1 grid grid-cols-24 gap-1">
            <div
              v-for="hourData in dayData.hours"
              :key="hourData.hour"
              :class="getCellClass(hourData.value)"
              :title="`${dayData.day} ${hourData.hour}:00 - ${hourData.count} activities ($${formatValue(hourData.value)}M)`"
              class="aspect-square rounded-sm cursor-pointer transition-all duration-200 hover:scale-110 hover:z-10 relative"
              @click="selectTimeSlot(dayData.day, hourData.hour, hourData)"
            >
              <div class="absolute inset-0 rounded-sm opacity-0 hover:opacity-100 transition-opacity bg-white/10"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- Legend -->
      <div class="flex items-center justify-between pt-4 border-t border-white/10">
        <div class="flex items-center space-x-2 text-xs text-white/60">
          <span>Less</span>
          <div class="flex space-x-1">
            <div class="w-3 h-3 rounded-sm bg-slate-700/50"></div>
            <div class="w-3 h-3 rounded-sm bg-blue-500/20"></div>
            <div class="w-3 h-3 rounded-sm bg-blue-500/40"></div>
            <div class="w-3 h-3 rounded-sm bg-blue-500/60"></div>
            <div class="w-3 h-3 rounded-sm bg-blue-500/80"></div>
            <div class="w-3 h-3 rounded-sm bg-blue-500"></div>
          </div>
          <span>More</span>
        </div>

        <div class="flex items-center space-x-4 text-xs text-white/60">
          <div class="flex items-center space-x-1">
            <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
            <span>Live</span>
          </div>
          <span>{{ totalActivities }} total activities</span>
        </div>
      </div>
    </div>

    <!-- Time Slot Details Modal -->
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
        v-if="selectedSlot"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @click="selectedSlot = null"
      >
        <div
          class="bg-slate-800/90 backdrop-blur border border-white/20 rounded-xl p-6 max-w-md w-full mx-4"
          @click.stop
        >
          <div class="flex items-center justify-between mb-4">
            <h4 class="text-lg font-semibold text-white">
              {{ selectedSlot.day }} {{ selectedSlot.hour }}:00
            </h4>
            <button
              @click="selectedSlot = null"
              class="p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              <HeroIcon name="XMarkIcon" class="w-5 h-5 text-white/70" />
            </button>
          </div>

          <div class="space-y-3">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <div class="text-xs text-white/60">Activity Count</div>
                <div class="text-xl font-bold text-white">{{ selectedSlot.data.count }}</div>
              </div>
              <div>
                <div class="text-xs text-white/60">Total Value</div>
                <div class="text-xl font-bold text-white">${{ formatValue(selectedSlot.data.value) }}M</div>
              </div>
            </div>

            <div>
              <div class="text-xs text-white/60 mb-2">Activity Breakdown</div>
              <div class="space-y-1">
                <div class="flex justify-between text-sm">
                  <span class="text-green-400">Buys: {{ Math.floor(selectedSlot.data.count * 0.6) }}</span>
                  <span class="text-green-400">${{ formatValue(selectedSlot.data.value * 0.65) }}M</span>
                </div>
                <div class="flex justify-between text-sm">
                  <span class="text-red-400">Sells: {{ Math.floor(selectedSlot.data.count * 0.4) }}</span>
                  <span class="text-red-400">${{ formatValue(selectedSlot.data.value * 0.35) }}M</span>
                </div>
              </div>
            </div>

            <Button
              @click="viewDetailedActivity"
              variant="primary"
              size="sm"
              class="w-full"
              icon-left="EyeIcon"
            >
              View Detailed Activity
            </Button>
          </div>
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

interface HeatmapData {
  day: string
  hours: {
    hour: number
    value: number
    count: number
  }[]
}

interface Props {
  data: HeatmapData[]
}

const props = defineProps<Props>()

const selectedSlot = ref<{
  day: string
  hour: number
  data: { value: number; count: number }
} | null>(null)

// Computed properties
const totalActivities = computed(() => {
  return props.data.reduce((total, day) => {
    return total + day.hours.reduce((dayTotal, hour) => dayTotal + hour.count, 0)
  }, 0)
})

const maxValue = computed(() => {
  return Math.max(...props.data.flatMap(day => day.hours.map(hour => hour.value)))
})

// Methods
function getCellClass(value: number): string {
  if (value === 0) return 'bg-slate-700/30'

  const intensity = value / maxValue.value

  if (intensity <= 0.2) return 'bg-blue-500/20 hover:bg-blue-500/30'
  if (intensity <= 0.4) return 'bg-blue-500/40 hover:bg-blue-500/50'
  if (intensity <= 0.6) return 'bg-blue-500/60 hover:bg-blue-500/70'
  if (intensity <= 0.8) return 'bg-blue-500/80 hover:bg-blue-500/90'
  return 'bg-blue-500 hover:bg-blue-400'
}

function formatValue(value: number): string {
  if (value >= 1000) return (value / 1000).toFixed(1) + 'B'
  return value.toFixed(1)
}

function selectTimeSlot(day: string, hour: number, data: { value: number; count: number }) {
  selectedSlot.value = { day, hour, data }
}

function viewDetailedActivity() {
  // Emit event to parent or navigate to detailed view
  selectedSlot.value = null
}
</script>

<style scoped>
/* Ensure grid cells maintain aspect ratio */
.grid-cols-24 {
  grid-template-columns: repeat(24, minmax(0, 1fr));
}

/* Custom hover effects */
.hover\:scale-110:hover {
  transform: scale(1.1);
  z-index: 10;
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
</style>