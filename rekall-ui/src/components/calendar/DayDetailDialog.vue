<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useModalGate } from '@/composables/useModalGate'
import { formatDuration } from '@/common/format/duration'
import { trapTabKey } from '@/common/a11y/focus-trap'
import type { DaySummaryRow } from '@/common/calendar/day-summary'
import type { TaskId } from '@/model/branded'

/**
 * Every task worked on one day, each with its total for that day rather than its sessions —
 * the calendar's unit is a day, `TimeLogDialog`'s is a task, and the two never try to be the
 * same list. Shell copied from `TimeLogDialog` on purpose: a dialog on this surface should feel
 * like the same object, not a different one that happens to also float over the page.
 */
const props = defineProps<{ date: Date; rows: readonly DaySummaryRow[] }>()
const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const router = useRouter()
const { open: openModal, close: closeModal } = useModalGate()

const panel = ref<HTMLElement | null>(null)

const heading = computed(() =>
  props.date.toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' })
)

const totalSeconds = computed(() => props.rows.reduce((sum, row) => sum + row.totalSeconds, 0))

function openTask(taskId: TaskId): void {
  store.selectTask(taskId)
  emit('close')
  if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
})
onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fade-in fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div
      ref="panel"
      class="rise flex max-h-[80vh] w-full max-w-[520px] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      :aria-label="`Time on ${heading}`"
      data-testid="day-detail-dialog"
    >
      <header class="flex items-center gap-3 border-b border-border px-6 py-4">
        <span class="min-w-0 flex-1 truncate text-[15px] font-semibold capitalize tracking-[-0.01em] text-text">
          {{ heading }}
        </span>
        <button
          class="focus-ring grid size-7 shrink-0 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Close"
          @click="emit('close')"
        >
          &times;
        </button>
      </header>

      <div class="min-h-0 flex-1 overflow-y-auto px-6 py-4">
        <p v-if="!rows.length" class="py-6 text-center text-[13px] text-text-subtle">
          Nothing tracked this day.
        </p>

        <button
          v-for="row in rows"
          :key="row.taskId"
          class="focus-ring mb-1.5 flex w-full items-center gap-3 rounded-[var(--radius-control)] border border-transparent px-3 py-2.5 text-left transition-colors last:mb-0 hover:border-border-strong hover:bg-surface-raised"
          data-testid="day-detail-row"
          @click="openTask(row.taskId)"
        >
          <span
            class="size-1.5 shrink-0 rounded-full"
            :class="row.isRunning ? 'animate-pulse bg-accent' : 'bg-text-subtle'"
            aria-hidden="true"
          />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-[13px] font-medium text-text">{{ row.taskTitle }}</span>
            <span class="block truncate font-mono text-[10.5px] text-anchor/80">{{ row.anchor }}</span>
          </span>
          <span class="shrink-0 font-mono text-[13px]" :class="row.isRunning ? 'text-accent' : 'text-text-muted'">
            {{ formatDuration(row.totalSeconds) }}
          </span>
        </button>
      </div>

      <footer class="flex items-center justify-between border-t border-border bg-canvas px-6 py-3.5">
        <span class="text-[11px] uppercase tracking-[0.09em] text-text-subtle">Day total</span>
        <span class="font-mono text-[16px] font-semibold text-accent">
          {{ formatDuration(totalSeconds) }}
        </span>
      </footer>
    </div>
  </div>
</template>
