<script setup lang="ts">
import { computed, ref } from 'vue'
import DiagramTag from '@/components/diagram/DiagramTag.vue'
import ProjectIcon from '@/components/ui/ProjectIcon.vue'
import { relativeTime } from '@/common/format/relative-time'
import { taskDiagrams, type TaskDiagrams } from '@/common/diagram/task-diagrams'
import { useNow } from '@/composables/useNow'
import type { DiagramSummary } from '@/model/diagram'
import type { Project, Task } from '@/model/catalog'
import type { DiagramId, ProjectId, TaskId } from '@/model/branded'
import type { PendingGeneration } from '@/stores/diagram.store'

/**
 * The tasks, grouped by project, each carrying its diagram state: a diagram to open, a session
 * drawing one now, or a way to ask for one. A diagram tied to no task sits apart at the end.
 */
const props = defineProps<{
  summaries: readonly DiagramSummary[]
  byTask: ReadonlyMap<TaskId, readonly DiagramSummary[]>
  generating: ReadonlySet<TaskId>
  projects: readonly Project[]
  tasks: readonly Task[]
  pending: readonly PendingGeneration[]
  selectedId: DiagramId | null
  arrivedId: DiagramId | null
  loaded: boolean
}>()

const emit = defineEmits<{
  select: [id: DiagramId]
  generate: [task: Task]
  openTerminal: [generation: PendingGeneration]
  dismiss: [key: number]
}>()

type Scope = 'all' | 'diagrams'

const now = useNow(60_000)
const filter = ref('')
const scope = ref<Scope>('all')

interface TaskEntry {
  readonly task: Task
  readonly state: TaskDiagrams
  readonly generation: PendingGeneration | null
}

interface Group {
  readonly projectId: ProjectId
  readonly project: Project | null
  readonly entries: TaskEntry[]
  readonly loose: DiagramSummary[]
}

const projectById = computed(() => new Map(props.projects.map((project) => [project.id, project])))
const generationByTask = computed(() => new Map(props.pending.map((generation) => [generation.taskId, generation])))

/** The task the open diagram belongs to, so its other diagrams stay listed under it. */
const selectedTaskId = computed(() => props.summaries.find((summary) => summary.id === props.selectedId)?.taskId ?? null)

const latestUpdate = (entry: TaskEntry): string => entry.state.diagrams[0]?.updatedAt ?? ''
const rank = (entry: TaskEntry): number => (entry.state.phase === 'generating' ? 0 : entry.state.phase === 'ready' ? 1 : 2)

function compareEntries(a: TaskEntry, b: TaskEntry): number {
  return (
    rank(a) - rank(b) ||
    latestUpdate(b).localeCompare(latestUpdate(a)) ||
    Number(a.task.status === 'DONE') - Number(b.task.status === 'DONE') ||
    a.task.title.localeCompare(b.task.title)
  )
}

const groups = computed<Group[]>(() => {
  const needle = filter.value.trim().toLowerCase()
  const matchesTask = (task: Task): boolean =>
    !needle || task.title.toLowerCase().includes(needle) || task.label.toLowerCase().includes(needle)
  const matchesDiagram = (summary: DiagramSummary): boolean =>
    !needle || summary.title.toLowerCase().includes(needle) || summary.question.toLowerCase().includes(needle)

  const byProject = new Map<ProjectId, Group>()
  const groupOf = (projectId: ProjectId): Group => {
    let group = byProject.get(projectId)
    if (!group) {
      group = { projectId, project: projectById.value.get(projectId) ?? null, entries: [], loose: [] }
      byProject.set(projectId, group)
    }
    return group
  }

  for (const task of props.tasks) {
    if (!matchesTask(task)) continue
    const state = taskDiagrams(task.id, props.byTask, props.generating)
    if (scope.value === 'diagrams' && state.phase === 'none') continue
    groupOf(task.projectId).entries.push({ task, state, generation: generationByTask.value.get(task.id) ?? null })
  }
  for (const summary of props.summaries) {
    if (!summary.taskId && matchesDiagram(summary)) groupOf(summary.projectId).loose.push(summary)
  }

  const shown = [...byProject.values()].filter((group) => group.entries.length + group.loose.length > 0)
  for (const group of shown) group.entries.sort(compareEntries)
  return shown.sort((a, b) => (a.project?.title ?? '').localeCompare(b.project?.title ?? ''))
})

