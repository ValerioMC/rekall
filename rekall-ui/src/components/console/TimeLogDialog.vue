<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useNow } from '@/composables/useNow'
import { formatDuration } from '@/common/format/duration'
import AppDialog from '@/components/ui/AppDialog.vue'
import type { Task, TimeEntry } from '@/model/catalog'
import type { TimeEntryId } from '@/model/branded'

const props = defineProps<{ task: Task; entries: readonly TimeEntry[] }>()
const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { run } = useAsyncAction()
const now = useNow()

const editingId = ref<TimeEntryId | null>(null)
const editStart = ref('')
const editStop = ref('')
const deletingId = ref<TimeEntryId | null>(null)
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

onMounted(async () => {
  await nextTick()
})
</script>

<template>
  <AppDialog
    :label="`Time on ${task.title}`"
    panel-class="flex flex-col max-h-[80vh] max-w-[520px]"
    panel-testid="time-log-dialog"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">
          Time on {{ task.title }}
        </span>
        <span class="block truncate font-mono text-caption text-anchor/80">{{ task.anchor }}</span>
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
      <p v-if="!entries.length" class="py-6 text-center text-body text-text-subtle">
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
            class="relative rounded-control border border-transparent px-2.5 py-2 hover:border-border-strong hover:bg-surface-raised"
            data-testid="time-log-row"
          >
            <!-- Centred on the rail (left-1 of the group, so 4.5px in, 14px left of the row):
                 a 9px disc whose 2px canvas ring cuts the rail around it, leaving 5px of colour. -->
            <span
              class="absolute left-[-14px] top-1/2 size-[9px] -translate-y-1/2 rounded-full border-2 border-surface"
              :class="entry.stoppedAt ? 'bg-text-subtle' : 'bg-accent shadow-[0_0_8px_-1px_var(--color-accent)]'"
              aria-hidden="true"
            />
          <div v-if="editingId !== entry.id" class="flex items-center gap-2.5">
            <span class="min-w-0 flex-1 font-mono text-small" :class="entry.stoppedAt ? 'text-text' : 'text-accent'">
              {{ timeOf(entry.startedAt) }} &ndash;
              <span v-if="entry.stoppedAt">{{ timeOf(entry.stoppedAt) }}</span>
              <span v-else class="inline-flex items-center gap-1">
                running
                <span class="size-1.5 animate-pulse rounded-full bg-accent" aria-hidden="true" />
              </span>
            </span>
            <span class="shrink-0 font-mono text-small text-text-muted">
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
              <CloseGlyph small />
            </button>
          </div>

          <div v-else class="flex flex-wrap items-center gap-2">
            <input
              v-model="editStart"
              type="datetime-local"
              class="field text-text h-8 rounded-control px-2 text-small"
            />
            <span class="text-text-subtle">&ndash;</span>
            <input
              v-if="entry.stoppedAt"
              v-model="editStop"
              type="datetime-local"
              class="field text-text h-8 rounded-control px-2 text-small"
            />
            <span v-else class="text-small text-accent">running</span>
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
      <span class="eyebrow text-caption font-normal">Total tracked</span>
      <span class="font-mono text-title font-semibold text-accent">
        {{ formatDuration(totalSeconds) }}
      </span>
    </footer>
    <template #overlay>
      <Transition name="dialog">
        <AppConfirm
          v-if="deletingId"
          title="Delete this session?"
          body="Removes it from this task's recap."
          :blast="`${deletingSummary} tracked · not recoverable`"
          confirm-label="Delete session"
          @cancel="deletingId = null"
          @confirm="remove"
        />
      </Transition>
    </template>
  </AppDialog>
</template>
