<script setup lang="ts">
/**
 * One task in the run queue's order: its place, its bead on the rail, what it is and where it has
 * got to, and the moves it allows. Only a waiting task moves or comes off; the running one is
 * watched, not removed.
 */
import AppButton from '@/components/ui/AppButton.vue'
import QueueBead from '@/components/queue/QueueBead.vue'
import { isSettled, type RunQueueItem, type RunQueueItemState } from '@/model/runQueue'

defineProps<{
  item: RunQueueItem
  position: number
  last: boolean
  live: boolean
  busy: boolean
  canMoveEarlier: boolean
  canMoveLater: boolean
}>()

defineEmits<{ move: [delta: -1 | 1]; remove: []; watch: [] }>()

const STATE_WORD: Readonly<Record<RunQueueItemState, string>> = {
  QUEUED: 'Waiting',
  RUNNING: 'Running',
  FINISHED: 'Finished',
  SKIPPED: 'Skipped',
  FAILED: 'Failed'
}

const STATE_TONE: Readonly<Record<RunQueueItemState, string>> = {
  QUEUED: 'text-text-subtle',
  RUNNING: 'text-accent',
  FINISHED: 'text-safe',
  SKIPPED: 'text-filed',
  FAILED: 'text-danger'
}

const MOVE_BUTTON =
  'focus-ring grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text disabled:opacity-30'
</script>

<template>
  <li
    class="group relative grid grid-cols-[22px_16px_minmax(0,1fr)_auto] items-start gap-x-2.5 rounded-control py-2 pr-1"
    :class="item.state === 'RUNNING' ? 'bg-accent-soft' : ''"
    :data-state="item.state"
    data-testid="run-queue-item"
  >
    <span class="pt-0.5 text-right font-mono text-caption tabular-nums text-text-subtle">{{ position }}</span>

    <span class="relative flex h-full justify-center pt-[5px]">
      <QueueBead :state="item.state" size="md" :live="live" />
      <!-- The rail below a bead is solid once the run has passed it: that bead has had its turn. -->
      <span
        v-if="!last"
        class="absolute bottom-[-10px] top-[20px] w-px"
        :class="isSettled(item.state) ? 'bg-border-strong' : 'bg-border'"
        aria-hidden="true"
      />
    </span>

    <span class="flex min-w-0 flex-col gap-0.5">
      <span class="truncate text-body font-medium text-text">{{ item.taskTitle }}</span>
      <span class="flex min-w-0 items-baseline gap-2 text-caption">
        <span class="truncate font-mono text-anchor">{{ item.anchor }}</span>
        <span class="shrink-0" :class="STATE_TONE[item.state]">{{ STATE_WORD[item.state] }}</span>
      </span>
      <span v-if="item.detail" class="text-caption leading-snug text-text-muted">{{ item.detail }}</span>
    </span>

    <span class="flex items-center gap-0.5 self-center">
      <template v-if="item.state === 'QUEUED'">
        <button
          type="button"
          :class="MOVE_BUTTON"
          :aria-label="`Move ${item.taskTitle} earlier`"
          :disabled="busy || !canMoveEarlier"
          data-testid="run-queue-up"
          @click="$emit('move', -1)"
        >
          <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M4 10l4-4 4 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <button
          type="button"
          :class="MOVE_BUTTON"
          :aria-label="`Move ${item.taskTitle} later`"
          :disabled="busy || !canMoveLater"
          data-testid="run-queue-down"
          @click="$emit('move', 1)"
        >
          <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <AppButton
          variant="danger-quiet"
          size="sm"
          :disabled="busy"
          :aria-label="`Take ${item.taskTitle} off the queue`"
          data-testid="run-queue-remove"
          @click="$emit('remove')"
        >
          Remove
        </AppButton>
      </template>
      <AppButton
        v-else-if="item.state === 'RUNNING'"
        variant="secondary"
        size="sm"
        data-testid="run-queue-watch"
        @click="$emit('watch')"
      >
        Watch session
      </AppButton>
    </span>
  </li>
</template>
