<script setup lang="ts">
import { computed, nextTick, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { formatDuration } from '@/common/format/duration'
import AppDialog from '@/components/ui/AppDialog.vue'
import { identityHue } from '@/common/identity'
import TokenCount from '@/components/ui/TokenCount.vue'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import type { DaySummaryRow } from '@/features/time/lib/day-summary'
import type { ProjectId, TaskId } from '@/model/branded'

const props = defineProps<{ date: Date; rows: readonly DaySummaryRow[] }>()
const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const tokenUsage = useTokenUsageStore()
const router = useRouter()

const heading = computed(() =>
  props.date.toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' })
)

const totalSeconds = computed(() => props.rows.reduce((sum, row) => sum + row.totalSeconds, 0))

interface DayGroup {
  readonly key: string
  readonly projectId: ProjectId | null
  readonly projectLabel: string
  readonly rows: DaySummaryRow[]
  totalSeconds: number
}

const groups = computed<DayGroup[]>(() => {
  const byKey = new Map<string, DayGroup>()
  for (const row of props.rows) {
    const key = row.projectId ?? row.taskId
    let group = byKey.get(key)
    if (!group) {
      group = { key, projectId: row.projectId, projectLabel: row.projectLabel, rows: [], totalSeconds: 0 }
      byKey.set(key, group)
    }
    group.rows.push(row)
    group.totalSeconds += row.totalSeconds
  }
  return [...byKey.values()].sort((a, b) => b.totalSeconds - a.totalSeconds)
})

function openTask(taskId: TaskId): void {
  store.selectTask(taskId)
  emit('close')
  if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
}

onMounted(async () => {
  await nextTick()
})
</script>

<template>
  <AppDialog
    :label="`Time on ${heading}`"
    panel-class="flex flex-col max-h-[80vh] max-w-[520px]"
    panel-testid="day-detail-dialog"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <span class="min-w-0 flex-1 truncate text-title font-semibold capitalize tracking-[-0.01em] text-text">
        {{ heading }}
      </span>
      <button
        class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto px-6 py-4">
      <p v-if="!rows.length" class="py-6 text-center text-body text-text-subtle">
        Nothing tracked this day.
      </p>

      <template v-for="group in groups" :key="group.key">
        <div class="mb-1.5 mt-3 flex items-center gap-2 px-0.5 first:mt-0">
          <TokenCount v-if="group.projectId" :totals="tokenUsage.ofProject(group.projectId, store.tasks)" :hue-of="group.projectId" />
          <span class="eyebrow min-w-0 flex-1 truncate text-caption">
            {{ group.projectLabel }}
          </span>
          <span class="shrink-0 font-mono text-micro text-text-subtle">{{ formatDuration(group.totalSeconds) }}</span>
        </div>

        <button
          v-for="row in group.rows"
          :key="row.taskId"
          class="identity-rail focus-ring mb-1.5 flex w-full items-center gap-3 rounded-control border border-transparent px-3 py-2.5 text-left transition-colors last:mb-0 hover:border-border-strong hover:bg-surface-raised"
          data-testid="day-detail-row"
          :style="{ '--identity-color': identityHue(group.projectId ?? row.taskId).line }"
          @click="openTask(row.taskId)"
        >
          <span
            class="size-1.5 shrink-0 rounded-full"
            :class="row.isRunning ? 'animate-pulse bg-accent' : 'bg-text-subtle'"
            aria-hidden="true"
          />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-body font-medium text-text">{{ row.taskTitle }}</span>
            <span class="block truncate font-mono text-micro text-anchor/80">{{ row.anchor }}</span>
          </span>
          <span class="shrink-0 font-mono text-body" :class="row.isRunning ? 'text-accent' : 'text-text-muted'">
            {{ formatDuration(row.totalSeconds) }}
          </span>
        </button>
      </template>
    </div>

    <footer class="flex items-center justify-between border-t border-border bg-canvas px-6 py-3.5">
      <span class="eyebrow text-caption font-normal">Day total</span>
      <span class="font-mono text-title font-semibold text-accent">
        {{ formatDuration(totalSeconds) }}
      </span>
    </footer>
  </AppDialog>
</template>
