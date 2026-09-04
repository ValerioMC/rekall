<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import ScopePicker from '@/components/console/ScopePicker.vue'
import RecordDialog from '@/components/console/RecordDialog.vue'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import NavigatorFilingDrawer from '@/components/console/NavigatorFilingDrawer.vue'
import ProjectTrace from '@/components/ui/ProjectTrace.vue'
import { partitionTasks } from '@/common/catalog/partition-tasks'
import { useConsoleStore } from '@/stores/console.store'
import {
  TASK_STATUS_COLOR,
  TASK_STATUS_LABEL,
  TASK_STATUS_ORDER,
  TASK_STATUS_RING
} from '@/model/catalog'
import { taskDraft } from '@/model/record-draft'
import type { RecordDraft } from '@/model/record-draft'
import type { Task } from '@/model/catalog'
import type { ProjectId } from '@/model/branded'

/**
 * The navigator, and the one rule that governs its layout: every control sits above the
 * scrolling content and never moves.
 *
 * The previous sidebar listed projects underneath the task groups, so filtering changed their
 * height and the projects slid up and down. A target that moves is a target you have to find
 * again, a hundred times a day.
 */
const store = useConsoleStore()
const {
  scopeCompany,
  scopeProject,
  scopeName,
  navMode,
  visibleTasks,
  visibleDocuments,
  selectedTaskId,
  selectedDocId,
  elsewhere,
  isLoading,
  runningEntries
} = storeToRefs(store)

const runningTaskIds = computed(() => new Set(runningEntries.value.map((entry) => entry.taskId)))

const editing = ref<RecordDraft | null>(null)

/** A task needs a project. Scoped to one, that is the answer; otherwise it has to be picked. */
const projectChoices = computed(() => store.scopedProjects)

/** Above a single project, a flat status list is a wall of look-alike rows; grouped by project it
 *  stays readable at any scale, so grouping is the default everywhere except inside one project. */
const groupByProject = computed(() => navMode.value === 'tasks' && scopeProject.value === null)

interface ProjectGroup {
  readonly projectId: ProjectId
  readonly projectTitle: string
  readonly companyName: string
  readonly tasks: Task[]
  /** The same tasks split into open work and filed work: see {@link partitionTasks}. */
  readonly active: Task[]
  readonly filed: Task[]
}

interface ProjectTaskBucket {
  readonly projectId: ProjectId
  readonly projectTitle: string
  readonly companyName: string
  readonly tasks: Task[]
}

const groupedByProject = computed<ProjectGroup[]>(() => {
  const byProject = new Map<ProjectId, ProjectTaskBucket>()
  for (const task of visibleTasks.value) {
    let bucket = byProject.get(task.projectId)
    if (!bucket) {
      bucket = {
        projectId: task.projectId,
        projectTitle: task.projectTitle,
        companyName: task.companyName,
        tasks: []
      }
      byProject.set(task.projectId, bucket)
    }
    bucket.tasks.push(task)
  }
  return [...byProject.values()]
    .map((bucket) => ({ ...bucket, ...partitionTasks(bucket.tasks) }))
    .sort((a, b) => a.projectTitle.localeCompare(b.projectTitle))
})

/** The status groups the navigator stacks, DONE held back for the filing drawer below them. */
const grouped = computed(() =>
  TASK_STATUS_ORDER.filter((status) => status !== 'DONE')
    .map((status) => ({
      status,
      tasks: visibleTasks.value.filter((task) => task.status === status)
    }))
    .filter((group) => group.tasks.length > 0)
)

/** Every finished task the current scope holds, listed only when the drawer is open. */
const filedInScope = computed(() =>
  visibleTasks.value.filter((task) => task.status === 'DONE')
)

/**
 * Which filing drawers are open, reset on every load.
 *
 * Deliberately not persisted, unlike {@link collapsedProjectIds}: finished work starts out of
 * the way every session and is pulled back only when it is actually wanted. One flag for the
 * status view's single drawer, a set of project ids for the per-project ones.
 */
