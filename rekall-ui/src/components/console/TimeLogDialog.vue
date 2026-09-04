<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useModalGate } from '@/composables/useModalGate'
import { useNow } from '@/composables/useNow'
import { formatDuration } from '@/common/format/duration'
import { trapTabKey } from '@/common/a11y/focus-trap'
import type { Task, TimeEntry } from '@/model/catalog'
import type { TimeEntryId } from '@/model/branded'

/**
 * The recap: every session on this task, grouped by the day it was worked, with the running
 * total at the bottom. Shell copied from `RecordDialog`; the content is a log rather than a
 * form, so rows edit in place instead of opening a second dialog.
 */
const props = defineProps<{ task: Task; entries: readonly TimeEntry[] }>()
const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { run } = useAsyncAction()
const { open: openModal, close: closeModal } = useModalGate()
const now = useNow()

const panel = ref<HTMLElement | null>(null)
const editingId = ref<TimeEntryId | null>(null)
const editStart = ref('')
const editStop = ref('')
const deletingId = ref<TimeEntryId | null>(null)
/**
 * Captured when the confirm opens, not read live from `entries`: the row is gone from that
 * array the instant the delete succeeds, and a confirm still on screen for its closing
 * animation must not go looking for what it described.
 */
const deletingSummary = ref('')

function toLocalInput(iso: string): string {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`
}

function fromLocalInput(value: string): string {
  return new Date(value).toISOString()
}

function timeOf(iso: string): string {
  return new Date(iso).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
}

function dayLabel(iso: string): string {
  const d = new Date(iso)
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(today.getDate() - 1)
  const sameDay = (a: Date, b: Date) =>
    a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate()
  if (sameDay(d, today)) return 'Today'
  if (sameDay(d, yesterday)) return 'Yesterday'
  return d.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' })
}

/** Entries arrive newest-first, so grouping preserves that order without a re-sort. */
const groups = computed(() => {
  const byDay = new Map<string, TimeEntry[]>()
  for (const entry of props.entries) {
    const d = new Date(entry.startedAt)
    const key = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`
    const bucket = byDay.get(key)
    if (bucket) bucket.push(entry)
    else byDay.set(key, [entry])
  }
  return [...byDay.values()].map((entries) => ({ label: dayLabel(entries[0]!.startedAt), entries }))
})

function durationOf(entry: TimeEntry): number {
  const end = entry.stoppedAt ? Date.parse(entry.stoppedAt) : now.value
  return (end - Date.parse(entry.startedAt)) / 1000
}

const totalSeconds = computed(() =>
  props.entries.reduce((sum, entry) => sum + durationOf(entry), 0)
)

function beginEdit(entry: TimeEntry): void {
  editingId.value = entry.id
  editStart.value = toLocalInput(entry.startedAt)
  editStop.value = entry.stoppedAt ? toLocalInput(entry.stoppedAt) : ''
}

function cancelEdit(): void {
  editingId.value = null
}

async function saveEdit(entry: TimeEntry): Promise<void> {
  const id = editingId.value
  if (!id) return
  const saved = await run(() =>
    store.editTimer(id, {
      startedAt: fromLocalInput(editStart.value),
      stoppedAt: entry.stoppedAt === null ? null : fromLocalInput(editStop.value)
    })
  )
  if (saved !== null) editingId.value = null
}

function beginDelete(entry: TimeEntry): void {
  deletingId.value = entry.id
  deletingSummary.value = formatDuration(durationOf(entry))
}

