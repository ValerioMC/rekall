export type UsageSeverity = 'NORMAL' | 'WARNING' | 'CRITICAL'

export interface UsageLimit {
  readonly key: string
  readonly label: string
  readonly percent: number
  readonly severity: UsageSeverity
  readonly resetsAt: string | null
}

export interface AccountUsage {
  readonly status: 'OK' | 'UNAUTHENTICATED' | 'RATE_LIMITED' | 'UNAVAILABLE' | 'CLI_MISSING'
  readonly limits: readonly UsageLimit[]
  readonly primary: UsageLimit | null
  readonly fetchedAt: string | null
  readonly retryAt: string | null
}
