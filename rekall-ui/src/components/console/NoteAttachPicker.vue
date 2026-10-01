<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { excerpt } from '@/common/format/excerpt'
import type { RekallDocument } from '@/model/catalog'
import { scopeAdmits, scopeName } from '@/model/note-scope'
import type { DocumentId, TaskId } from '@/model/branded'

/**
 * The shortcut under "New note": the notes this task may carry and does not yet, hung from the
 * button that opened it. One click or Enter puts the chosen note on the task, opens it and closes
 * the panel. Taking a note off stays on the card; `NoteQuickPicker` is the toggle for both ways.
 */
const props = defineProps<{
  taskId: TaskId
  /** The element the panel hangs from: its left edge is the panel's left edge. */
  anchor: HTMLElement
}>()

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { documents, tasks, projects, companies } = storeToRefs(store)
const { run } = useAsyncAction()

const WIDTH = 360
const GUTTER = 8
const PANEL_HEIGHT = 380

const panel = ref<HTMLElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const filter = ref('')
const highlighted = ref(0)
const busy = ref<DocumentId | null>(null)
const position = ref<{ top: number; left: number; width: number; opensUp: boolean }>({
  top: 0,
  left: 0,
  width: 0,
  opensUp: false
})

const taskProject = computed(() => {
  const projectId = tasks.value.find((task) => task.id === props.taskId)?.projectId
  return projects.value.find((project) => project.id === projectId) ?? null
})

/** Notes the task's scope admits that are not on it, the most recently touched first. */
const available = computed<RekallDocument[]>(() => {
  const project = taskProject.value
  if (project === null) return []
  return documents.value
    .filter(
      (document) =>
        scopeAdmits(document.scope, project) && !document.tasks.some((ref) => ref.id === props.taskId)
    )
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
})

const rows = computed<RekallDocument[]>(() => {
  const parts = filter.value.trim().toLowerCase().split(/\s+/).filter(Boolean)
  if (!parts.length) return available.value
  return available.value.filter((document) => {
    const hay = `${document.title} ${document.kind} ${document.bodyMarkdown}`.toLowerCase()
    return parts.every((part) => hay.includes(part))
  })
})

/** Where the note lives and what else holds it, so two notes with one title can be told apart. */
function whereItLives(document: RekallDocument): string {
  const home = scopeName(document.scope, projects.value, companies.value)
  const [first, ...rest] = document.tasks
  if (!first) return `${home}, on no task`
  return rest.length === 0
    ? `${home}, on ${first.projectLabel}/${first.label}`
    : `${home}, on ${document.tasks.length} tasks`
}

watch(filter, () => {
  highlighted.value = 0
})

watch(rows, (next) => {
  if (highlighted.value >= next.length) highlighted.value = Math.max(0, next.length - 1)
})

/** Opens under the button, or above it when the button sits too low for the panel. */
function place(): void {
  const rect = props.anchor.getBoundingClientRect()
  const width = Math.min(WIDTH, window.innerWidth - 2 * GUTTER)
  const left = Math.max(GUTTER, Math.min(rect.left, window.innerWidth - width - GUTTER))
  const opensUp = window.innerHeight - rect.bottom < PANEL_HEIGHT && rect.top > window.innerHeight - rect.bottom
  position.value = {
    top: opensUp ? window.innerHeight - rect.top + 6 : rect.bottom + 6,
    left,
    width,
    opensUp
  }
}

async function attach(document: RekallDocument): Promise<void> {
  if (busy.value !== null) return
  busy.value = document.id
  try {
    const attached = await run(async () => {
      await store.attachNoteToTask(document.id, props.taskId)
      return true
    }, `${document.title} is on this task.`)
    if (!attached) return
    store.selectDocument(document.id)
    emit('close')
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
    void attach(row)
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
  await nextTick()
  filterField.value?.focus()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown, true)
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', place)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="panel"
      class="fixed z-(--z-overlay) flex max-h-[min(380px,calc(100vh-24px))] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      :style="{
        left: `${position.left}px`,
        width: `${position.width}px`,
        ...(position.opensUp ? { bottom: `${position.top}px` } : { top: `${position.top}px` })
      }"
      role="dialog"
      aria-label="Add an existing note to this task"
      data-testid="note-attach-picker"
    >
      <div class="flex items-center gap-2 border-b border-border px-3 py-2">
        <label class="sr-only" for="note-attach-filter">Find a note to add</label>
        <input
          id="note-attach-filter"
          ref="filterField"
          v-model="filter"
          type="search"
          autocomplete="off"
          spellcheck="false"
          placeholder="Find a note to add"
          class="focus-ring h-7 min-w-0 flex-1 rounded-[var(--radius-control)] border border-border bg-canvas px-2.5 text-[12px] text-text placeholder:text-text-subtle"
          data-testid="note-attach-filter"
        />
        <span class="shrink-0 font-mono text-[10.5px] tabular-nums text-text-subtle" data-testid="note-attach-count">
          {{ rows.length }}
        </span>
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto py-1">
        <p
          v-if="!available.length"
          class="px-3.5 py-3 text-[11.5px] leading-relaxed text-text-subtle"
          data-testid="note-attach-empty"
        >
          Every note this task can carry is already on it.
        </p>
        <p
          v-else-if="!rows.length"
          class="px-3.5 py-3 text-[11.5px] leading-relaxed text-text-subtle"
          data-testid="note-attach-empty"
        >
          No note matches that.
        </p>
        <ol v-else>
          <li v-for="(document, index) in rows" :key="document.id">
            <button
              type="button"
              class="focus-ring relative flex w-full flex-col gap-0.5 px-3.5 py-2 text-left transition-colors"
              :class="[index === highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised', busy === document.id && 'opacity-60']"
              :title="`Add ${document.title} to this task`"
              :data-row-index="index"
              data-testid="note-attach-row"
              @mousemove="highlighted = index"
              @click="attach(document)"
            >
              <span class="flex min-w-0 items-center gap-2">
                <span class="min-w-0 flex-1 truncate text-[12.5px] font-medium text-text">{{ document.title }}</span>
                <span
                  class="shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted"
                >
                  {{ document.kind }}
                </span>
              </span>
              <span class="line-clamp-1 text-[11.5px] text-text-subtle">{{ excerpt(document.bodyMarkdown, 90) }}</span>
              <span class="truncate font-mono text-[10.5px] text-anchor/80" data-testid="note-attach-where">
                {{ whereItLives(document) }}
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
          add to this task
        </span>
        <button
          type="button"
          class="focus-ring text-[10.5px] text-text-subtle transition-colors hover:text-text"
          @click="emit('close')"
        >
          Close
        </button>
      </div>
    </div>
  </Teleport>
</template>
