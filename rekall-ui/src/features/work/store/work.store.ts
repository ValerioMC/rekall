import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { loadWorkPreferences, saveWorkPreferences } from '@/common/config/work-preferences'
import { toggledSet } from '@/common/config/persisted-set'
import { groupByProject, isInView, matchesQuery, rowsOfView, sortRows, viewCounts } from '@/features/work/lib/work-view-rules'
import { workRows } from '@/features/work/lib/work-rows'
import type { WorkDensity, WorkSort, WorkView } from '@/features/work/model/work-view'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { ProjectId, TaskId } from '@/model/branded'

/** How long a task stays listed after a decision took it out of its tab, so the seal's arrival is seen. */
const LINGER_MS = 1400

/**
 * What the Work screen shows and how: the tab, the sort, the density and the text filter, over the
 * tasks the console's scope lets through. The tab, the density and the scope are remembered.
 */
export const useWorkStore = defineStore('work', () => {
  const consoleStore = useConsoleStore()
  const tokenUsage = useTokenUsageStore()
  const remembered = loadWorkPreferences()

  const view = ref<WorkView>(remembered.view)
  const density = ref<WorkDensity>(remembered.density)
  const sort = ref<WorkSort>('task')
  const query = ref('')
  const unfoldedFiled = ref<ReadonlySet<ProjectId>>(new Set())
  const peekedTaskId = ref<TaskId | null>(null)
  const lingering = ref<ReadonlySet<TaskId>>(new Set())

  const rows = computed(() =>
    workRows(consoleStore.scopedTasks, consoleStore.steps, (taskId) => tokenUsage.ofTask(taskId).generated)
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
  const groups = computed(() =>
    groupByProject(shown.value, finished.value, consoleStore.projects, consoleStore.scopeCompany === null)
  )

  function toggleFiled(projectId: ProjectId): void {
    unfoldedFiled.value = toggledSet(unfoldedFiled.value, projectId)
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

  watch(
    () => consoleStore.isLoading,
    (loading) => {
      if (!loading) dropScopeNoLongerThere()
    },
    { immediate: true }
  )

  watch([view, density, () => consoleStore.scopeCompany, () => consoleStore.scopeProject], () =>
    saveWorkPreferences({
      view: view.value,
      density: density.value,
      companyId: consoleStore.scopeCompany,
      projectId: consoleStore.scopeProject
    })
  )

  return { view, density, sort, query, unfoldedFiled, peekedTaskId, rows, counts, shown, groups, toggleFiled, togglePeek, linger }
})
