import { asCompanyId, asProjectId, type CompanyId, type ProjectId } from '@/model/branded'
import {
  DEFAULT_PROJECT_TASK_SORT,
  OPTIONAL_TASK_COLUMNS,
  PROJECT_TASK_DENSITIES,
  PROJECT_TASK_SORT_KEYS,
  PROJECT_TASK_VIEWS,
  TASK_FILTER_KEYS,
  TASK_PAGE_SIZES,
  type OptionalTaskColumn,
  type ProjectTaskDensity,
  type ProjectTaskSort,
  type ProjectTaskView,
  type TaskFilters,
  type TaskPageSize
} from '@/features/project/model/project-task-list'

/**
 * What a project's Tasks page remembers per browser: how the list is drawn, sorted and paged, the
 * filters picked on each project, and the scope it was narrowed to.
 */
export interface ProjectTaskPreferences {
  readonly view: ProjectTaskView
  readonly density: ProjectTaskDensity
  readonly sort: ProjectTaskSort
  readonly pageSize: TaskPageSize
  readonly hiddenColumns: readonly OptionalTaskColumn[]
  readonly sidePreview: boolean
  /** Keyed by project id. */
  readonly filters: Readonly<Record<string, TaskFilters>>
  readonly companyId: CompanyId | null
  readonly projectId: ProjectId | null
}

const KEY = 'rekall.work.preferences'

export const DEFAULT_PROJECT_TASK_PREFERENCES: ProjectTaskPreferences = {
  view: 'needs',
  density: 'comfortable',
  sort: DEFAULT_PROJECT_TASK_SORT,
  pageSize: 25,
  hiddenColumns: [],
  sidePreview: true,
  filters: {},
  companyId: null,
  projectId: null
}

function oneOf<T extends string | number>(allowed: readonly T[], value: unknown, fallback: T): T {
  return (allowed as readonly unknown[]).includes(value) ? (value as T) : fallback
}

function idOrNull(value: unknown): string | null {
  return typeof value === 'string' && value !== '' ? value : null
}

function record(value: unknown): Record<string, unknown> {
  return typeof value === 'object' && value !== null ? (value as Record<string, unknown>) : {}
}

function texts(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : []
}

function sortFrom(value: unknown): ProjectTaskSort {
  const { key, descending } = record(value)
  if (typeof descending !== 'boolean') return DEFAULT_PROJECT_TASK_SORT
  return { key: oneOf(PROJECT_TASK_SORT_KEYS, key, DEFAULT_PROJECT_TASK_SORT.key), descending }
}

function filtersFrom(value: unknown): Record<string, TaskFilters> {
  return Object.fromEntries(
    Object.entries(record(value)).map(([projectId, stored]) => {
      const filters = Object.fromEntries(TASK_FILTER_KEYS.map((key) => [key, texts(record(stored)[key])]))
      return [projectId, filters as unknown as TaskFilters]
    })
  )
}

function preferencesFrom(stored: Record<string, unknown>): ProjectTaskPreferences {
  const fallback = DEFAULT_PROJECT_TASK_PREFERENCES
  const companyId = idOrNull(stored.companyId)
  const projectId = idOrNull(stored.projectId)
  return {
    view: oneOf(PROJECT_TASK_VIEWS, stored.view, fallback.view),
    density: oneOf(PROJECT_TASK_DENSITIES, stored.density, fallback.density),
    sort: sortFrom(stored.sort),
    pageSize: oneOf(TASK_PAGE_SIZES, stored.pageSize, fallback.pageSize),
    hiddenColumns: OPTIONAL_TASK_COLUMNS.filter((column) => texts(stored.hiddenColumns).includes(column)),
    sidePreview: typeof stored.sidePreview === 'boolean' ? stored.sidePreview : fallback.sidePreview,
    filters: filtersFrom(stored.filters),
    companyId: companyId === null ? null : asCompanyId(companyId),
    projectId: projectId === null ? null : asProjectId(projectId)
  }
}

export function loadProjectTaskPreferences(): ProjectTaskPreferences {
  try {
    const raw = window.localStorage.getItem(KEY)
    return preferencesFrom(record(raw ? JSON.parse(raw) : {}))
  } catch {
    return DEFAULT_PROJECT_TASK_PREFERENCES
  }
}

// Storage can be full or blocked; the screen still works for this session, it is just not kept.
export function saveProjectTaskPreferences(preferences: ProjectTaskPreferences): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(preferences))
  } catch {
    return
  }
}
