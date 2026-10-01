<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import NotePlacementRow from '@/components/console/NotePlacementRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useSettledOrder } from '@/composables/useSettledOrder'
import { identityHue } from '@/common/identity'
import { groupByProject, matchesTaskQuery } from '@/common/catalog/task-search'
import {
  GLOBAL_SCOPE,
  NOTE_SCOPE_KIND_LABEL,
  scopeAdmits,
  scopeName,
  type NoteScope,
  type NoteScopeKind
} from '@/model/note-scope'
import type { Task } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

/**
 * The middle column while a note is being started from the Notes side: a name, where it lives,
 * and, optionally, the tasks inside that scope it goes on. It starts in the project of the task
 * in view (or the one the scope picker shows); a note on no task is fine, its scope keeps it.
 * Create sends the lot through `store.createNote`, and the new note opens in the editor.
 */
const store = useConsoleStore()
const { tasks, projects, companies, visibleTasks, selectedTask, scopeProject, scopeCompany } = storeToRefs(store)
const { run, isRunning } = useAsyncAction()

interface ComposerRow {
  readonly task: Task
  readonly attached: boolean
}

const SCOPE_KINDS: readonly NoteScopeKind[] = ['PROJECT', 'COMPANY', 'GLOBAL']

const SCOPE_HINT: Readonly<Record<NoteScopeKind, string>> = {
  PROJECT: 'Listed with this project, and goes only on its tasks.',
  COMPANY: 'Shared by every project of this company.',
  GLOBAL: 'Listed everywhere, and goes on any task.'
}

const startProject = selectedTask.value?.projectId ?? scopeProject.value
const startCompany =
  projects.value.find((project) => project.id === startProject)?.companyId ??
  scopeCompany.value ??
  (companies.value.length === 1 ? companies.value[0]!.id : null)

const title = ref('')
const filter = ref('')
const highlighted = ref(0)
const scopeKind = ref<NoteScopeKind>(startProject ? 'PROJECT' : startCompany ? 'COMPANY' : 'GLOBAL')
const ownerProject = ref<ProjectId | null>(startProject)
const ownerCompany = ref<CompanyId | null>(startCompany)
const picked = ref<Set<TaskId>>(new Set(selectedTask.value ? [selectedTask.value.id] : []))
const titleField = ref<HTMLInputElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const list = ref<HTMLElement | null>(null)

/** The scope as chosen, or null while Project or Company still waits for which one. */
const scope = computed<NoteScope | null>(() => {
  if (scopeKind.value === 'GLOBAL') return GLOBAL_SCOPE
  if (scopeKind.value === 'COMPANY') return ownerCompany.value ? { kind: 'COMPANY', id: ownerCompany.value } : null
  return ownerProject.value ? { kind: 'PROJECT', id: ownerProject.value } : null
})

const projectOptions = computed(() =>
  projects.value.map((project) => ({ value: project.id as string, label: `${project.title}  ·  ${project.companyName}` }))
)
const companyOptions = computed(() =>
  companies.value.map((company) => ({ value: company.id as string, label: company.name }))
)

const ownerProjectChoice = computed<string | null>({
  get: () => ownerProject.value,
  set: (value) => (ownerProject.value = (value || null) as ProjectId | null)
})
const ownerCompanyChoice = computed<string | null>({
  get: () => ownerCompany.value,
  set: (value) => (ownerCompany.value = (value || null) as CompanyId | null)
})

const scopeLabel = computed(() => (scope.value ? scopeName(scope.value, projects.value, companies.value) : '…'))

function admitted(task: Task): boolean {
  const project = projects.value.find((candidate) => candidate.id === task.projectId)
  return scope.value !== null && project !== undefined && scopeAdmits(scope.value, project)
}

// A tick outside the scope would be refused on create, so a narrower scope lets it go.
watch(scope, () => {
  const kept = [...picked.value].filter((id) => {
    const task = tasks.value.find((candidate) => candidate.id === id)
    return task !== undefined && admitted(task)
  })
  if (kept.length !== picked.value.size) picked.value = new Set(kept)
})

const searching = computed(() => filter.value.trim().length > 0)
const manyCompanies = computed(() => companies.value.length > 1)
const canCreate = computed(() => scope.value !== null && !isRunning.value)

