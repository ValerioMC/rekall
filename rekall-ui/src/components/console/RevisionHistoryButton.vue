<script setup lang="ts">
import { ref } from 'vue'
import RevisionHistoryDialog from '@/components/console/RevisionHistoryDialog.vue'
import type { TaskId } from '@/model/branded'
import type { RevisionKind } from '@/model/revision'

defineProps<{ taskId: TaskId; taskTitle: string; kind: RevisionKind }>()
const emit = defineEmits<{ restored: [] }>()

const open = ref(false)

function onRestored(): void {
  open.value = false
  emit('restored')
}
</script>

<template>
  <button
    type="button"
    class="focus-ring inline-flex h-7 shrink-0 items-center gap-1.5 rounded-[var(--radius-control)] border border-border bg-transparent px-2.5 text-xs font-medium text-text-muted transition-all duration-150 hover:border-border-strong hover:bg-surface-raised hover:text-text active:translate-y-px"
    title="Earlier versions, kept whenever something replaced or deleted this"
    aria-haspopup="dialog"
    data-testid="revision-history-open"
    @click="open = true"
  >
    <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M1.5 6a4.5 4.5 0 1 0 1.3-3.2" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <path d="M1.4 1.4v2.1h2.1" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M6 3.6V6l1.6 1" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span>History</span>
  </button>

  <Teleport to="body">
    <Transition name="dialog">
      <RevisionHistoryDialog
        v-if="open"
        :task-id="taskId"
        :task-title="taskTitle"
        :kind="kind"
        @close="open = false"
        @restored="onRestored"
      />
    </Transition>
  </Teleport>
</template>
