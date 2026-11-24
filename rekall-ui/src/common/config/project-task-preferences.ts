import { asCompanyId, asProjectId, type CompanyId, type ProjectId } from '@/model/branded'
import { PROJECT_TASK_DENSITIES, PROJECT_TASK_VIEWS, type ProjectTaskDensity, type ProjectTaskView } from '@/features/project/model/project-task-list'

/** What a project's Tasks page remembers per browser: the tab and density in use, and the scope it was narrowed to. */
export interface ProjectTaskPreferences {
  readonly view: ProjectTaskView
  readonly density: ProjectTaskDensity
  readonly companyId: CompanyId | null
  readonly projectId: ProjectId | null
}

const KEY = 'rekall.work.preferences'

export const DEFAULT_PROJECT_TASK_PREFERENCES: ProjectTaskPreferences = {
  view: 'needs',
  density: 'comfortable',
  companyId: null,
  projectId: null
}

function oneOf<T extends string>(allowed: readonly T[], value: unknown, fallback: T): T {
  return typeof value === 'string' && (allowed as readonly string[]).includes(value) ? (value as T) : fallback
}

function idOrNull(value: unknown): string | null {
  return typeof value === 'string' && value !== '' ? value : null
}

export function loadProjectTaskPreferences(): ProjectTaskPreferences {
  try {
    const raw = window.localStorage.getItem(KEY)
    const stored: Record<string, unknown> = raw ? (JSON.parse(raw) as Record<string, unknown>) : {}
    const companyId = idOrNull(stored.companyId)
    const projectId = idOrNull(stored.projectId)
    return {
      view: oneOf(PROJECT_TASK_VIEWS, stored.view, DEFAULT_PROJECT_TASK_PREFERENCES.view),
      density: oneOf(PROJECT_TASK_DENSITIES, stored.density, DEFAULT_PROJECT_TASK_PREFERENCES.density),
      companyId: companyId === null ? null : asCompanyId(companyId),
      projectId: projectId === null ? null : asProjectId(projectId)
    }
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
