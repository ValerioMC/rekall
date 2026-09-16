<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import NotePlacementRow from '@/components/console/NotePlacementRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { groupByProject, matchesTaskQuery } from '@/common/catalog/task-search'
import type { Task } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * The middle column while a note is being started from the Notes side. A note is born on at
 * least one task, so the composer is a name and the tasks it goes on, in one place: the task in
 * view is ticked to begin with, the active tasks in scope are offered underneath, and typing
 * widens the list to every task. Create sends the lot through `store.createNote`; the new note
 * opens in the editor and this column goes back to being its placements.
 */
const store = useConsoleStore()
const { tasks, visibleTasks, selectedTaskId, companies } = storeToRefs(store)
const { run, isRunning } = useAsyncAction()

interface ComposerRow {
  readonly task: Task
  readonly attached: boolean
}

const title = ref('')
const filter = ref('')
const highlighted = ref(0)
const picked = ref<Set<TaskId>>(new Set(selectedTaskId.value ? [selectedTaskId.value] : []))
const titleField = ref<HTMLInputElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const list = ref<HTMLElement | null>(null)

const searching = computed(() => filter.value.trim().length > 0)
const manyCompanies = computed(() => companies.value.length > 1)
const canCreate = computed(() => picked.value.size > 0 && !isRunning.value)

/**
 * Not searching: the tasks already ticked, then the live tasks in scope as a place to start.
 * Searching: every task that matches. Ticked ones come first inside their project either way.
 */
const rows = computed<ComposerRow[]>(() => {
  const build = (task: Task): ComposerRow => ({ task, attached: picked.value.has(task.id) })
  const pool = searching.value
    ? tasks.value.filter((task) => matchesTaskQuery(task, filter.value))
    : tasks.value.filter(
        (task) =>
          picked.value.has(task.id) ||
          (task.status !== 'DONE' && visibleTasks.value.some((visible) => visible.id === task.id))
      )
  const built = pool.map(build)
  return [...built.filter((row) => row.attached), ...built.filter((row) => !row.attached)]
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

onMounted(() => titleField.value?.focus())

function walkIndex(taskId: TaskId): number {
  return walk.value.findIndex((row) => row.task.id === taskId)
}

function toggle(task: Task): void {
  const next = new Set(picked.value)
  if (next.has(task.id)) next.delete(task.id)
  else next.add(task.id)
  picked.value = next
}

async function create(): Promise<void> {
  if (!canCreate.value) return
  const onTasks = tasks.value.filter((task) => picked.value.has(task.id)).map((task) => task.id)
  await run(() => store.createNote(onTasks, title.value), 'Note created')
}

function cancel(): void {
  store.closeNoteComposer()
}

function scrollHighlightedIntoView(): void {
  const row = list.value?.querySelector<HTMLElement>(`[data-walk-index="${highlighted.value}"]`)
  row?.scrollIntoView({ block: 'nearest' })
}

/** Cmd/Ctrl+Enter creates from anywhere in the composer. */
function onComposerKeydown(event: KeyboardEvent): void {
  if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault()
    void create()
  }
}

/** Enter on the name creates when a task is ticked, otherwise moves on to picking one. */
function onTitleKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    cancel()
    return
  }
  if (event.key === 'Enter' && !event.metaKey && !event.ctrlKey) {
    event.preventDefault()
    if (picked.value.size) void create()
    else filterField.value?.focus()
  }
}

function onFilterKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    if (filter.value) filter.value = ''
    else cancel()
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
  if (event.key === 'Enter' && !event.metaKey && !event.ctrlKey) {
    const row = walk.value[highlighted.value]
    if (!row) return
    event.preventDefault()
    toggle(row.task)
  }
}
</script>

<template>
  <section
    class="flex min-h-0 w-(--spacing-notelist) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="New note"
    data-testid="note-composer"
    @keydown="onComposerKeydown"
  >
    <header class="flex h-(--spacing-header) shrink-0 items-center gap-2 border-b border-border px-3.5">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13.5px] font-semibold text-text">New note</span>
        <span class="block truncate text-[10.5px] text-text-subtle">
          Name it, tick where it goes
        </span>
      </span>
      <button
        type="button"
        class="focus-ring shrink-0 rounded border border-border bg-surface-raised px-1.5 py-px font-mono text-[9.5px] text-text-muted transition-colors hover:text-text"
        title="Cancel"
        data-testid="note-composer-cancel"
        @click="cancel"
      >
        esc
      </button>
    </header>

    <div class="shrink-0 border-b border-border px-3.5 py-3">
      <label class="sr-only" for="note-composer-title">Note name</label>
      <input
        id="note-composer-title"
        ref="titleField"
        v-model="title"
        type="text"
        autocomplete="off"
        spellcheck="false"
        placeholder="untitled.md"
        class="focus-ring w-full rounded border-0 bg-transparent text-[17px] font-semibold tracking-[-0.015em] text-text outline-none placeholder:text-text-subtle/70"
        data-testid="note-composer-title"
        @keydown="onTitleKeydown"
      />
    </div>

    <div class="shrink-0 border-b border-border px-2.5 py-2">
      <label class="sr-only" for="note-composer-filter">Find a task to put it on</label>
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
          id="note-composer-filter"
          ref="filterField"
          v-model="filter"
          type="search"
          autocomplete="off"
          spellcheck="false"
          placeholder="Put it on a task"
          class="focus-ring h-8 w-full rounded-[var(--radius-control)] border border-border bg-canvas pl-7 pr-2.5 text-[12px] text-text placeholder:text-text-subtle"
          data-testid="note-composer-filter"
          @keydown="onFilterKeydown"
        />
      </div>
    </div>

    <div ref="list" class="min-h-0 flex-1 overflow-y-auto py-1.5" data-testid="note-composer-list">
      <p
        v-if="!walk.length"
        class="px-4 py-3 text-[12px] leading-relaxed text-text-subtle"
        data-testid="note-composer-empty"
      >
        <template v-if="searching">No task matches that.</template>
        <template v-else>Type to find a task. A note lives on at least one.</template>
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
          :highlighted="searching && walkIndex(row.task.id) === highlighted"
          :walk-index="walkIndex(row.task.id)"
          @hover="highlighted = walkIndex(row.task.id)"
          @toggle="toggle(row.task)"
        />
      </div>
    </div>

    <div class="flex shrink-0 items-center gap-2 border-t border-border bg-canvas px-3.5 py-2">
      <span class="min-w-0 flex-1 truncate text-[10.5px] text-text-subtle" data-testid="note-composer-count">
        <template v-if="picked.size">
          On {{ picked.size }} task{{ picked.size === 1 ? '' : 's' }}
          <span class="text-text-subtle/70">·</span>
          <kbd class="rounded border border-border px-1 font-mono text-[9.5px]">⌘↵</kbd>
        </template>
        <template v-else>Tick at least one task</template>
      </span>
      <AppButton
        variant="primary"
        size="sm"
        :disabled="!canCreate"
        :loading="isRunning"
        data-testid="note-composer-create"
        @click="create"
      >
        Create note
      </AppButton>
    </div>
  </section>
</template>

<style scoped>
.placement-group {
  border-left: 2px solid var(--identity-color, var(--color-border-strong));
  padding-left: 2px;
}
</style>