async function remove(): Promise<void> {
  const id = deletingId.value
  if (!id) return
  await run(() => store.deleteTimer(id), 'Session deleted')
  deletingId.value = null
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && !deletingId.value) {
    event.stopPropagation()
    emit('close')
    return
  }
  if (panel.value && !deletingId.value) trapTabKey(panel.value, event)
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
      :aria-label="`Time on ${task.title}`"
      data-testid="time-log-dialog"
    >
      <header class="flex items-center gap-3 border-b border-border px-6 py-4">
        <span class="min-w-0 flex-1">
          <span class="block text-[15px] font-semibold tracking-[-0.01em] text-text">
            Time on {{ task.title }}
          </span>
          <span class="block truncate font-mono text-[11px] text-anchor/80">{{ task.anchor }}</span>
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
        <p v-if="!entries.length" class="py-6 text-center text-[13px] text-text-subtle">
          No sessions yet.
        </p>

        <div v-for="group in groups" :key="group.label" class="mb-4 last:mb-0">
          <p class="mb-1.5 eyebrow">
            {{ group.label }}
          </p>

          <div class="relative pl-3.5">
            <span class="absolute inset-y-1 left-1 w-px bg-border-strong" aria-hidden="true" />
            <div
              v-for="entry in group.entries"
              :key="entry.id"
              class="relative rounded-[var(--radius-control)] border border-transparent px-2.5 py-2 hover:border-border-strong hover:bg-surface-raised"
              data-testid="time-log-row"
            >
              <span
                class="absolute left-[-9px] top-1/2 size-1.5 -translate-y-1/2 rounded-full border-2 border-canvas"
                :class="entry.stoppedAt ? 'bg-text-subtle' : 'bg-accent'"
                aria-hidden="true"
              />
            <div v-if="editingId !== entry.id" class="flex items-center gap-2.5">
              <span class="min-w-0 flex-1 font-mono text-[12.5px]" :class="entry.stoppedAt ? 'text-text' : 'text-accent'">
                {{ timeOf(entry.startedAt) }} &ndash;
                <span v-if="entry.stoppedAt">{{ timeOf(entry.stoppedAt) }}</span>
                <span v-else class="inline-flex items-center gap-1">
                  running
                  <span class="size-1.5 animate-pulse rounded-full bg-accent" aria-hidden="true" />
                </span>
              </span>
              <span class="shrink-0 font-mono text-[12px] text-text-muted">
                {{ formatDuration(durationOf(entry)) }}
              </span>
              <button
                class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-accent-soft hover:text-accent"
                :aria-label="`Edit this session`"
                @click="beginEdit(entry)"
              >
                <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                  <path
                    d="M11.3 2.7a1.5 1.5 0 0 1 2.1 2.1L5.8 12.4l-2.9.7.7-2.9 7.7-7.5Z"
                    stroke="currentColor"
                    stroke-width="1.3"
                    stroke-linejoin="round"
                  />
                </svg>
              </button>
              <button
                v-if="entry.stoppedAt"
                class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
                aria-label="Delete this session"
                @click="beginDelete(entry)"
              >
                &times;
              </button>
            </div>

            <div v-else class="flex flex-wrap items-center gap-2">
              <input
                v-model="editStart"
                type="datetime-local"
                class="focus-ring h-8 rounded-[var(--radius-control)] border border-border bg-canvas px-2 text-[12px] text-text outline-none transition-colors hover:border-border-strong focus:border-accent"
              />
              <span class="text-text-subtle">&ndash;</span>
              <input
                v-if="entry.stoppedAt"
                v-model="editStop"
                type="datetime-local"
                class="focus-ring h-8 rounded-[var(--radius-control)] border border-border bg-canvas px-2 text-[12px] text-text outline-none transition-colors hover:border-border-strong focus:border-accent"
              />
              <span v-else class="text-[12px] text-accent">running</span>
              <span class="ml-auto flex gap-1.5">
                <AppButton variant="ghost" size="sm" @click="cancelEdit">Cancel</AppButton>
                <AppButton variant="primary" size="sm" @click="saveEdit(entry)">Save</AppButton>
              </span>
            </div>
            </div>
          </div>
        </div>
      </div>

      <footer class="flex items-center justify-between border-t border-border bg-canvas px-6 py-3.5">
        <span class="eyebrow text-[11px] font-normal">Total tracked</span>
        <span class="font-mono text-[16px] font-semibold text-accent">
          {{ formatDuration(totalSeconds) }}
        </span>
      </footer>
    </div>

    <AppConfirm
      v-if="deletingId"
      title="Delete this session?"
      body="Removes it from this task's recap."
      :blast="`${deletingSummary} tracked · not recoverable`"
      confirm-label="Delete session"
      @cancel="deletingId = null"
      @confirm="remove"
    />
  </div>
</template>
