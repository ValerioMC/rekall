<script setup lang="ts">
import { computed } from 'vue'
import TaskTokenChip from '@/features/tokens/components/TaskTokenChip.vue'
import { useContextSize } from '@/features/context/composables/useContextSize'
import { contextShares } from '@/features/context/lib/context-shares'
import { compactCount } from '@/features/context/model/context'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * What `/rk` hands a session for the task, as a total and a bar of its heaviest parts, with the
 * tokens the task has generated beneath. The estimate is for comparing, not a bill.
 */
const props = defineProps<{ task: Task }>()

const MAX_ROWS = 5
const SEGMENT_TONES = ['bg-text-muted', 'bg-text-subtle', 'bg-border-field', 'bg-border-strong', 'bg-border'] as const

const { size, failed } = useContextSize(() => props.task.id)
const shares = computed(() => (size.value ? contextShares(size.value, MAX_ROWS) : []))
const heaviestNote = computed(
  () => size.value?.parts.find((part) => part.label.startsWith('Note: ') && !part.reference) ?? null
)
</script>

<template>
  <div data-testid="context-group">
    <p v-if="failed" class="text-small text-text-subtle" data-testid="context-size">size unknown</p>
    <p v-else-if="!size" class="text-small text-text-subtle" data-testid="context-size">measuring…</p>
    <template v-else>
      <p class="flex items-baseline justify-between gap-2">
        <span class="font-mono text-body tabular-nums text-text" data-testid="context-size">≈ {{ compactCount(size.estimatedTokens) }} tokens</span>
        <span class="text-caption text-text-subtle">what /rk hands over</span>
      </p>
      <div class="mt-2 flex h-1.5 gap-0.5 overflow-hidden rounded-full bg-canvas shadow-[var(--shadow-well)]" role="presentation">
        <span
          v-for="(share, index) in shares"
          :key="share.label"
          class="block h-full"
          :class="SEGMENT_TONES[index]"
          :style="{ width: `${share.percent}%` }"
        />
      </div>
      <ul class="mt-2.5 grid gap-1.5" data-testid="context-size-parts">
        <li v-for="(share, index) in shares" :key="share.label" class="flex items-center gap-2 text-caption text-text-muted">
          <span class="size-2 shrink-0 rounded-key" :class="SEGMENT_TONES[index]" aria-hidden="true" />
          <span class="min-w-0 flex-1 truncate" :title="share.label">{{ share.label }}</span>
          <span v-if="share.reference" class="shrink-0 text-micro text-accent">reference</span>
          <span class="shrink-0 font-mono tabular-nums">≈ {{ compactCount(share.tokens) }}</span>
        </li>
      </ul>
      <p v-if="heaviestNote" class="mt-2.5 text-caption leading-relaxed text-text-subtle">
        A note most sessions do not need can go <span class="text-text">By reference</span> from its own pane.
      </p>
    </template>
    <div class="mt-3 flex items-center justify-between gap-2 border-t border-border pt-2.5 text-caption text-text-muted">
      <span>Generated so far</span>
      <TaskTokenChip :task="task" />
    </div>
  </div>
</template>
