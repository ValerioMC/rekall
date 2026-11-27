<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useNow } from '@/composables/useNow'
import { formatClock } from '@/common/format/duration'
import DockEntry from './DockEntry.vue'
import DockPanel from './DockPanel.vue'
import type { TimeEntry } from '@/features/time/model/time-entry'
import type { ProjectId, TaskId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { runningEntries } = storeToRefs(store)
const { run } = useAsyncAction()
const router = useRouter()
const now = useNow()

const sortedRunning = computed(() =>
  [...runningEntries.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt))
)

function liveSeconds(entry: TimeEntry): number {
  return (now.value - Date.parse(entry.startedAt)) / 1000
}

function projectIdOf(entry: TimeEntry): ProjectId | null {
  return store.tasks.find((t) => t.id === entry.taskId)?.projectId ?? null
}

async function stop(taskId: TaskId): Promise<void> {
  await run(() => store.pauseTimer(taskId))
}

function jumpTo(taskId: TaskId): void {
  emit('close')
  void router.push(`/task/${taskId}`)
}
</script>

<template>
  <DockPanel
    panel-id="dock-panel-running"
    title="Running now"
    :count="runningEntries.length"
    tint="time"
    @close="emit('close')"
  >
    <DockEntry
      v-for="entry in sortedRunning"
      :key="entry.id"
      :project-id="projectIdOf(entry)"
      :title="entry.taskTitle"
      jump-test-id="running-dock-jump"
      data-testid="running-dock-row"
      @jump="jumpTo(entry.taskId)"
    >
      <div class="mt-1.5 flex min-h-6 items-center gap-2">
        <span class="min-w-0 flex-1 truncate font-mono text-caption text-anchor">{{ entry.anchor }}</span>
        <span class="shrink-0 font-mono text-body tabular-nums text-accent">
          {{ formatClock(liveSeconds(entry)) }}
        </span>
        <button
          class="focus-ring relative grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
          :aria-label="`Stop tracking ${entry.taskTitle}`"
          data-testid="running-dock-stop"
          @click="stop(entry.taskId)"
        >
          <svg class="size-3" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <rect x="4" y="4" width="8" height="8" rx="1.5" />
          </svg>
        </button>
      </div>
    </DockEntry>
  </DockPanel>
</template>