function diagramCount(group: Group): number {
  return group.entries.filter((entry) => entry.state.phase !== 'none').length + group.loose.length
}

/** Asked for from the row: it opens what exists, goes to the session drawing it, or asks for one. */
function activate(entry: TaskEntry): void {
  if (entry.generation && entry.state.phase === 'generating') return emit('openTerminal', entry.generation)
  const latest = entry.state.diagrams[0]
  if (latest) return emit('select', latest.id)
  emit('generate', entry.task)
}

function isSelected(entry: TaskEntry): boolean {
  return selectedTaskId.value === entry.task.id
}

function hint(entry: TaskEntry): string {
  if (entry.state.phase === 'generating') return entry.generation?.request ?? 'A session is drawing it'
  const latest = entry.state.diagrams[0]
  return latest ? `${latest.nodeCount} elements · ${relativeTime(latest.updatedAt, now.value)}` : 'No diagram yet'
}
</script>

<template>
  <aside class="flex w-(--spacing-nav) shrink-0 flex-col border-r border-border bg-surface/60" aria-label="Diagram tasks">
    <div class="flex flex-col gap-2 border-b border-border p-3">
      <label class="relative block">
        <span class="sr-only">Filter tasks</span>
        <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-text-subtle" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><circle cx="7" cy="7" r="4.2" /><path d="m10.2 10.2 3.3 3.3" /></svg>
        <input v-model="filter" type="search" placeholder="Find a task" class="field h-8 w-full rounded-[var(--radius-control)] pl-8 pr-2.5 text-[12.5px] text-text" data-testid="diagram-filter" />
      </label>
      <div class="grid grid-cols-2 gap-1 rounded-[var(--radius-control)] bg-canvas p-0.5" role="group" aria-label="Which tasks">
        <button
          v-for="option in [{ value: 'all', label: 'All tasks' }, { value: 'diagrams', label: 'With a diagram' }] as const"
          :key="option.value"
          class="focus-ring h-7 rounded-[7px] text-[12px] transition-colors"
          :class="scope === option.value ? 'bg-surface-raised font-medium text-text' : 'text-text-subtle hover:text-text'"
          :aria-pressed="scope === option.value"
          :data-testid="`diagram-scope-${option.value}`"
          @click="scope = option.value"
        >
          {{ option.label }}
        </button>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-16 pt-2">
      <div v-if="!loaded" class="flex flex-col gap-2 p-1" role="status" aria-busy="true">
        <span class="sr-only">Loading tasks</span>
        <span v-for="index in 4" :key="index" class="skeleton h-[52px] rounded-[var(--radius-control)]" />
      </div>

      <p v-else-if="groups.length === 0 && (filter || scope === 'diagrams')" class="px-3 py-6 text-center text-[12px] leading-relaxed text-text-subtle">
        {{ filter ? `Nothing matches “${filter}”.` : 'No task has a diagram yet.' }}
      </p>

      <p v-else-if="groups.length === 0" class="px-3 py-6 text-center text-[12px] leading-relaxed text-text-subtle">
        No tasks yet. A diagram is drawn from a task.
      </p>

      <section v-for="group in groups" :key="group.projectId" class="mb-3" :aria-label="group.project?.title ?? 'Project'">
        <h3 class="flex h-7 items-center gap-2 px-2.5 text-[11.5px] font-medium text-text-subtle">
          <ProjectIcon :icon="group.project?.icon ?? 'folder'" :size="13" />
          <span class="min-w-0 flex-1 truncate">{{ group.project?.title ?? 'Unknown project' }}</span>
          <span class="font-mono tabular-nums" :title="`${diagramCount(group)} with a diagram`">{{ diagramCount(group) }}</span>
        </h3>

        <div v-for="entry in group.entries" :key="entry.task.id" class="mb-0.5">
          <button
            class="focus-ring group/row block w-full rounded-[var(--radius-control)] px-3 py-2 text-left transition-colors"
            :class="[
              isSelected(entry) ? 'selected-row' : 'hover:bg-surface-raised',
              entry.state.diagrams.some((diagram) => diagram.id === arrivedId) ? 'row-arrived' : ''
            ]"
            :aria-current="isSelected(entry)"
            :data-testid="entry.state.phase === 'none' ? 'diagram-task-bare' : 'diagram-task'"
            @click="activate(entry)"
          >
            <span class="flex items-center gap-2">
              <span class="min-w-0 flex-1 truncate text-[13px] font-medium" :class="entry.state.phase === 'none' ? 'text-text-muted' : 'text-text'">{{ entry.task.title }}</span>
              <DiagramTag :phase="entry.state.phase" :count="entry.state.diagrams.length" />
            </span>
            <span class="mt-1 flex items-center gap-2">
              <span class="anchor-chip min-w-0 shrink truncate px-1.5 py-px text-[9.5px] leading-[15px]">{{ entry.task.label }}</span>
              <span class="min-w-0 flex-1 truncate text-right text-[11px] text-text-subtle" :class="entry.state.phase === 'ready' && 'tabular-nums'">
                <template v-if="entry.state.phase === 'none'">
                  <span class="group-hover/row:hidden group-focus-visible/row:hidden">{{ hint(entry) }}</span>
                  <span class="hidden font-medium text-accent group-hover/row:inline group-focus-visible/row:inline">Generate a diagram</span>
                </template>
                <template v-else>{{ hint(entry) }}</template>
              </span>
            </span>
          </button>

          <div v-if="entry.state.phase === 'generating' && entry.generation" class="mb-1 flex items-center gap-1 px-3 pt-0.5 text-[11px] text-text-subtle">
            <button class="focus-ring rounded underline-offset-2 hover:text-text hover:underline" @click="emit('openTerminal', entry.generation)">Open its terminal</button>
            <span aria-hidden="true">·</span>
            <button class="focus-ring rounded underline-offset-2 hover:text-text hover:underline" aria-label="Stop waiting for this diagram" @click="emit('dismiss', entry.generation.key)">Stop waiting</button>
          </div>

          <ul v-if="isSelected(entry) && entry.state.diagrams.length > 1" class="mb-1 ml-4 border-l border-border pl-2" aria-label="Diagrams of this task">
            <li v-for="diagram in entry.state.diagrams" :key="diagram.id">
              <button
                class="focus-ring flex w-full items-baseline gap-2 rounded-md px-2 py-1 text-left text-[12px] transition-colors hover:bg-surface-raised"
                :class="diagram.id === selectedId ? 'text-text' : 'text-text-muted'"
                :aria-current="diagram.id === selectedId"
                data-testid="diagram-row"
                @click="emit('select', diagram.id)"
              >
                <span class="min-w-0 flex-1 truncate">{{ diagram.title }}</span>
                <span class="shrink-0 text-[10.5px] tabular-nums text-text-subtle">{{ relativeTime(diagram.updatedAt, now) }}</span>
              </button>
            </li>
          </ul>
        </div>

        <template v-if="group.loose.length > 0">
          <h4 class="mt-2 px-2.5 text-[11px] text-text-subtle">Not tied to a task</h4>
          <button
            v-for="summary in group.loose"
            :key="summary.id"
            class="focus-ring mb-0.5 block w-full rounded-[var(--radius-control)] px-3 py-2 text-left transition-colors"
            :class="[selectedId === summary.id ? 'selected-row' : 'hover:bg-surface-raised', arrivedId === summary.id ? 'row-arrived' : '']"
            :aria-current="selectedId === summary.id"
            data-testid="diagram-row"
            @click="emit('select', summary.id)"
          >
            <span class="flex items-baseline gap-2">
              <span class="min-w-0 flex-1 truncate text-[13px] font-medium text-text">{{ summary.title }}</span>
              <span class="shrink-0 text-[11px] tabular-nums text-text-subtle">{{ relativeTime(summary.updatedAt, now) }}</span>
            </span>
            <span class="mt-1 block font-mono text-[10.5px] tabular-nums text-text-subtle">{{ summary.nodeCount }} elements · {{ summary.edgeCount }} relations</span>
          </button>
        </template>
      </section>
    </div>
  </aside>
</template>

<style scoped>
/* A diagram that just arrived from a session announces itself once, then is a row like any other. */
.row-arrived {
  animation: row-arrived 1.4s ease-out 1;
}

@keyframes row-arrived {
  0% {
    box-shadow: 0 0 0 1px var(--color-accent), 0 0 18px -4px var(--color-accent);
  }
  100% {
    box-shadow: 0 0 0 1px transparent, 0 0 18px -4px transparent;
  }
}
</style>
