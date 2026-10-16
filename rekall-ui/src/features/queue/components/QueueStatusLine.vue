<script setup lang="ts">
/** The run queue's one-line state, under the panel's header: scheduled, running or holding. Idle says nothing. */
import { computed } from 'vue'
import { formatResetIn } from '@/common/format/countdown'
import { clockLabel, type RunProgress, type RunQueue } from '@/features/queue/model/runQueue'

const props = defineProps<{ queue: RunQueue; progress: RunProgress; now: number }>()

interface QueueStatus {
  readonly tone: 'scheduled' | 'running' | 'holding'
  readonly title: string
  readonly detail: string
}

const status = computed<QueueStatus | null>(() => {
  const { queue, progress, now } = props
  switch (queue.state) {
    case 'SCHEDULED':
      return {
        tone: 'scheduled',
        title: `Starts at ${clockLabel(queue.startAt, now)}`,
        detail: `In ${formatResetIn(queue.startAt, now)}. Keep Rekall open: the queue runs inside it.`
      }
    case 'RUNNING':
      return {
        tone: 'running',
        title: progress.running ? `Running ${progress.running.taskTitle}` : 'Starting the next task',
        detail: `${progress.settled} of ${progress.total} done, ${progress.waiting} waiting.`
      }
    case 'HOLDING':
      return {
        tone: 'holding',
        title: `Holding until ${clockLabel(queue.holdUntil, now)}`,
        detail: `${queue.holdReason ?? ''} Resumes in ${formatResetIn(queue.holdUntil, now)}.`
      }
    default:
      return null
  }
})
</script>

<template>
  <Transition name="fade-quick">
    <div
      v-if="status"
      class="flex items-baseline gap-3 border-b px-6 py-2.5"
      :class="{
        'border-accent/30 bg-accent-soft': status.tone === 'running',
        'border-warn/30 bg-warn-soft': status.tone === 'holding',
        'border-border bg-surface-raised': status.tone === 'scheduled'
      }"
      role="status"
      aria-live="polite"
      data-testid="run-queue-status"
    >
      <span
        class="shrink-0 text-body font-semibold"
        :class="status.tone === 'holding' ? 'text-warn' : status.tone === 'running' ? 'text-accent' : 'text-text'"
      >
        {{ status.title }}
      </span>
      <span class="min-w-0 text-small text-text-muted">{{ status.detail }}</span>
    </div>
  </Transition>
</template>
