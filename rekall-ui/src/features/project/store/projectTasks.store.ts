import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { loadProjectTaskPreferences, saveProjectTaskPreferences } from '@/common/config/project-task-preferences'
import { toggledSet } from '@/common/config/persisted-set'
import { epicEntries, tasksInEntry } from '@/features/project/lib/epic-entries'
import { listedTasks } from '@/features/project/lib/listed-tasks'
import { isInView, sortRows, viewCounts } from '@/features/project/lib/project-task-rules'
import { passesFilters } from '@/features/project/lib/task-filters'
import { pageOf } from '@/features/project/lib/task-pagination'
import { useTaskFilters } from '@/features/project/store/useTaskFilters'
import { useTaskLayout } from '@/features/project/store/useTaskLayout'
import { taskRows } from '@/features/task/lib/task-rows'
import type { ProjectTaskView } from '@/features/project/model/project-task-list'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { ProjectId, TaskId } from '@/model/branded'

/** How long a task stays listed after a decision took it out of its tab, so the seal's arrival is seen. */
const LINGER_MS = 1400

/**
 * What a project's Tasks page lists: the view, the search and filters, the order, the page and
 * the task previewed, over the tasks of the project in view. `useTaskLayout` holds how it is drawn.
 */
export const useProjectTasksStore = defineStore('project-tasks', () => {
  const consoleStore = useConsoleStore()
  const tokenUsage = useTokenUsageStore()
  const remembered = loadProjectTaskPreferences()

  const view = ref<ProjectTaskView>(remembered.view)
  const foldedEpics = ref<ReadonlySet<TaskId>>(new Set())
  const peekedTaskId = ref<TaskId | null>(null)
  const lingering = ref<ReadonlySet<TaskId>>(new Set())
  const focusedProjectId = ref<ProjectId | null>(null)
  const lastFocusedProjectId = ref<ProjectId | null>(null)
  const page = ref(1)
  const clock = ref(Date.now())

  const tasksInFocus = computed(() =>
    focusedProjectId.value === null
      ? consoleStore.scopedTasks
      : consoleStore.tasks.filter((task) => task.projectId === focusedProjectId.value)
  )
  const projectInView = computed(() => focusedProjectId.value ?? consoleStore.scopeProject)
  const rows = computed(() =>
    listedTasks(
      taskRows(tasksInFocus.value, consoleStore.steps, (taskId) => tokenUsage.ofTask(taskId).generated),
      consoleStore.steps,
      consoleStore.timeEntries,
      clock.value
    )
  )
  const layout = useTaskLayout(remembered, rows)
  const filtering = useTaskFilters({ remembered: remembered.filters, projectId: projectInView, rows, view })
  const counts = computed(() => viewCounts(rows.value))
  const shown = computed(() =>
    sortRows(
      filtering.searched.value.filter(
        (row) => lingering.value.has(row.task.id) || (isInView(row, view.value) && passesFilters(row, filtering.filters.value))
      ),
      layout.sort.value
    )
  )
  const paging = computed(() =>
    pageOf(epicEntries(shown.value, titleOfTask), page.value, layout.pageSize.value, tasksInEntry)
  )
  const previewed = computed(() => {
    const listed = paging.value.items.flatMap((entry) => (entry.kind === 'epic' ? [entry.row, ...entry.members] : [entry.row]))
    return listed.find((row) => row.task.id === peekedTaskId.value) ?? listed[0] ?? null
  })

  function titleOfTask(id: TaskId): string | null {
    return consoleStore.tasks.find((candidate) => candidate.id === id)?.title ?? null
  }

  /** Narrows the list to a project while its page shows it; null lets the console's scope rule. The same project keeps its page and search. */
  function focusProject(projectId: ProjectId | null): void {
    focusedProjectId.value = projectId
    peekedTaskId.value = null
    if (projectId === null || projectId === lastFocusedProjectId.value) return
    lastFocusedProjectId.value = projectId
    filtering.query.value = ''
    page.value = 1
  }

  function goToPage(requested: number): void {
    page.value = requested
    peekedTaskId.value = null
  }

  function toggleEpic(epicId: TaskId): void {
    foldedEpics.value = toggledSet(foldedEpics.value, epicId)
  }

  function togglePeek(taskId: TaskId): void {
    peekedTaskId.value = peekedTaskId.value === taskId ? null : taskId
  }

  /** Beside the list the preview is always showing a task, so a press picks one and never closes it. */
  function preview(taskId: TaskId): void {
    peekedTaskId.value = taskId
  }

  function linger(taskId: TaskId): void {
    lingering.value = new Set(lingering.value).add(taskId)
    window.setTimeout(() => (lingering.value = toggledSet(lingering.value, taskId)), LINGER_MS)
  }

  /** Moves the instant a running timer is counted up to. */
  function tick(nowMs: number): void {
    clock.value = nowMs
  }

  function restoreScope(): void {
    if (consoleStore.scopeCompany !== null || consoleStore.scopeProject !== null) return
    consoleStore.setScope(remembered.companyId, remembered.projectId)
  }

  function dropScopeNoLongerThere(): void {
    const companyGone =
      consoleStore.scopeCompany !== null && !consoleStore.companies.some((company) => company.id === consoleStore.scopeCompany)
    const projectGone =
      consoleStore.scopeProject !== null && !consoleStore.projects.some((project) => project.id === consoleStore.scopeProject)
    if (companyGone || projectGone) consoleStore.setScope(null, null)
  }

  restoreScope()

  watch(view, filtering.dropStatusFilters)
  watch([view, filtering.query, filtering.filtersByProject, layout.sort, layout.pageSize], () => (page.value = 1))

  watch(
    () => consoleStore.isLoading,
    (loading) => {
      if (!loading) dropScopeNoLongerThere()
    },
    { immediate: true }
  )

  watch(
    [
      view,
      layout.density,
      layout.sort,
      layout.pageSize,
      layout.hiddenColumns,
      layout.sidePreview,
      filtering.filtersByProject,
      () => consoleStore.scopeCompany,
      () => consoleStore.scopeProject
    ],
    () =>
      saveProjectTaskPreferences({
        view: view.value,
        density: layout.density.value,
        sort: layout.sort.value,
        pageSize: layout.pageSize.value,
        hiddenColumns: [...layout.hiddenColumns.value],
        sidePreview: layout.sidePreview.value,
        filters: filtering.filtersByProject.value,
        companyId: consoleStore.scopeCompany,
        projectId: consoleStore.scopeProject
      })
  )

  return {
    view,
    foldedEpics,
    peekedTaskId,
    projectInView,
    rows,
    counts,
    shown,
    paging,
    previewed,
    ...layout,
    ...filtering,
    focusProject,
    goToPage,
    toggleEpic,
    togglePeek,
    preview,
    linger,
    tick
  }
})
