<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { RekallDocument } from '@/model/catalog'
import type { DocumentId, TaskId } from '@/model/branded'

/**
 * Every note in the store, hung under the button that opened it, so a note can be put on the
 * selected task or taken off it without leaving the description or the checklist. The ones on
 * this task sit first; a click or Enter flips membership. The write path is `store.saveNote`,
 * the same one `NoteAssignmentDialog` uses from the note's side.
 */
const props = defineProps<{
  taskId: TaskId
  /** The element the panel hangs from: its right edge is the panel's right edge. */
  anchor: HTMLElement
}>()

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { documents } = storeToRefs(store)
const { run } = useAsyncAction()

const panel = ref<HTMLElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const filter = ref('')
const highlighted = ref(0)
const busy = ref<DocumentId | null>(null)
const position = ref<{ top: number; left: number; width: number }>({ top: 0, left: 0, width: 0 })

const WIDTH = 380
const GUTTER = 8

interface PickerRow {
  readonly document: RekallDocument
  readonly attached: boolean
  /** True when this task is the only one the note is on, so removing it would orphan the note. */
  readonly onlyHere: boolean
  readonly elsewhere: string
}

function isOn(document: RekallDocument): boolean {
  return document.tasks.some((ref) => ref.id === props.taskId)
}

function matches(document: RekallDocument, needle: string): boolean {
  if (!needle) return true
  const hay = `${document.title} ${document.kind} ${document.bodyMarkdown}`.toLowerCase()
  return needle
    .split(/\s+/)
    .every((part) => hay.includes(part))
}

function describeElsewhere(document: RekallDocument, attached: boolean): string {
  const others = document.tasks.filter((ref) => ref.id !== props.taskId)
  if (others.length === 0) return attached ? 'only here' : 'on no task'
  if (others.length === 1) {
    const only = others[0]!
    return `${attached ? 'also ' : ''}on ${only.projectLabel}/${only.label}`
  }
  return `${attached ? 'also ' : ''}on ${others.length} tasks`
}

const rows = computed<PickerRow[]>(() => {
  const needle = filter.value.trim().toLowerCase()
  const byRecency = [...documents.value].sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  const build = (document: RekallDocument): PickerRow => {
    const attached = isOn(document)
    return {
      document,
      attached,
      onlyHere: attached && document.tasks.length === 1,
      elsewhere: describeElsewhere(document, attached)
    }
  }
  const visible = byRecency.filter((document) => matches(document, needle)).map(build)
  return [...visible.filter((row) => row.attached), ...visible.filter((row) => !row.attached)]
})

const attachedCount = computed(() => documents.value.filter(isOn).length)

watch(rows, (next) => {
  if (highlighted.value >= next.length) highlighted.value = Math.max(0, next.length - 1)
})

watch(filter, () => {
  highlighted.value = 0
})

function place(): void {
  const rect = props.anchor.getBoundingClientRect()
  const width = Math.min(WIDTH, window.innerWidth - 2 * GUTTER)
  const left = Math.max(GUTTER, Math.min(rect.right - width, window.innerWidth - width - GUTTER))
  position.value = { top: rect.bottom + 6, left, width }
}

async function toggle(row: PickerRow): Promise<void> {
  if (busy.value !== null || row.onlyHere) return
  const current = row.document.tasks.map((ref) => ref.id)
  const next = row.attached
    ? current.filter((id) => id !== props.taskId)
    : [...current, props.taskId]
  busy.value = row.document.id
  try {
    await run(
      () => store.saveNote(row.document.id, { taskIds: next }),
      row.attached ? `Removed ${row.document.title}.` : `Added ${row.document.title}.`
    )
  } finally {
    busy.value = null
  }
}

function scrollHighlightedIntoView(): void {
  const row = panel.value?.querySelector<HTMLElement>(`[data-row-index="${highlighted.value}"]`)
  row?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    event.stopPropagation()
    if (!rows.value.length) return
    const delta = event.key === 'ArrowDown' ? 1 : -1
    highlighted.value = (highlighted.value + delta + rows.value.length) % rows.value.length
    scrollHighlightedIntoView()
    return
  }
  if (event.key === 'Enter') {
    const row = rows.value[highlighted.value]
    if (!row) return
    event.preventDefault()
    event.stopPropagation()
    void toggle(row)
    return
  }
  // Every other key belongs to the filter field, and to nothing behind the panel.
  event.stopPropagation()
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target as Node | null
  if (!target) return
  if (panel.value?.contains(target) || props.anchor.contains(target)) return
  emit('close')
}