const showFiledInScope = ref(false)
const revealedProjectIds = ref<Set<ProjectId>>(new Set())

function toggleRevealed(projectId: ProjectId): void {
  const next = new Set(revealedProjectIds.value)
  if (next.has(projectId)) next.delete(projectId)
  else next.add(projectId)
  revealedProjectIds.value = next
}

/**
 * A filed task that gets selected, by search or by walking the list, opens the drawer it is
 * in. A selection you cannot see is one you cannot tell you made.
 */
watch(selectedTaskId, () => {
  const task = store.selectedTask
  if (!task || task.status !== 'DONE') return
  showFiledInScope.value = true
  if (!revealedProjectIds.value.has(task.projectId)) toggleRevealed(task.projectId)
})

const COLLAPSE_KEY = 'rekall.nav.collapsed-projects'

function loadCollapsed(): Set<ProjectId> {
  try {
    const raw = localStorage.getItem(COLLAPSE_KEY)
    const parsed: unknown = raw ? JSON.parse(raw) : []
    return Array.isArray(parsed) ? new Set(parsed as ProjectId[]) : new Set()
  } catch {
    return new Set()
  }
}

const collapsedProjectIds = ref<Set<ProjectId>>(loadCollapsed())

function toggleCollapsed(projectId: ProjectId): void {
  const next = new Set(collapsedProjectIds.value)
  if (next.has(projectId)) next.delete(projectId)
  else next.add(projectId)
  collapsedProjectIds.value = next
  try {
    localStorage.setItem(COLLAPSE_KEY, JSON.stringify([...next]))
  } catch {}
}

/**
 * A new task opens on the project you are already in.
 *
 * When the scope does not name one, the project of the task in view is the next best answer and
 * the first in scope is the last resort. Whichever it lands on, the editor shows it as a field:
 * a guess that cannot be seen is how a task ends up in the wrong project.
 */
function beginCreate(): void {
  const projectId =
    scopeProject.value ?? store.selectedTask?.projectId ?? projectChoices.value[0]?.id
  if (!projectId) return
  editing.value = taskDraft(projectId)
}

function editTask(task: Task): void {
  editing.value = taskDraft(task.projectId, task)
}

/** The keyboard's way in, so editing does not require finding a row with the pointer first. */
function editSelected(): void {
  const task = store.selectedTask
  if (task) editTask(task)
}

defineExpose({ beginCreate, editSelected })
</script>