/**
 * Not searching: the tasks already ticked, then the live tasks in view as a place to start.
 * Searching: every task in the scope that matches. Ticked ones come first inside their project
 * as the list is drawn, and a tick after that leaves every row where it is.
 */
const rankedRows = computed<ComposerRow[]>(() => {
  const build = (task: Task): ComposerRow => ({ task, attached: picked.value.has(task.id) })
  const inScope = tasks.value.filter(admitted)
  const pool = searching.value
    ? inScope.filter((task) => matchesTaskQuery(task, filter.value))
    : inScope.filter(
        (task) =>
          picked.value.has(task.id) ||
          (task.status !== 'DONE' && visibleTasks.value.some((visible) => visible.id === task.id))
      )
  const built = pool.map(build)
  return [...built.filter((row) => row.attached), ...built.filter((row) => !row.attached)]
})

const rows = useSettledOrder(() => rankedRows.value, (row) => row.task.id, [filter, scope])

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
  const owner = scope.value
  if (!canCreate.value || !owner) return
  const onTasks = tasks.value.filter((task) => picked.value.has(task.id)).map((task) => task.id)
  await run(() => store.createNote({ scope: owner, taskIds: onTasks, title: title.value }), 'Note created')
}

function cancel(): void {
  store.closeNoteComposer()
}

function scrollHighlightedIntoView(): void {
  const row = list.value?.querySelector<HTMLElement>(`[data-walk-index="${highlighted.value}"]`)
  row?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
}

/** Cmd/Ctrl+Enter creates from anywhere in the composer. */
function onComposerKeydown(event: KeyboardEvent): void {
  if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault()
    void create()
  }
}

/** Enter on the name creates: the scope is already chosen, and tasks are optional. */
function onTitleKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    cancel()
    return
  }
  if (event.key === 'Enter' && !event.metaKey && !event.ctrlKey) {
    event.preventDefault()
    void create()
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
          Name it, say where it lives
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

    <div class="shrink-0 border-b border-border px-3.5 py-3" data-testid="note-composer-scope">
      <p class="eyebrow mb-1.5 text-[11px]">Lives in</p>
      <div class="grid grid-cols-3 gap-0.5 rounded-[8px] bg-canvas p-0.5" role="radiogroup" aria-label="Where the note lives">
        <button
          v-for="kind in SCOPE_KINDS"
          :key="kind"
          type="button"
          role="radio"
          class="focus-ring h-7 rounded-[6px] text-[12px] transition-colors"
          :class="scopeKind === kind ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
          :aria-checked="scopeKind === kind"
          :data-testid="`note-composer-scope-${kind.toLowerCase()}`"
          @click="scopeKind = kind"
        >
          {{ NOTE_SCOPE_KIND_LABEL[kind] }}
        </button>
      </div>
      <AppSelect
        v-if="scopeKind === 'PROJECT'"
        v-model="ownerProjectChoice"
        class="mt-2"
        :options="projectOptions"
        placeholder="Pick a project"
        data-testid="note-composer-project"
      />
      <AppSelect
        v-else-if="scopeKind === 'COMPANY'"
        v-model="ownerCompanyChoice"
        class="mt-2"
        :options="companyOptions"
        placeholder="Pick a company"
        data-testid="note-composer-company"
      />
      <p class="mt-1.5 text-[11px] leading-snug text-text-subtle">{{ SCOPE_HINT[scopeKind] }}</p>
    </div>

    <div class="shrink-0 border-b border-border px-2.5 py-2">
      <p class="eyebrow mb-1.5 px-1 text-[11px]">
        On tasks <span class="font-normal normal-case text-text-subtle">· optional</span>
      </p>
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
        <template v-if="!scope">Pick where it lives to see its tasks.</template>
        <template v-else-if="searching">No task in {{ scopeLabel }} matches that.</template>
        <template v-else>Type to find a task in {{ scopeLabel }}, or create it on none.</template>
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
        In <span class="text-text-muted">{{ scopeLabel }}</span>
        <template v-if="picked.size"> · on {{ picked.size }} task{{ picked.size === 1 ? '' : 's' }}</template>
        <span class="text-text-subtle/70"> · </span>
        <kbd class="rounded border border-border px-1 font-mono text-[9.5px]">⌘↵</kbd>
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
