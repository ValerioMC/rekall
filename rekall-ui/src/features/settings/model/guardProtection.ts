export type GuardProtectionState =
  | 'INSTALLED'
  | 'NOT_INSTALLED'
  | 'NEEDS_REPAIR'
  | 'INVALID_SETTINGS'
  | 'UNSUPPORTED'
  | 'TRUST_REQUIRED'
  | 'DISABLED'

export interface GuardProtection {
  readonly state: GuardProtectionState
  readonly settingsFile: string
  readonly configured: boolean
  readonly problem: string | null
}
