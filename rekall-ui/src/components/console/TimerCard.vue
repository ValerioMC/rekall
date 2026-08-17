<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { relativeTime } from '@/common/format/relative-time'
import { formatClock, formatDuration } from '@/common/format/duration'
import { useNow } from '@/composables/useNow'
import type { TimeEntry } from '@/model/catalog'

/**
 * How long this task has taken, pinned above the wrapup because it is the one thing on this
 * pane that is live: everything else describes the task, this one is ticking while you read it.
 *
 * Props in, events out, the same shape `WrapupCard` uses — the store's timer actions live in
 * `NoteListPane`, not here.
 */
const props = defineProps<{
  entries: readonly TimeEntry[]
  isRunning: boolean
}>()

const emit = defineEmits<{ start: []; pause: []; openLog: [] }>()

const now = useNow()

const runningEntry = computed(() => props.entries.find((entry) => entry.stoppedAt === null) ?? null)
const hasHistory = computed(() => props.entries.length > 0)

const closedSeconds = computed(() =>
  props.entries.reduce((sum, entry) => {
    if (!entry.stoppedAt) return sum
    return sum + (Date.parse(entry.stoppedAt) - Date.parse(entry.startedAt)) / 1000
  }, 0)
)

/** The current session's elapsed time, ticking — zero unless it is this task that is running. */
const liveSeconds = computed(() => {
  if (!props.isRunning || !runningEntry.value) return 0
  return (now.value - Date.parse(runningEntry.value.startedAt)) / 1000
})

const totalSeconds = computed(() => closedSeconds.value + liveSeconds.value)

const lastStoppedAt = computed(() => {
  const closed = props.entries.filter((entry) => entry.stoppedAt).map((entry) => entry.stoppedAt as string)
  return closed.length ? closed.reduce((a, b) => (a > b ? a : b)) : null
})

const buttonLabel = computed(() => (props.isRunning ? 'Pause' : hasHistory.value ? 'Resume' : 'Start'))

function toggle(): void {
  if (props.isRunning) emit('pause')
  else emit('start')
}
</script>

<template>
  <div
    class="mb-2 rounded-[var(--radius-control)] border p-2.5 transition-colors"
    :class="isRunning ? 'border-accent/50 bg-accent-soft' : 'border-border-strong bg-surface-raised'"
    data-testid="timer-card"
  >
    <div class="flex items-center gap-2">
      <span class="relative flex size-2 shrink-0" aria-hidden="true">
        <span
          v-if="isRunning"
          class="absolute inline-flex size-full animate-ping rounded-full bg-accent opacity-75"
        />
        <span
          class="relative inline-flex size-2 rounded-full"
          :class="isRunning ? 'bg-accent' : 'bg-text-subtle'"
        />
      </span>
      <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">Time</span>

      <button
        v-if="hasHistory"
        class="focus-ring ml-auto grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-hover hover:text-text"
        title="View all sessions"
        aria-label="View all sessions"
        data-testid="timer-open-log"
        @click="emit('openLog')"
      >
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <circle cx="8" cy="8.5" r="6" stroke="currentColor" stroke-width="1.4" />
          <path d="M8 5.3v3.4l2.4 1.4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <div class="mt-2 flex items-end justify-between gap-3">
      <span
        class="texture-scan rounded-[6px] px-1.5 font-mono text-[22px] font-semibold tabular-nums tracking-tight"
        :class="isRunning ? 'text-accent' : 'text-text-muted'"
        data-testid="timer-display"
      >
        {{ isRunning ? formatClock(liveSeconds) : formatDuration(totalSeconds) }}
      </span>
      <AppButton
        :variant="isRunning ? 'secondary' : 'primary'"
        size="sm"
        data-testid="timer-toggle"
        @click="toggle"
      >
        {{ buttonLabel }}
      </AppButton>
    </div>

    <p v-if="isRunning && hasHistory" class="mt-1.5 text-[11px] text-text-subtle">
      {{ formatDuration(totalSeconds) }} total &middot; {{ entries.length }}
      session{{ entries.length === 1 ? '' : 's' }}
    </p>
    <p v-else-if="hasHistory" class="mt-1.5 text-[11px] text-text-subtle">
      {{ entries.length }} session{{ entries.length === 1 ? '' : 's' }}
      <template v-if="lastStoppedAt"> &middot; stopped {{ relativeTime(lastStoppedAt) }}</template>
    </p>
  </div>
</template>
