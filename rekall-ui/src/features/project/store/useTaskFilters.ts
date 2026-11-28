import { computed, ref, type Ref } from 'vue'
import { isInView, matchesQuery } from '@/features/project/lib/project-task-rules'
import { applicableFilters, filterOptions, hasFilters, passesFilters } from '@/features/project/lib/task-filters'
import {
  NO_TASK_FILTERS,
  type ListedTask,
  type ProjectTaskView,
  type TaskFilterKey,
  type TaskFilterOption,
  type TaskFilters
} from '@/features/project/model/project-task-list'
import type { ProjectId } from '@/model/branded'

export interface TaskFilterSources {
  readonly remembered: Readonly<Record<string, TaskFilters>>
  readonly projectId: Ref<ProjectId | null>
  readonly rows: Ref<readonly ListedTask[]>
  readonly view: Ref<ProjectTaskView>
}

/**
 * What narrows the task list inside a view: the search text and the filters, which are kept per
 * project because tags and epics mean nothing on another one.
 */
export function useTaskFilters({ remembered, projectId, rows, view }: TaskFilterSources) {
  const query = ref('')
  const filtersByProject = ref<Readonly<Record<string, TaskFilters>>>(remembered)

  const filters = computed(() =>
    applicableFilters(filtersByProject.value[projectId.value ?? ''] ?? NO_TASK_FILTERS, rows.value)
  )
  const isNarrowed = computed(() => query.value.trim() !== '' || hasFilters(filters.value))
  const searched = computed(() => rows.value.filter((row) => matchesQuery(row, query.value)))

  function store(next: TaskFilters): void {
    filtersByProject.value = { ...filtersByProject.value, [projectId.value ?? '']: next }
  }

  function toggleFilter(key: TaskFilterKey, value: string): void {
    const picked = filters.value[key]
    const next = picked.includes(value) ? picked.filter((candidate) => candidate !== value) : [...picked, value]
    store({ ...filters.value, [key]: next })
  }

  function resetFilter(key: TaskFilterKey): void {
    store({ ...filters.value, [key]: [] })
  }

  function clearNarrowing(): void {
    query.value = ''
    store(NO_TASK_FILTERS)
  }

  /** A status belongs to the view it was picked in, so changing view drops it on every project. */
  function dropStatusFilters(): void {
    filtersByProject.value = Object.fromEntries(
      Object.entries(filtersByProject.value).map(([id, kept]) => [id, { ...kept, status: [] }])
    )
  }

  function optionsOf(key: TaskFilterKey): TaskFilterOption[] {
    const candidates = searched.value.filter((row) => isInView(row, view.value) && passesFilters(row, filters.value, key))
    return filterOptions(key, candidates, filters.value[key], rows.value)
  }

  return { query, filtersByProject, filters, isNarrowed, searched, toggleFilter, resetFilter, clearNarrowing, dropStatusFilters, optionsOf }
}
