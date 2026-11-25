<script setup lang="ts">
import { computed } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import { formatTokens } from '@/common/format/token-count'
import { contextFillLevel } from '@/features/context-fill/lib/context-fill-level'
import type { ContextFillLevel, ContextFillSubject } from '@/features/context-fill/model/context-fill'
import { useConsoleStore } from '@/stores/console.store'

/**
 * How full the context window of a chat session was at the end of its last turn on a task or on
 * one of its steps: the percent and a thin bar, warmer as it fills. Nothing when no turn has
 * recorded a fill.
 */
const props = defineProps<{ subject: ContextFillSubject }>()

const LEVEL_TONES: Readonly<Record<ContextFillLevel, { readonly text: string; readonly bar: string }>> = {
  neutral: { text: 'text-text-muted', bar: 'bg-text-muted' },
  warning: { text: 'text-warn', bar: 'bg-warn' },
  danger: { text: 'text-danger', bar: 'bg-danger' }
}

const store = useConsoleStore()

const fill = computed(() => store.contextFillOf(props.subject))
const tone = computed(() => LEVEL_TONES[contextFillLevel(fill.value?.percent ?? 0)])
const description = computed(() => {
  const current = fill.value
  if (!current) return ''
  const held = `${formatTokens(current.usedTokens)} of ${formatTokens(current.windowTokens)} tokens of the context window`
  return `${held}, recorded ${relativeTime(current.recordedAt)}`
})
</script>

<template>
  <span
    v-if="fill"
    class="inline-flex h-6 shrink-0 items-center gap-1.5 rounded-key border border-border bg-surface px-2 font-mono text-caption tabular-nums"
    :class="tone.text"
    :title="description"
    :aria-label="`Context ${fill.percent}% full. ${description}`"
    data-testid="context-fill-badge"
    :data-level="contextFillLevel(fill.percent)"
    role="img"
  >
    <span class="relative h-1 w-8 overflow-hidden rounded-full bg-canvas" aria-hidden="true">
      <span class="absolute inset-y-0 left-0 rounded-full" :class="tone.bar" :style="{ width: `${fill.percent}%` }" />
    </span>
    {{ fill.percent }}%
  </span>
</template>
