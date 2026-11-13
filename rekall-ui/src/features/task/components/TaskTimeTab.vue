<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import TimerCard from '@/features/time/components/TimerCard.vue'
import TimeLogDialog from '@/features/time/components/TimeLogDialog.vue'
import { trackedSeconds } from '@/features/task/lib/tracked-seconds'
import { formatDate, formatDateTime } from '@/common/format/app-locale'
import { formatDuration } from '@/common/format/duration'
import { relativeTime } from '@/common/format/relative-time'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useNow } from '@/composables/useNow'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { Task } from '@/features/catalog/model/catalog'

/** The Time tab: what the task has tracked, its sessions, its tokens, and the timer that adds to them. */
const props = defineProps<{ task: Task }>()

const store = useConsoleStore()
const tokenUsage = useTokenUsageStore()
const { run } = useAsyncAction()
const now = useNow()
const { selectedTaskEntries, runningEntries } = storeToRefs(store)

const showLog = ref(false)
const isRunningHere = computed(() => runningEntries.value.some((entry) => entry.taskId === props.task.id))
const tracked = computed(() => trackedSeconds(selectedTaskEntries.value, now.value))
const generatedTokens = computed(() => tokenUsage.ofTask(props.task.id).generated)
const lastSession = computed(() => selectedTaskEntries.value[0] ?? null)

async function startTimer(): Promise<void> {
  await run(() => store.startTimer(props.task.id))
}

async function pauseTimer(): Promise<void> {
  await run(() => store.pauseTimer(props.task.id))
}

function sessionSeconds(startedAt: string, stoppedAt: string | null): number {
  const end = stoppedAt === null ? now.value : Date.parse(stoppedAt)
  return Math.max(0, end - Date.parse(startedAt)) / 1000
}
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto" data-testid="time-tab">
    <div class="flex w-full flex-col gap-5 px-6 py-5">
      <div class="grid grid-cols-3 gap-3">
        <div class="rounded-card border border-border bg-surface px-4 py-3" data-testid="time-stat-tracked">
          <p class="text-caption text-text-subtle">Tracked</p>
          <p class="mt-1 font-mono text-heading font-semibold tabular-nums text-text">{{ formatDuration(tracked) }}</p>
          <p class="text-caption text-text-subtle">
            {{ selectedTaskEntries.length }} session{{ selectedTaskEntries.length === 1 ? '' : 's' }}
          </p>
        </div>
        <div class="rounded-card border border-border bg-surface px-4 py-3" data-testid="time-stat-tokens">
          <p class="text-caption text-text-subtle">Tokens</p>
          <p class="mt-1 font-mono text-heading font-semibold tabular-nums text-text">{{ generatedTokens.toLocaleString() }}</p>
          <p class="text-caption text-text-subtle">generated</p>
        </div>
        <div class="rounded-card border border-border bg-surface px-4 py-3" data-testid="time-stat-last">
          <p class="text-caption text-text-subtle">Last session</p>
          <p class="mt-1 text-heading font-semibold text-text">{{ lastSession ? relativeTime(lastSession.startedAt) : 'None yet' }}</p>
          <p class="text-caption text-text-subtle">{{ lastSession ? formatDate(new Date(lastSession.startedAt)) : 'start the timer' }}</p>
        </div>
      </div>

      <TimerCard :entries="selectedTaskEntries" :is-running="isRunningHere" @start="startTimer" @pause="pauseTimer" @open-log="showLog = true" />

      <section v-if="selectedTaskEntries.length" aria-label="Sessions" data-testid="time-sessions">
        <p class="eyebrow mb-2">Sessions</p>
        <div
          v-for="entry in selectedTaskEntries"
          :key="entry.id"
          class="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 border-t border-border py-2 text-small"
        >
          <span class="text-text-muted">
            {{ formatDateTime(new Date(entry.startedAt)) }}<template v-if="entry.stoppedAt === null"> · running</template>
          </span>
          <span class="font-mono tabular-nums text-text-muted">{{ formatDuration(sessionSeconds(entry.startedAt, entry.stoppedAt)) }}</span>
        </div>
      </section>
      <p v-else class="text-small text-text-muted" data-testid="time-empty">No session yet. Start the timer, or let a session on this task start it.</p>
    </div>

    <Transition name="dialog">
      <TimeLogDialog v-if="showLog" :task="task" :entries="selectedTaskEntries" @close="showLog = false" />
    </Transition>
  </div>
</template>
