<script setup lang="ts">
/**
 * The run queue in the top bar: the dial, one short line of state, and a strand of beads, one
 * per task in the run. It is the queue's status and the way into it: a click opens the panel.
 *
 * The line says the one thing worth knowing in each state and nothing else: how many are lined
 * up, when a schedule starts, which task is running and how far along the run is, or when a hold
 * lifts. It shares the bar with the anchor field, which has to keep room for its placeholder, so
 * by default it is the dial and a figure; the words come in from 1680px and the running task's
 * title from 1760px. The full sentence is always the label and the tooltip.
 */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import RunDial from '@/components/queue/RunDial.vue'
import QueueBead from '@/components/queue/QueueBead.vue'
import { useRunQueueStore } from '@/stores/runQueue.store'
import { useNow } from '@/composables/useNow'
import { formatResetIn } from '@/common/format/countdown'
import { clockLabel, isSettled, type RunDialFace } from '@/model/runQueue'

const MAX_BEADS = 8

const store = useRunQueueStore()
const { queue, progress, panelOpen } = storeToRefs(store)
const now = useNow(15_000)

const face = computed<RunDialFace>(() => {
  switch (queue.value.state) {
    case 'SCHEDULED':
      return 'scheduled'
    case 'RUNNING':
      return 'running'
    case 'HOLDING':
      return 'holding'
    default:
      return progress.value.waiting > 0 ? 'ready' : 'empty'
  }
})

/** The run's beads: shown while the queue is armed, or while a finished run is still on the list. */
const beads = computed(() => {
  const items = queue.value.items
  const showing = queue.value.state !== 'IDLE' || items.some((item) => isSettled(item.state))
  return showing ? items.slice(0, MAX_BEADS) : []
})
const overflow = computed(() => Math.max(0, queue.value.items.length - MAX_BEADS))

/** Which of the run the running task is, counted from one. */
const ordinal = computed(() => progress.value.settled + (progress.value.running ? 1 : 0))

const label = computed(() => {
  const { state, startAt, holdUntil, holdReason } = queue.value
  const { waiting, total, running } = progress.value
  if (state === 'SCHEDULED') {
    return `Run queue: ${waiting} ${waiting === 1 ? 'task' : 'tasks'}, starts at ${clockLabel(startAt, now.value)}, in ${formatResetIn(startAt, now.value)}`
  }
  if (state === 'RUNNING') {
    return running
      ? `Run queue: running ${running.taskTitle}, ${ordinal.value} of ${total}`
      : `Run queue: starting the next task, ${progress.value.settled} of ${total} done`
  }
  if (state === 'HOLDING') {
    return `Run queue: holding. ${holdReason ?? ''} Resumes at ${clockLabel(holdUntil, now.value)}`
  }
  if (waiting > 0) return `Run queue: ${waiting} ${waiting === 1 ? 'task' : 'tasks'} waiting, not started`
  return 'Run queue: empty'
})
</script>

<template>
  <button
    type="button"
    class="focus-ring group flex h-8 max-w-[340px] shrink-0 items-center gap-1.5 rounded-[var(--radius-control)] border px-1.5 text-[12px] transition-colors hover:bg-surface-hover"
    :class="
      queue.state === 'HOLDING'
        ? 'border-warn/40 bg-warn-soft text-warn hover:border-warn'
        : queue.state === 'RUNNING'
          ? 'border-accent/50 bg-surface-raised text-text hover:border-accent'
          : 'border-border-strong bg-surface-raised text-text-subtle hover:border-accent'
    "
    :aria-label="label"
    :title="label"
    aria-haspopup="dialog"
    :aria-expanded="panelOpen"
    :data-state="queue.state"
    data-testid="queue-beacon"
    @click="store.openPanel()"
  >
    <RunDial :face="face" :start-at="queue.startAt" :ceiling-percent="queue.ceilingPercent" />

    <span class="flex min-w-0 items-baseline gap-1.5" data-testid="queue-beacon-line">
      <template v-if="queue.state === 'SCHEDULED'">
        <span class="sr-only min-[1680px]:not-sr-only">Starts</span>
        <span class="font-mono tabular-nums text-accent">{{ clockLabel(queue.startAt, now) }}</span>
      </template>
      <template v-else-if="queue.state === 'RUNNING'">
        <span class="font-mono tabular-nums text-accent">{{ ordinal }}/{{ progress.total }}</span>
        <span v-if="progress.running" class="hidden min-w-0 max-w-[160px] truncate text-text-muted min-[1760px]:inline">
          {{ progress.running.taskTitle }}
        </span>
      </template>
      <template v-else-if="queue.state === 'HOLDING'">
        <span class="sr-only min-[1680px]:not-sr-only">Resumes</span>
        <span class="font-mono tabular-nums">{{ clockLabel(queue.holdUntil, now) }}</span>
      </template>
      <template v-else>
        <span class="sr-only min-[1680px]:not-sr-only">Queue</span>
        <span
          v-if="progress.waiting > 0"
          class="rounded-full bg-surface-hover px-1.5 font-mono text-[10px] tabular-nums leading-[15px] text-text-muted"
        >
          {{ progress.waiting }}
        </span>
      </template>
    </span>

    <span
      v-if="beads.length"
      class="flex shrink-0 items-center gap-[3px] max-[1440px]:hidden"
      data-testid="queue-beacon-strand"
    >
      <QueueBead
        v-for="item in beads"
        :key="item.id"
        :state="item.state"
        :live="queue.state === 'RUNNING'"
      />
      <span v-if="overflow" class="ml-0.5 font-mono text-[10px] text-text-subtle">+{{ overflow }}</span>
    </span>
  </button>
</template>