onMounted(async () => {
  place()
  window.addEventListener('keydown', onKeydown, true)
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', place)
  window.addEventListener('scroll', place, true)
  await nextTick()
  filterField.value?.focus()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown, true)
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', place)
  window.removeEventListener('scroll', place, true)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="panel"
      class="note-picker rise fixed z-(--z-overlay) flex max-h-[min(460px,calc(100vh-24px))] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      :style="{ top: `${position.top}px`, left: `${position.left}px`, width: `${position.width}px` }"
      role="dialog"
      aria-label="Notes on this task"
      data-testid="note-picker"
    >
      <div class="flex items-center gap-2 border-b border-border px-3 py-2">
        <label class="sr-only" for="note-picker-filter">Find a note</label>
        <input
          id="note-picker-filter"
          ref="filterField"
          v-model="filter"
          type="search"
          autocomplete="off"
          spellcheck="false"
          placeholder="Find a note"
          class="focus-ring h-7 min-w-0 flex-1 rounded-[var(--radius-control)] border border-border bg-canvas px-2.5 text-[12px] text-text placeholder:text-text-subtle"
          data-testid="note-picker-filter"
        />
        <span
          class="shrink-0 font-mono text-[10.5px] tabular-nums text-text-subtle"
          :title="`${attachedCount} on this task`"
          data-testid="note-picker-count"
        >
          {{ attachedCount }}/{{ documents.length }}
        </span>
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto py-1" data-testid="note-picker-list">
        <p
          v-if="!documents.length"
          class="px-3.5 py-3 text-[11.5px] leading-relaxed text-text-subtle"
          data-testid="note-picker-empty"
        >
          No note yet. Press
          <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd>
          to write the first one on this task.
        </p>
        <p
          v-else-if="!rows.length"
          class="px-3.5 py-3 text-[11.5px] leading-relaxed text-text-subtle"
          data-testid="note-picker-empty"
        >
          No note matches that.
        </p>
        <ol v-else>
          <li
            v-for="(row, index) in rows"
            :key="row.document.id"
            :class="
              index > 0 && row.attached !== rows[index - 1]!.attached && 'mt-1 border-t border-border pt-1'
            "
          >
            <button
              type="button"
              class="note-row focus-ring group/row relative flex w-full items-center gap-2.5 px-3.5 py-1.5 text-left transition-colors"
              :class="[
                index === highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised',
                row.attached && 'note-row-on',
                row.onlyHere ? 'cursor-default' : 'cursor-pointer',
                busy === row.document.id && 'opacity-60'
              ]"
              :aria-pressed="row.attached"
              :aria-disabled="row.onlyHere"
              :title="
                row.onlyHere
                  ? 'This is the only task the note is on. A note needs at least one.'
                  : row.attached
                    ? `Remove ${row.document.title} from this task`
                    : `Add ${row.document.title} to this task`
              "
              :data-row-index="index"
              :data-attached="row.attached"
              data-testid="note-picker-row"
              @mousemove="highlighted = index"
              @click="toggle(row)"
            >
              <span
                class="grid size-4 shrink-0 place-items-center rounded-[5px] border transition-colors"
                :class="
                  row.attached
                    ? 'border-accent-deep bg-accent-soft text-accent'
                    : 'border-border-strong text-transparent group-hover/row:border-accent/60'
                "
                aria-hidden="true"
              >
                <svg class="size-3" viewBox="0 0 24 24" fill="none">
                  <path
                    d="M5 12.5l4.5 4.5L19 7.5"
                    stroke="currentColor"
                    stroke-width="2.6"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </span>
              <span class="flex min-w-0 flex-1 flex-col gap-0.5">
                <span class="flex min-w-0 items-center gap-2">
                  <span
                    class="min-w-0 flex-1 truncate text-[12.5px]"
                    :class="row.attached ? 'text-text' : 'text-text-muted'"
                    :title="row.document.title"
                  >
                    {{ row.document.title }}
                  </span>
                  <span
                    class="shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted"
                  >
                    {{ row.document.kind }}
                  </span>
                </span>
                <span
                  class="truncate font-mono text-[10.5px]"
                  :class="row.onlyHere ? 'text-accent/80' : 'text-text-subtle'"
                  data-testid="note-picker-elsewhere"
                >
                  {{ row.elsewhere }}
                </span>
              </span>
            </button>
          </li>
        </ol>
      </div>

      <div class="flex items-center justify-between gap-2 border-t border-border bg-canvas px-3.5 py-2">
        <span class="text-[10.5px] text-text-subtle">
          <kbd class="rounded border border-border px-1 font-mono text-[9.5px]">↑↓</kbd>
          move,
          <kbd class="rounded border border-border px-1 font-mono text-[9.5px]">↵</kbd>
          add or remove
        </span>
        <button
          type="button"
          class="focus-ring text-[10.5px] text-text-subtle transition-colors hover:text-text"
          @click="emit('close')"
        >
          Done
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.note-picker {
  animation: picker-in 140ms cubic-bezier(0.16, 1, 0.3, 1);
  transform-origin: top right;
}
@keyframes picker-in {
  from {
    opacity: 0;
    transform: translateY(-4px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.note-row-on::before {
  content: '';
  position: absolute;
  left: 4px;
  top: 8px;
  bottom: 8px;
  width: 3px;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--color-accent-strong), var(--color-accent-deep));
}

@media (prefers-reduced-motion: reduce) {
  .note-picker {
    animation: none;
  }
}
</style>
