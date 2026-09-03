<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useDockLane } from '@/composables/useDockLane'
import { useNow } from '@/composables/useNow'
import { formatClock } from '@/common/format/duration'
import { identityHue } from '@/common/identity'
import type { TimeEntry } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * Every task being worked right now, reachable and stoppable from any screen.
 *
 * Parallel timers only earn their keep if you can see them all at a glance without going back
 * to the task that started them, and this is that glance. It renders nothing while nothing is
 * running, so it never competes for attention on a quiet day.
 *
 * Floating over the application is what makes it reachable from every screen and is also how it
 * came to sit on the step composer's Add button. It now measures itself into `useDockLane`, so
 * the corner is shared rather than taken, and rides 10px off the bottom: the height of the
 * composer's own padding, which puts the pill and that button on one line instead of stacked.
 */
const store = useConsoleStore()
const { runningEntries } = storeToRefs(store)
const { run } = useAsyncAction()
const router = useRouter()
const now = useNow()
const pill = useDockLane()

const expanded = ref(false)

const sortedRunning = computed(() =>
  [...runningEntries.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt))
)

const latest = computed(() => sortedRunning.value[0] ?? null)

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
  expanded.value = false
  if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
}
</script>

<template>
  <div
    v-if="runningEntries.length"
    class="fixed bottom-2.5 right-4 z-(--z-sticky) flex flex-col items-end gap-2"
    data-testid="running-dock"
  >
    <div
      v-if="expanded"
      class="rise w-[300px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
    >
      <header class="flex items-center justify-between border-b border-border px-3.5 py-2.5">
        <span class="text-[11px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
          Running now
        </span>
        <button
          class="focus-ring grid size-6 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Collapse"
          @click="expanded = false"
        >
          &times;
        </button>
      </header>

      <ul class="max-h-[280px] overflow-y-auto p-1.5">
        <li v-for="entry in sortedRunning" :key="entry.id" data-testid="running-dock-row">
          <div class="flex items-center gap-2 rounded-[var(--radius-control)] px-1.5 py-2 hover:bg-surface-raised">
            <button
              class="focus-ring flex min-w-0 flex-1 items-center gap-2 rounded-[var(--radius-control)] text-left"
              :title="`Jump to ${entry.taskTitle}`"
              @click="jumpTo(entry.taskId)"
            >
              <span class="time-dial shrink-0" aria-hidden="true" />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-[12.5px] font-medium text-text">
                  {{ entry.taskTitle }}
                </span>
                <span class="flex items-center gap-1 truncate font-mono text-[10px] text-anchor/80">
                  <span class="size-1 shrink-0 rounded-full" :style="{ backgroundColor: identityHue(projectIdOf(entry)).base }" aria-hidden="true" />
                  {{ entry.anchor }}
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
      </ul>
    </div>

    <!-- The pill alone carries the ref: it is the part that is always on screen, and so the
         part the rest of the interface has to leave room for. -->
    <button
      ref="pill"
      class="focus-ring glass flex h-10 items-center gap-2.5 rounded-full border border-border-strong pl-2.5 pr-3.5 shadow-lift transition-colors hover:border-accent/60"
      data-testid="running-dock-toggle"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span class="time-dial shrink-0" aria-hidden="true" />
      <span class="text-[12.5px] font-medium text-text">
        {{ runningEntries.length }} running
      </span>
      <span v-if="latest" class="font-mono text-[12px] tabular-nums text-accent">
        {{ formatClock(liveSeconds(latest)) }}
      </span>
    </button>
  </div>
</template>
