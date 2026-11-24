import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { loadProjectTaskPreferences, saveProjectTaskPreferences } from '@/common/config/project-task-preferences'
import { toggledSet } from '@/common/config/persisted-set'
import { groupByProject, isInView, matchesQuery, rowsOfView, sortRows, viewCounts } from '@/features/project/lib/project-task-rules'
import { pageOf } from '@/features/project/lib/task-pagination'
import { taskRows } from '@/features/task/lib/task-rows'
import type { ProjectTaskDensity, ProjectTaskSort, ProjectTaskView } from '@/features/project/model/project-task-list'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { ProjectId, TaskId } from '@/model/branded'

/** How long a task stays listed after a decision took it out of its tab, so the seal's arrival is seen. */
const LINGER_MS = 1400

/**
 * What a project's Tasks page shows and how: the tab, the sort, the density and the text filter,
 * over the tasks of the project in view. The tab, the density and the console's scope are remembered.
 */
export const useProjectTasksStore = defineStore('project-tasks', () => {
  const consoleStore = useConsoleStore()
  const tokenUsage = useTokenUsageStore()
  const remembered = loadProjectTaskPreferences()

  const view = ref<ProjectTaskView>(remembered.view)
  const density = ref<ProjectTaskDensity>(remembered.density)
  const sort = ref<ProjectTaskSort>('task')
  const query = ref('')
  const unfoldedFiled = ref<ReadonlySet<ProjectId>>(new Set())
  const foldedEpics = ref<ReadonlySet<TaskId>>(new Set())
  const peekedTaskId = ref<TaskId | null>(null)
  const lingering = ref<ReadonlySet<TaskId>>(new Set())
  const focusedProjectId = ref<ProjectId | null>(null)
  const page = ref(1)

  const tasksInFocus = computed(() =>
    focusedProjectId.value === null
      ? consoleStore.scopedTasks
      : consoleStore.tasks.filter((task) => task.projectId === focusedProjectId.value)
  )
  const projectInView = computed(() => focusedProjectId.value ?? consoleStore.scopeProject)
  const rows = computed(() =>
    taskRows(tasksInFocus.value, consoleStore.steps, (taskId) => tokenUsage.ofTask(taskId).generated)
  )
  const searchedRows = computed(() => rows.value.filter((row) => matchesQuery(row, query.value)))
  const counts = computed(() => viewCounts(rows.value))
  const shown = computed(() =>
    sortRows(
      searchedRows.value.filter((row) => lingering.value.has(row.task.id) || isInView(row, view.value)),
      sort.value
    )
  )
  const finished = computed(() =>
    view.value === 'filed' ? [] : sortRows(rowsOfView(searchedRows.value, 'filed'), sort.value)
  )
  const paging = computed(() => pageOf(shown.value, page.value))
  const groups = computed(() =>
    groupByProject(paging.value.rows, finished.value, consoleStore.projects, consoleStore.scopeCompany === null)
  )

  /** Narrows the ledger to one project while its page shows it, whatever the console's scope is; null lets the scope rule again. */
  function focusProject(projectId: ProjectId | null): void {
    focusedProjectId.value = projectId
    peekedTaskId.value = null
  }

  function goToPage(requested: number): void {
    page.value = requested
    peekedTaskId.value = null
  }

  function toggleFiled(projectId: ProjectId): void {
    unfoldedFiled.value = toggledSet(unfoldedFiled.value, projectId)
  }

  function toggleEpic(epicId: TaskId): void {
    foldedEpics.value = toggledSet(foldedEpics.value, epicId)
  }

  function togglePeek(taskId: TaskId): void {
    peekedTaskId.value = peekedTaskId.value === taskId ? null : taskId
  }

  function linger(taskId: TaskId): void {
    lingering.value = new Set(lingering.value).add(taskId)
    window.setTimeout(() => (lingering.value = toggledSet(lingering.value, taskId)), LINGER_MS)
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

  watch([view, query, sort, projectInView, () => consoleStore.scopeCompany], () => (page.value = 1))

  watch(
    () => consoleStore.isLoading,
    (loading) => {
      if (!loading) dropScopeNoLongerThere()
    },
    { immediate: true }
  )

  watch([view, density, () => consoleStore.scopeCompany, () => consoleStore.scopeProject], () =>
    saveProjectTaskPreferences({
      view: view.value,
      density: density.value,
      companyId: consoleStore.scopeCompany,
      projectId: consoleStore.scopeProject
    })
  )

  return { view, density, sort, query, unfoldedFiled, foldedEpics, peekedTaskId, projectInView, rows, counts, shown, paging, groups, focusProject, goToPage, toggleFiled, toggleEpic, togglePeek, linger }
})
