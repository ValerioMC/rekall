<script setup lang="ts">
/**
 * The run queue in the top bar: the dial, one short line of state, and a strand of beads, one
 * per task in the run. It is the queue's status and the way into it: a click opens the panel.
 *
 * The line says the one thing worth knowing in each state and nothing else: how many are lined
 * up, when a schedule starts, which task is running and how far along the run is, or when a hold
 * lifts. It shares the bar with the anchor field, which has to keep room for its placeholder, so
 * by default it is the dial and a figure; the words and the running task's step come in once the
 * header's right cell is 43rem wide, the task's title at 53rem, and the strand clips before it
 * overflows. The full sentence is always the label and the tooltip.
 */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import RunDial from '@/features/queue/components/RunDial.vue'
import QueueBead from '@/features/queue/components/QueueBead.vue'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useNow } from '@/composables/useNow'
import { formatResetIn } from '@/common/format/countdown'
import { clockLabel, isSettled, type RunDialFace } from '@/features/queue/model/runQueue'

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

/** The running task's current step with the count it is one of; none without a checklist or between steps. */
const runningStep = computed(() => {
  const steps = progress.value.steps
  if (!steps || steps.currentIndex === null) return null
  return { index: steps.currentIndex, total: steps.total }
})

const stepSentence = computed(() =>
  runningStep.value ? `, step ${runningStep.value.index} of ${runningStep.value.total}` : ''
)

const label = computed(() => {
  const { state, startAt, holdUntil, holdReason } = queue.value
  const { waiting, total, running } = progress.value
  if (state === 'SCHEDULED') {
    return `Run queue: ${waiting} ${waiting === 1 ? 'task' : 'tasks'}, starts at ${clockLabel(startAt, now.value)}, in ${formatResetIn(startAt, now.value)}`
  }
  if (state === 'RUNNING') {
    return running
      ? `Run queue: running ${running.taskTitle}, task ${ordinal.value} of ${total}${stepSentence.value}`
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
    class="status-segment focus-ring group min-w-0 max-w-[340px] pl-1.5"
    :class="
      queue.state === 'HOLDING'
        ? 'bg-warn-soft text-warn'
        : queue.state === 'RUNNING'
          ? 'text-text'
          : 'text-text-muted hover:text-text'
    "
    :aria-label="label"
    :title="label"
    aria-haspopup="dialog"
    :aria-expanded="panelOpen"
    :data-state="queue.state"
    data-testid="queue-beacon"
    @click="store.openPanel()"
  >
    <RunDial
      :face="face"
      :start-at="queue.startAt"
      :ceiling-percent="queue.ceilingPercent"
      :steps="progress.steps"
    />

    <span class="flex min-w-0 items-baseline gap-1.5" data-testid="queue-beacon-line">
      <template v-if="queue.state === 'SCHEDULED'">
        <span class="sr-only @min-[43rem]:not-sr-only">Starts</span>
        <span class="font-mono tabular-nums text-accent">{{ clockLabel(queue.startAt, now) }}</span>
      </template>
      <template v-else-if="queue.state === 'RUNNING'">
        <span class="font-mono tabular-nums text-accent">{{ ordinal }}/{{ progress.total }}</span>
        <span
          v-if="runningStep"
          class="hidden font-mono tabular-nums text-text-muted @min-[43rem]:inline"
          data-testid="queue-beacon-step"
        >
          step {{ runningStep.index }}/{{ runningStep.total }}
        </span>
        <span v-if="progress.running" class="hidden min-w-0 max-w-[160px] truncate text-text-muted @min-[53rem]:inline">
          {{ progress.running.taskTitle }}
        </span>
      </template>
      <template v-else-if="queue.state === 'HOLDING'">
        <span class="sr-only @min-[43rem]:not-sr-only">Resumes</span>
        <span class="font-mono tabular-nums">{{ clockLabel(queue.holdUntil, now) }}</span>
      </template>
      <template v-else>
        <span class="sr-only @min-[43rem]:not-sr-only">Queue</span>
        <span
          v-if="progress.waiting > 0"
          class="rounded-full bg-surface-hover px-1.5 font-mono text-micro tabular-nums leading-[15px] text-text-muted"
        >
          {{ progress.waiting }}
        </span>
      </template>
    </span>

    <span
      v-if="beads.length"
      class="flex min-w-0 items-center gap-[3px] overflow-hidden @max-[26rem]:hidden"
      data-testid="queue-beacon-strand"
    >
      <QueueBead
        v-for="item in beads"
        :key="item.id"
        :state="item.state"
        :live="queue.state === 'RUNNING'"
      />
      <span v-if="overflow" class="ml-0.5 font-mono text-micro text-text-subtle">+{{ overflow }}</span>
    </span>
  </button>
</template>
