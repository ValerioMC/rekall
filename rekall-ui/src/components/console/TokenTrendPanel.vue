<script setup lang="ts">
import { computed, ref } from 'vue'
import TokenTrendChart from '@/components/console/TokenTrendChart.vue'
import { formatTokens } from '@/common/format/token-count'
import { TREND_RANGES, trendOf, utcToday, type TrendRange } from '@/common/token/token-trend'
import type { TaskId } from '@/model/branded'
import type { TokenDay, TokenTotals } from '@/model/token-usage'

/** What opens from a token count: the running total of `taskIds` over one month, three, or all of it. */
const props = defineProps<{
  taskIds: readonly TaskId[]
  days: readonly TokenDay[]
  totals: TokenTotals
  color: string
  subject: 'task' | 'project'
}>()

const range = ref<TrendRange>('30d')

const points = computed(() => trendOf(props.days, new Set(props.taskIds), range.value, utcToday()))

const readings = computed(() => [
  { label: 'Input', value: props.totals.input },
  { label: 'Cache write', value: props.totals.cacheWrite },
  { label: 'Cache read', value: props.totals.cacheRead }
])
</script>

<template>
  <div class="px-3.5 pb-3 pt-3" data-testid="token-trend-panel">
    <div class="mb-2.5 flex items-center justify-between gap-3">
      <p class="eyebrow">Tokens generated</p>
      <div
        class="flex shrink-0 gap-0.5 rounded-control bg-canvas p-0.5"
        role="group"
        aria-label="Period shown"
      >
        <button
          v-for="option in TREND_RANGES"
          :key="option.id"
          type="button"
          class="focus-ring h-6 cursor-pointer rounded-control px-2 text-caption transition-colors duration-(--motion-quick)"
          :class="
            range === option.id
              ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
              : 'text-text-subtle hover:text-text'
          "
          :aria-pressed="range === option.id"
          :data-testid="`token-trend-range-${option.id}`"
          @click="range = option.id"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <TokenTrendChart v-if="points.length" :points="points" :color="color" />
    <p v-else class="py-6 text-center text-caption text-text-subtle" data-testid="token-trend-empty">
      No session has generated tokens on this {{ subject }} yet.
    </p>

    <dl class="mt-3 grid grid-cols-3 gap-2 border-t border-border pt-2.5">
      <div v-for="reading in readings" :key="reading.label">
        <dt class="text-micro uppercase tracking-[0.04em] text-text-subtle">{{ reading.label }}</dt>
        <dd class="font-mono text-small tabular-nums text-text-muted">{{ formatTokens(reading.value) }}</dd>
      </div>
    </dl>
  </div>
</template>
