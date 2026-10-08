<script setup lang="ts">
import { computed } from 'vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/model/catalog'
import type { Task } from '@/model/catalog'

/**
 * One task a note is on, inside its project's card. The body is inert: the two labelled keys at
 * its foot are the only ways out, one to the task, one taking the note off it.
 */
const props = withDefaults(defineProps<{ task: Task; busy?: boolean }>(), { busy: false })

defineEmits<{ detach: []; open: [] }>()

const stepShare = computed(() =>
  props.task.stepCount ? Math.round((props.task.stepsDone / props.task.stepCount) * 100) : 0
)
</script>

<template>
  <li
    class="placed-row px-3 py-2.5 transition-opacity duration-(--motion-quick)"
    :class="busy && 'opacity-50'"
    :aria-busy="busy || undefined"
    data-testid="note-placement-row"
    data-attached="true"
  >
    <p class="line-clamp-2 text-small font-medium leading-snug text-text">{{ task.title }}</p>

    <div class="mt-1.5 flex min-w-0 items-center gap-2">
      <span class="inline-flex shrink-0 items-center gap-1.5 text-micro text-text-muted">
        <span class="size-1.5 rounded-full" :class="TASK_STATUS_COLOR[task.status]" aria-hidden="true" />
        {{ TASK_STATUS_LABEL[task.status] }}
      </span>
      <span class="anchor-chip min-w-0 truncate px-1.5 text-micro leading-4">{{ task.label }}</span>
    </div>

    <div v-if="task.stepCount" class="mt-2 flex items-center gap-2" :title="`${task.stepsDone} of ${task.stepCount} steps done`">
      <span class="trough h-1 flex-1 overflow-hidden">
        <span class="tube-gold block h-full rounded-full" :class="stepShare === 100 && 'tube-safe'" :style="{ width: `${stepShare}%` }" />
      </span>
      <span class="font-mono text-micro tabular-nums text-text-subtle">{{ task.stepsDone }}/{{ task.stepCount }}</span>
    </div>

    <div class="mt-2 flex items-center gap-1">
      <button
        type="button"
        class="placed-action focus-ring"
        :title="`Open ${task.title} on the Tasks side`"
        :aria-label="`Open ${task.title}`"
        data-testid="note-placement-open"
        @click="$emit('open')"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M2.5 6h7M6.5 3l3 3-3 3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        Open task
      </button>
      <button
        type="button"
        class="placed-action placed-action-detach focus-ring ml-auto"
        :title="`Take this note off ${task.title}`"
        :aria-label="`Take this note off ${task.title}`"
        :disabled="busy"
        data-testid="note-placement-remove"
        @click="$emit('detach')"
      >
        <!-- A chain link with its middle cut: the note leaves the task, nothing is deleted. -->
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            d="M6.5 4.5l1-1a2.83 2.83 0 0 1 4 4l-1 1M9.5 11.5l-1 1a2.83 2.83 0 0 1-4-4l1-1M3 3l1.5 1.5M13 13l-1.5-1.5"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        Detach
      </button>
    </div>
  </li>
</template>

<style scoped>
.placed-row + .placed-row {
  border-top: 1px solid var(--color-border);
}

.placed-action {
  display: inline-flex;
  height: 26px;
  align-items: center;
  gap: 6px;
  border-radius: var(--radius-control);
  padding: 0 8px;
  font-size: var(--text-caption);
  color: var(--color-text-subtle);
  transition:
    background-color var(--motion-quick) ease,
    color var(--motion-quick) ease;

  &:hover:not(:disabled) {
    background-color: var(--color-surface-hover);
    color: var(--color-text);
  }

  &:disabled {
    cursor: not-allowed;
  }
}

/* Red only under the pointer: a column of detach keys is not a column of alarms. */
.placed-action-detach:hover:not(:disabled) {
  background-color: var(--color-danger-soft);
  color: var(--color-danger);
}
</style>
