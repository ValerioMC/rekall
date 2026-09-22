<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useNow } from '@/composables/useNow'
import { formatClock } from '@/common/format/duration'
import { identityHue } from '@/common/identity'
import DockPanel from './DockPanel.vue'
import type { TimeEntry } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

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

function projectIdOf(entry: TimeEntry): string {
  return store.tasks.find((t) => t.id === entry.taskId)?.projectId ?? entry.taskId
}

async function stop(taskId: TaskId): Promise<void> {
  await run(() => store.pauseTimer(taskId))
}

function jumpTo(taskId: TaskId): void {
  store.selectTask(taskId)
  emit('close')
  if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
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
    <li v-for="entry in sortedRunning" :key="entry.id" data-testid="running-dock-row">
      <div
        class="flex items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-2 hover:bg-surface-raised"
      >
        <button
          class="focus-ring flex min-w-0 flex-1 items-center gap-2.5 rounded-[var(--radius-control)] text-left"
          :title="`Jump to ${entry.taskTitle}`"
          data-testid="running-dock-jump"
          @click="jumpTo(entry.taskId)"
        >
          <span class="min-w-0 flex-1">
            <span class="block truncate text-[12.5px] font-medium text-text">
              {{ entry.taskTitle }}
            </span>
            <span class="mt-0.5 flex items-center gap-1.5 truncate font-mono text-[10px] text-anchor/80">
              <span
                class="size-1 shrink-0 rounded-full"
                :style="{ backgroundColor: identityHue(projectIdOf(entry)).base }"
                aria-hidden="true"
              />
              <span class="min-w-0 truncate">{{ entry.anchor }}</span>
            </span>
          </span>
        </button>
        <span class="shrink-0 font-mono text-[12px] tabular-nums text-accent">
          {{ formatClock(liveSeconds(entry)) }}
        </span>
        <button
          class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
          :aria-label="`Stop tracking ${entry.taskTitle}`"
          data-testid="running-dock-stop"
          @click="stop(entry.taskId)"
        >
          <svg class="size-3" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
            <rect x="4" y="4" width="8" height="8" rx="1.5" />
          </svg>
        </button>
      </div>
    </li>
  </DockPanel>
</template>
