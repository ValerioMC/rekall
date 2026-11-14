export const MOTIONS = ['full', 'reduced'] as const
export type Motion = (typeof MOTIONS)[number]

export const SCALES = [90, 100, 110, 125] as const
export type Scale = (typeof SCALES)[number]

export const TASK_WIDTHS = ['narrow', 'standard', 'wide', 'full'] as const
export type TaskWidth = (typeof TASK_WIDTHS)[number]

/** How wide the task page's column is, from its centre; `full` has no limit. */
export const TASK_WIDTH_PIXELS: Readonly<Record<TaskWidth, number | null>> = {
  narrow: 760,
  standard: 1040,
  wide: 1360,
  full: null
}

/** What the console remembers per browser about how it looks and how it asks; the server never sees it. */
export interface UiPreferences {
  readonly motion: Motion
  readonly scale: Scale
  readonly taskWidth: TaskWidth
  readonly taskRailCollapsed: boolean
  readonly reviewAutoAdvance: boolean
  readonly reviewConfirmSendBack: boolean
  readonly workdayGoalHours: number
  readonly usageWarningPercent: number
}

export const WORKDAY_GOAL_RANGE = { min: 1, max: 16 } as const
export const USAGE_WARNING_RANGE = { min: 50, max: 100 } as const

export const DEFAULT_UI_PREFERENCES: UiPreferences = {
  motion: 'full',
  scale: 100,
  taskWidth: 'wide',
  taskRailCollapsed: false,
  reviewAutoAdvance: true,
  reviewConfirmSendBack: false,
  workdayGoalHours: 8,
  usageWarningPercent: 80
}

const KEY = 'rekall.ui.preferences'

function oneOf<T extends string | number>(allowed: readonly T[], value: unknown, fallback: T): T {
  return allowed.find((candidate) => candidate === value) ?? fallback
}

function flag(value: unknown, fallback: boolean): boolean {
  return typeof value === 'boolean' ? value : fallback
}

export function withinRange(value: unknown, range: { readonly min: number; readonly max: number }, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? Math.min(range.max, Math.max(range.min, Math.round(value))) : fallback
}

export function loadUiPreferences(): UiPreferences {
  try {
    const raw = window.localStorage.getItem(KEY)
    const stored: Record<string, unknown> = raw ? (JSON.parse(raw) as Record<string, unknown>) : {}
    const defaults = DEFAULT_UI_PREFERENCES
    return {
      motion: oneOf(MOTIONS, stored.motion, defaults.motion),
      scale: oneOf(SCALES, stored.scale, defaults.scale),
      taskWidth: oneOf(TASK_WIDTHS, stored.taskWidth, defaults.taskWidth),
      taskRailCollapsed: flag(stored.taskRailCollapsed, defaults.taskRailCollapsed),
      reviewAutoAdvance: flag(stored.reviewAutoAdvance, defaults.reviewAutoAdvance),
      reviewConfirmSendBack: flag(stored.reviewConfirmSendBack, defaults.reviewConfirmSendBack),
      workdayGoalHours: withinRange(stored.workdayGoalHours, WORKDAY_GOAL_RANGE, defaults.workdayGoalHours),
      usageWarningPercent: withinRange(stored.usageWarningPercent, USAGE_WARNING_RANGE, defaults.usageWarningPercent)
    }
  } catch {
    return DEFAULT_UI_PREFERENCES
  }
}

// Storage can be full or blocked; the choice still holds for this session, it is just not kept.
export function saveUiPreferences(preferences: UiPreferences): void {
  try {
    window.localStorage.setItem(KEY, JSON.stringify(preferences))
  } catch {
    return
  }
}
