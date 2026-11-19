<script setup lang="ts">
import { computed } from 'vue'
import { isPlanComplete, tubeShares } from '@/common/plan/tube-shares'

/**
 * A plan as one gold tube: done in gold (green once every step is), claimed hatched, and while a
 * session works the step it is on carries the sheen. The numbers read out for a screen reader.
 */
const props = withDefaults(
  defineProps<{ done: number; claimed?: number; total: number; live?: boolean; size?: 'sm' | 'md' | 'lg' }>(),
  { claimed: 0, live: false, size: 'md' }
)

const shares = computed(() => tubeShares(props.done, props.claimed, props.total, props.live))
const complete = computed(() => isPlanComplete(props.done, props.total))
const height = computed(() => ({ sm: 'h-1.5', md: 'h-2', lg: 'h-2.5' })[props.size])
const label = computed(() => {
  const waiting = props.claimed > 0 ? `, ${props.claimed} waiting for review` : ''
  return `${props.done} of ${props.total} steps done${waiting}`
})
</script>

<template>
  <span class="trough flex min-w-0 overflow-hidden" :class="height" role="img" :aria-label="label" data-testid="plan-tube">
    <span
      v-if="shares.done > 0"
      class="tube-gold h-full transition-[width] duration-(--motion-slow)"
      :class="complete && 'tube-safe'"
      :style="{ width: `${shares.done}%` }"
      data-testid="plan-tube-done"
    />
    <span
      v-if="shares.claimed > 0"
      class="ledger-claimed h-full transition-[width] duration-(--motion-slow)"
      :style="{ width: `${shares.claimed}%` }"
      data-testid="plan-tube-claimed"
    />
    <span
      v-if="shares.running > 0"
      class="ledger-running h-full"
      :style="{ width: `${shares.running}%` }"
      data-testid="plan-tube-running"
    />
  </span>
</template>
