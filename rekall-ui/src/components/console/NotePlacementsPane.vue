<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import NotePlacementRow from '@/components/console/NotePlacementRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { groupByProject, matchesTaskQuery } from '@/common/catalog/task-search'
import type { Task } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * The middle column while browsing notes: where the selected note is, and where else it could
 * be. It replaces the task's own column there, so picking a note never drags a task into view.
 * The list is the note's tasks grouped by project; typing widens it to every task that matches,
 * with the ones the note is already on ticked and first in their project, and a click or Enter
 * flips membership either way.
 * Writes go through `store.attachNoteToTask` / `store.detachNoteFromTask`, which keep the rule
 * that a note is on at least one task.
 */
const store = useConsoleStore()
const { selectedDocument, tasks, companies } = storeToRefs(store)
const { run } = useAsyncAction()

interface PlacementRow {
  readonly task: Task
  readonly attached: boolean
  /** True when this is the only task the note is on: it cannot be taken off. */
  readonly onlyHere: boolean
}

const filter = ref('')
const highlighted = ref(0)
const busy = ref<TaskId | null>(null)
const showDone = ref(false)
const list = ref<HTMLElement | null>(null)

const searching = computed(() => filter.value.trim().length > 0)
const manyCompanies = computed(() => companies.value.length > 1)

const attachedIds = computed(
  () => new Set(selectedDocument.value?.tasks.map((ref) => ref.id) ?? [])
)

const attachedTasks = computed(() =>
  tasks.value.filter((task) => attachedIds.value.has(task.id))
)

const liveAttached = computed(() => attachedTasks.value.filter((task) => task.status !== 'DONE'))
const doneAttached = computed(() => attachedTasks.value.filter((task) => task.status === 'DONE'))

/** Browsing: the tasks it is on. Searching: every task that matches, the ones it is on first in their project. */
const rows = computed<PlacementRow[]>(() => {
  const build = (task: Task): PlacementRow => {
    const attached = attachedIds.value.has(task.id)
    return { task, attached, onlyHere: attached && attachedIds.value.size === 1 }
  }
  if (!searching.value) {
    const shown = showDone.value || !liveAttached.value.length ? attachedTasks.value : liveAttached.value
    return shown.map(build)
  }
  const hits = tasks.value.filter((task) => matchesTaskQuery(task, filter.value)).map(build)
  return [...hits.filter((row) => row.attached), ...hits.filter((row) => !row.attached)]
})

const groups = computed(() => groupByProject(rows.value, (row) => row.task))

/** The flat order the arrow keys walk, which is the order the groups render in. */
const walk = computed(() => groups.value.flatMap((group) => group.rows))

watch(walk, (next) => {
  if (highlighted.value >= next.length) highlighted.value = Math.max(0, next.length - 1)
})

watch(filter, () => {
  highlighted.value = 0
})

watch(selectedDocument, () => {
  filter.value = ''
  showDone.value = false
})

function walkIndex(taskId: TaskId): number {
  return walk.value.findIndex((row) => row.task.id === taskId)
}

async function toggle(row: PlacementRow): Promise<void> {
  const document = selectedDocument.value
  if (!document || busy.value !== null || row.onlyHere) return
  busy.value = row.task.id
  try {
    if (row.attached) {
      await run(
        () => store.detachNoteFromTask(document.id, row.task.id),
        `Taken off ${row.task.projectLabel}/${row.task.label}.`
      )
    } else {
      await run(
        () => store.attachNoteToTask(document.id, row.task.id),
        `Put on ${row.task.projectLabel}/${row.task.label}.`
      )
    }
  } finally {
    busy.value = null
  }
}

/** The one deliberate way out: the task opens on the Tasks side, with this note still in view. */
function openTask(task: Task): void {
  store.setNavMode('tasks')
  store.selectTask(task.id)
  if (selectedDocument.value) store.selectDocument(selectedDocument.value.id)
}

function scrollHighlightedIntoView(): void {
  const row = list.value?.querySelector<HTMLElement>(`[data-walk-index="${highlighted.value}"]`)
  row?.scrollIntoView({ block: 'nearest' })
}

function onFilterKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    if (!filter.value) return
    event.preventDefault()
    filter.value = ''
    return
  }
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    if (!walk.value.length) return
    const delta = event.key === 'ArrowDown' ? 1 : -1
    highlighted.value = (highlighted.value + delta + walk.value.length) % walk.value.length
    scrollHighlightedIntoView()
    return
  }
  if (event.key === 'Enter') {
    const row = walk.value[highlighted.value]
    if (!row) return
    event.preventDefault()
    void toggle(row)
  }
}
</script>

