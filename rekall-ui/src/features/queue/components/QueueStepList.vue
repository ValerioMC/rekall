<script setup lang="ts">
/**
 * A task's steps under its row in the run queue panel: title and state only. No graph, no flow
 * and no actions; moving, removing and watching stay on the task row. The marks are the task
 * beads', so a claimed and an accepted step look alike: the queue treats both as complete.
 */
import QueueBead from '@/features/queue/components/QueueBead.vue'
import { stepBeadState, type RunQueueStep, type RunQueueStepState } from '@/features/queue/model/runQueue'

defineProps<{ steps: readonly RunQueueStep[]; live: boolean; listId: string }>()

const STATE_NAME: Readonly<Record<RunQueueStepState, string>> = {
  OPEN: 'Open',
  RUNNING: 'Running',
  CLAIMED: 'Claimed, waiting for your review',
  DONE: 'Accepted',
  NOT_NEEDED: 'Not needed'
}

function spokenState(step: RunQueueStep): string {
  return step.current ? `Current. ${STATE_NAME[step.state]}` : STATE_NAME[step.state]
}
</script>

<template>
  <ol :id="listId" class="flex min-w-0 flex-col gap-px" data-testid="queue-step-list">
    <li
      v-for="(step, index) in steps"
      :key="index"
      class="flex min-w-0 items-center gap-2 rounded-control px-1.5 py-1 text-caption"
      :class="step.current ? 'bg-surface-raised font-medium' : ''"
      :aria-current="step.current ? 'step' : undefined"
      :data-state="step.state"
      :data-current="step.current"
      data-testid="queue-step"
    >
      <QueueBead :state="stepBeadState(step)" size="md" :live="live" />
      <span
        class="min-w-0 truncate"
        :class="step.state === 'NOT_NEEDED' ? 'text-text-muted' : 'text-text'"
        :title="step.title"
      >
        {{ step.title }}
      </span>
      <span v-if="step.state === 'NOT_NEEDED'" class="shrink-0 text-text-muted" aria-hidden="true">Not needed</span>
      <span class="sr-only">{{ spokenState(step) }}</span>
    </li>
  </ol>
</template>
