import { asCompanyId, asProjectId, type CompanyId, type ProjectId } from '@/model/branded'
import { WORK_DENSITIES, WORK_VIEWS, type WorkDensity, type WorkView } from '@/features/work/model/work-view'

/** What the Work screen remembers per browser: the tab and density in use, and the scope it was narrowed to. */
export interface WorkPreferences {
  readonly view: WorkView
  readonly density: WorkDensity
  readonly companyId: CompanyId | null
  readonly projectId: ProjectId | null
}

const KEY = 'rekall.work.preferences'

export const DEFAULT_WORK_PREFERENCES: WorkPreferences = {
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

export function loadWorkPreferences(): WorkPreferences {
  try {
    const raw = window.localStorage.getItem(KEY)
    const stored: Record<string, unknown> = raw ? (JSON.parse(raw) as Record<string, unknown>) : {}
    const companyId = idOrNull(stored.companyId)
    const projectId = idOrNull(stored.projectId)
    return {
      view: oneOf(WORK_VIEWS, stored.view, DEFAULT_WORK_PREFERENCES.view),
      density: oneOf(WORK_DENSITIES, stored.density, DEFAULT_WORK_PREFERENCES.density),
      companyId: companyId === null ? null : asCompanyId(companyId),
      projectId: projectId === null ? null : asProjectId(projectId)
    }
  } catch {
    return DEFAULT_WORK_PREFERENCES
  }
}

// Storage can be full or blocked; the screen still works for this session, it is just not kept.
export function saveWorkPreferences(preferences: WorkPreferences): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(preferences))
  } catch {
    return
  }
}