<template>
  <section
    class="flex min-h-0 w-(--spacing-notelist) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Tasks the selected note is on"
    data-testid="note-placements"
  >
    <header class="flex h-(--spacing-header) shrink-0 items-center gap-2 border-b border-border px-3.5">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13.5px] font-semibold text-text">
          {{ selectedDocument?.title ?? 'Notes' }}
        </span>
        <span v-if="selectedDocument" class="block truncate text-[10.5px] text-text-subtle">
          On {{ attachedIds.size }} task{{ attachedIds.size === 1 ? '' : 's' }}
        </span>
      </span>
      <span
        v-if="selectedDocument"
        class="shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted"
      >
        {{ selectedDocument.kind }}
      </span>
    </header>

    <div v-if="!selectedDocument" class="px-4 py-4 text-[12.5px] leading-relaxed text-text-subtle">
      <p>Pick a note to see which tasks it is on.</p>
      <p class="mt-2">
        Or press
        <kbd class="rounded border border-border px-1 font-mono text-[10px]">N</kbd>
        to start a new one.
      </p>
    </div>

    <template v-else>
      <div class="shrink-0 border-b border-border px-2.5 py-2">
        <label class="sr-only" for="note-placements-filter">Find a task to put this note on</label>
        <div class="relative">
          <svg
            class="pointer-events-none absolute left-2.5 top-1/2 size-3 -translate-y-1/2 text-text-subtle"
            viewBox="0 0 12 12"
            fill="none"
            aria-hidden="true"
          >
            <circle cx="5.2" cy="5.2" r="3.6" stroke="currentColor" stroke-width="1.3" />
            <path d="M8 8l2.6 2.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
          <input
            id="note-placements-filter"
            v-model="filter"
            type="search"
            autocomplete="off"
            spellcheck="false"
            placeholder="Put it on a task"
            class="focus-ring h-8 w-full rounded-[var(--radius-control)] border border-border bg-canvas pl-7 pr-2.5 text-[12px] text-text placeholder:text-text-subtle"
            data-testid="note-placements-filter"
            @keydown="onFilterKeydown"
          />
        </div>
      </div>

      <div ref="list" class="min-h-0 flex-1 overflow-y-auto py-1.5" data-testid="note-placements-list">
        <p
          v-if="searching && !walk.length"
          class="px-4 py-3 text-[12px] leading-relaxed text-text-subtle"
          data-testid="note-placements-empty"
        >
          No task matches that.
        </p>

        <div
          v-for="group in groups"
          :key="group.projectId"
          class="placement-group mx-2 mb-2"
          :style="{ '--identity-color': identityHue(group.projectId).base }"
        >
          <div class="flex items-baseline gap-1.5 px-2 pb-1 pt-1.5">
            <span class="truncate text-[11px] font-medium text-text-muted">{{ group.projectTitle }}</span>
            <span v-if="manyCompanies" class="truncate text-[10px] text-text-subtle">
              {{ group.companyName }}
            </span>
          </div>

          <NotePlacementRow
            v-for="row in group.rows"
            :key="row.task.id"
            :task="row.task"
            :attached="row.attached"
            :locked="row.onlyHere"
            :busy="busy === row.task.id"
            :highlighted="searching && walkIndex(row.task.id) === highlighted"
            :walk-index="walkIndex(row.task.id)"
            openable
            @hover="highlighted = walkIndex(row.task.id)"
            @toggle="toggle(row)"
            @open="openTask(row.task)"
          />
        </div>

        <button
          v-if="!searching && doneAttached.length && liveAttached.length"
          type="button"
          class="focus-ring mx-4 mt-1 inline-flex items-center gap-1.5 rounded-full border border-border-strong bg-surface-raised py-0.5 pl-2 pr-2.5 font-mono text-[11px] text-text-muted transition-colors hover:text-text"
          :aria-expanded="showDone"
          data-testid="note-placements-done-toggle"
          @click="showDone = !showDone"
        >
          <svg
            class="size-2.5 transition-transform"
            :class="showDone && 'rotate-90'"
            viewBox="0 0 24 24"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M9 6l6 6-6 6"
              stroke="currentColor"
              stroke-width="2.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          {{ doneAttached.length }} done
        </button>
      </div>

      <div class="shrink-0 border-t border-border bg-canvas px-3.5 py-2 text-[10.5px] text-text-subtle">
        <template v-if="searching">
          <kbd class="rounded border border-border px-1 font-mono text-[9.5px]">↑↓</kbd>
          move,
          <kbd class="rounded border border-border px-1 font-mono text-[9.5px]">↵</kbd>
          put on or take off
        </template>
        <template v-else>Type to find a task. Click one it is on to take it off.</template>
      </div>
    </template>
  </section>
</template>

<style scoped>
.placement-group {
  border-left: 2px solid var(--identity-color, var(--color-border-strong));
  padding-left: 2px;
}
</style>