<template>
  <nav
    class="flex min-h-0 w-(--spacing-nav) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Navigator"
  >
    <!-- Fixed controls. Nothing here shifts when the list below changes. -->
    <div class="flex shrink-0 flex-col gap-2.5 border-b border-border p-2.5">
      <ScopePicker />

      <div class="flex gap-0.5 rounded-[8px] bg-canvas p-0.5" role="group" aria-label="Browse by">
        <button
          v-for="mode in (['tasks', 'notes'] as const)"
          :key="mode"
          class="focus-ring flex h-7 flex-1 items-center justify-center gap-1.5 rounded-[6px] text-[12px] capitalize transition-colors"
          :class="
            navMode === mode
              ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
              : 'text-text-subtle hover:text-text'
          "
          :aria-pressed="navMode === mode"
          @click="navMode = mode"
        >
          {{ mode }}
          <span class="font-mono text-[10.5px] text-text-subtle">
            {{ mode === 'tasks' ? visibleTasks.length : visibleDocuments.length }}
          </span>
        </button>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto pb-4">
      <!-- Shaped like the list it stands in for, so the empty states below never get a chance
           to flash "no project here yet" while the first fetch is still in flight. -->
      <div v-if="isLoading" class="flex flex-col gap-4 p-2 pt-3" aria-hidden="true">
        <div v-for="group in 3" :key="group" class="flex flex-col gap-1.5">
          <div class="skeleton mx-1.5 mb-1 h-2.5 w-16" />
          <div v-for="row in 2" :key="row" class="skeleton h-9 w-full rounded-[var(--radius-control)]" />
        </div>
      </div>

      <template v-else>
      <button
        v-if="navMode === 'tasks' && projectChoices.length"
        data-testid="new-task"
        class="focus-ring m-2 flex w-[calc(100%-16px)] items-center gap-2 rounded-[var(--radius-control)] border border-dashed border-border-strong px-2.5 py-2 text-left text-[12.5px] text-text-muted transition-colors hover:border-solid hover:border-accent hover:bg-accent-soft hover:text-text"
        @click="beginCreate"
      >
        <span class="text-accent">+</span> New task
        <span
          v-if="scopeProject !== null"
          class="ml-auto min-w-0 truncate font-mono text-[10.5px] text-text-subtle"
        >
          in {{ scopeName }}
        </span>
      </button>

      <template v-if="navMode === 'tasks' && groupByProject">
        <div v-for="group in groupedByProject" :key="group.projectId" class="px-2 pt-1.5 first:pt-0.5">
          <button
            class="focus-ring flex w-full items-start gap-2 rounded-[var(--radius-control)] px-1.5 pb-1.5 pt-2 text-left"
            @click="toggleCollapsed(group.projectId)"
          >
            <svg
              class="mt-0.5 size-3 shrink-0 text-text-subtle transition-transform"
              :class="!collapsedProjectIds.has(group.projectId) && 'rotate-90'"
              viewBox="0 0 24 24"
              fill="none"
              aria-hidden="true"
            >
              <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <span class="min-w-0 flex-1">
              <span class="flex items-center gap-2">
                <span class="section-label min-w-0 flex-1 truncate">{{ group.projectTitle }}</span>
                <span
                  class="shrink-0 rounded-full bg-surface-raised px-1.5 py-px font-mono text-[10px] tabular-nums text-text-subtle"
                >
                  {{ group.tasks.length }}
                </span>
              </span>
              <span class="mt-1 flex items-center gap-2">
                <span
                  v-if="scopeCompany === null"
                  class="shrink-0 truncate text-[10px] text-text-subtle"
                >
                  {{ group.companyName }}
                </span>
                <ProjectTrace :id="group.projectId" size="sm" />
              </span>
            </span>
          </button>

          <div v-show="!collapsedProjectIds.has(group.projectId)" class="mt-0.5 flex flex-col gap-0.5">
            <NavigatorTaskRow
              v-for="task in group.active"
              :key="task.id"
              :task="task"
              :selected="task.id === selectedTaskId"
              :running="runningTaskIds.has(task.id)"
              :show-context="false"
              :show-company="false"
              @select="store.selectTask(task.id)"
              @edit="editTask(task)"
            />

            <NavigatorFilingDrawer
              v-if="group.filed.length"
              :count="group.filed.length"
              :open="revealedProjectIds.has(group.projectId)"
              @toggle="toggleRevealed(group.projectId)"
            >
              <NavigatorTaskRow
                v-for="task in group.filed"
                :key="task.id"
                :task="task"
                :selected="task.id === selectedTaskId"
                :running="runningTaskIds.has(task.id)"
                :show-context="false"
                :show-company="false"
                filed
                @select="store.selectTask(task.id)"
                @edit="editTask(task)"
              />
            </NavigatorFilingDrawer>
          </div>
        </div>

        <p v-if="!groupedByProject.length" class="px-4 py-3 text-[12.5px] leading-relaxed text-text-subtle">
          <template v-if="!projectChoices.length">
            No project here yet. Add one from the picker above, then a task can live in it.
          </template>
          <template v-else>No task here yet.</template>
        </p>
      </template>

      <template v-else-if="navMode === 'tasks'">
        <div v-for="group in grouped" :key="group.status" class="px-2">
          <div class="flex items-center gap-2.5 px-1.5 pb-1.5 pt-5">
            <span class="relative grid size-3 shrink-0 place-items-center" aria-hidden="true">
              <span class="absolute size-3 rounded-full" :class="TASK_STATUS_RING[group.status]" />
              <span class="relative size-[7px] rounded-full" :class="TASK_STATUS_COLOR[group.status]" />
            </span>
            <span class="section-label flex-1">{{ TASK_STATUS_LABEL[group.status] }}</span>
            <span
              class="shrink-0 rounded-full bg-surface-raised px-1.5 py-px font-mono text-[10px] tabular-nums text-text-subtle"
            >
              {{ group.tasks.length }}
            </span>
          </div>

          <div class="flex flex-col gap-0.5">
            <NavigatorTaskRow
              v-for="task in group.tasks"
              :key="task.id"
              :task="task"
              :selected="task.id === selectedTaskId"
              :running="runningTaskIds.has(task.id)"
              :show-context="true"
              :show-company="scopeCompany === null"
              @select="store.selectTask(task.id)"
              @edit="editTask(task)"
            />
          </div>
        </div>

        <div v-if="filedInScope.length" class="px-2 pt-2">
          <NavigatorFilingDrawer
            :count="filedInScope.length"
            :open="showFiledInScope"
            @toggle="showFiledInScope = !showFiledInScope"
          >
            <NavigatorTaskRow
              v-for="task in filedInScope"
              :key="task.id"
              :task="task"
              :selected="task.id === selectedTaskId"
              :running="runningTaskIds.has(task.id)"
              :show-context="true"
              :show-company="scopeCompany === null"
              filed
              @select="store.selectTask(task.id)"
              @edit="editTask(task)"
            />
          </NavigatorFilingDrawer>
        </div>

        <p
          v-if="!grouped.length && !filedInScope.length"
          class="px-4 py-3 text-[12.5px] leading-relaxed text-text-subtle"
        >
          <template v-if="!projectChoices.length">
            No project here yet. Add one from the picker above, then a task can live in it.
          </template>
          <template v-else>No task here yet.</template>
        </p>
      </template>

      <template v-else>
        <div class="px-2">
          <button
            v-for="document in visibleDocuments"
            :key="document.id"
            data-testid="note-row"
            class="focus-ring mt-0.5 flex w-full items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-1.5 text-left transition-colors"
            :class="
              document.id === selectedDocId
                ? 'selected-row text-text'
                : 'text-text-muted hover:bg-surface-raised hover:text-text'
            "
            :aria-current="document.id === selectedDocId"
            @click="store.selectDocument(document.id)"
          >
            <span class="min-w-0 flex-1">
              <span class="block truncate text-[13px]">{{ document.title }}</span>
              <span class="block truncate font-mono text-[10.5px] text-anchor/80">
                {{ document.tasks.map((t) => t.label).join(', ') }}
              </span>
            </span>
            <span
              v-if="document.tasks.length > 1"
              class="anchor-chip shrink-0 px-1.5 py-px text-[9.5px]"
              :title="`On ${document.tasks.length} tasks`"
            >
              &#8942; {{ document.tasks.length }}
            </span>
          </button>
        </div>

        <p
          v-if="!visibleDocuments.length"
          class="px-4 py-3 text-[12.5px] leading-relaxed text-text-subtle"
        >
          No note matches.
        </p>
      </template>

      <!-- Results the project scope is hiding, rather than a silent nothing. -->
      <button
        v-if="elsewhere"
        class="focus-ring m-2 flex w-[calc(100%-16px)] flex-wrap items-center gap-2 rounded-[var(--radius-control)] border border-border-strong bg-canvas px-2.5 py-2 text-left text-[12px] text-text-muted transition-colors hover:border-accent hover:text-text"
        @click="store.setScope(null)"
      >
        <span>{{ elsewhere.count }} more in {{ elsewhere.names.join(', ') }}</span>
        <span class="ml-auto whitespace-nowrap text-[11.5px] text-accent">
          search everywhere &#8594;
        </span>
      </button>
      </template>
    </div>

    <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
  </nav>
</template>
