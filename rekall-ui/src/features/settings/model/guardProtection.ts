export type GuardProtectionState = 'INSTALLED' | 'NOT_INSTALLED' | 'INVALID_SETTINGS'

export interface GuardProtection {
  readonly state: GuardProtectionState
  readonly settingsFile: string
  /** Why Claude Code's settings file cannot be edited; only set when it is invalid. */
  readonly problem: string | null
}
