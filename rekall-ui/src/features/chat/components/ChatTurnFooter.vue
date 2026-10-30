<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import { formatTokens } from '@/common/format/token-count'
import type { TurnOutcome, TurnUsage } from '@/features/chat/model/chat-event'

/** What a turn cost, on one line after its last item. */
const props = defineProps<{ usage: TurnUsage; durationMs: number; outcome: TurnOutcome }>()

const OUTCOME_LABELS: Readonly<Record<Exclude<TurnOutcome, 'completed'>, string>> = {
  interrupted: 'stopped',
  failed: 'failed'
}

const figures = computed(() =>
  [
    `in ${formatTokens(props.usage.inputTokens)}`,
    `cache read ${formatTokens(props.usage.cacheReadInputTokens)}`,
    `cache write ${formatTokens(props.usage.cacheCreationInputTokens)}`,
    `out ${formatTokens(props.usage.outputTokens)}`,
    `${(props.durationMs / 1000).toFixed(1)} s`
  ].join(' · ')
)
</script>

<template>
  <p class="flex flex-wrap items-center gap-2 font-mono text-micro text-text-subtle tabular-nums" data-testid="chat-turn-footer">
    <span>{{ figures }}</span>
    <AppBadge v-if="outcome !== 'completed'" :tone="outcome === 'failed' ? 'danger' : 'neutral'">{{
      OUTCOME_LABELS[outcome]
    }}</AppBadge>
  </p>
</template>
