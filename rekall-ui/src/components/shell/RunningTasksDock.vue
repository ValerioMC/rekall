<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useNow } from '@/composables/useNow'
import { formatClock } from '@/common/format/duration'
import type { TimeEntry } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * Every task being worked right now, reachable and stoppable from any screen.
 *
 * Parallel timers only earn their keep if you can see them all at a glance without going back
 * to the task that started them — this is that glance. It renders nothing while nothing is
 * running, so it never competes for attention on a quiet day.
 */
const store = useConsoleStore()
const { runningEntries } = storeToRefs(store)
const { run } = useAsyncAction()
const router = useRouter()
const now = useNow()

const expanded = ref(false)

const sortedRunning = computed(() =>
  [...runningEntries.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt))
)

const latest = computed(() => sortedRunning.value[0] ?? null)

function liveSeconds(entry: TimeEntry): number {
  return (now.value - Date.parse(entry.startedAt)) / 1000
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
    class="fixed bottom-4 right-4 z-(--z-sticky) flex flex-col items-end gap-2"
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
              <span class="flex h-3 shrink-0 items-end gap-[2px]" aria-hidden="true">
                <span
                  v-for="bar in 3"
                  :key="bar"
                  class="eq-bar w-[3px] rounded-full bg-accent"
                  :style="{ animationDelay: `${bar * 120}ms` }"
                />
              </span>
              <span class="min-w-0 flex-1">
                <span class="block truncate text-[12.5px] font-medium text-text">
                  {{ entry.taskTitle }}
                </span>
                <span class="block truncate font-mono text-[10px] text-anchor/80">
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

    <button
      class="focus-ring glass flex h-10 items-center gap-2.5 rounded-full border border-border-strong pl-2.5 pr-3.5 shadow-lift transition-colors hover:border-accent/60"
      data-testid="running-dock-toggle"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span class="flex h-3 shrink-0 items-end gap-[2px]" aria-hidden="true">
        <span
          v-for="bar in 3"
          :key="bar"
          class="eq-bar w-[3px] rounded-full bg-accent"
          :style="{ animationDelay: `${bar * 120}ms` }"
        />
      </span>
      <span class="text-[12.5px] font-medium text-text">
        {{ runningEntries.length }} running
      </span>
      <span v-if="latest" class="font-mono text-[12px] tabular-nums text-accent">
        {{ formatClock(liveSeconds(latest)) }}
      </span>
    </button>
  </div>
</template>
