import { fetchCodexUsage } from '@/features/codex/api/codex.api'
import type { CodexUsageWindow } from '@/features/codex/model/codex'
import type { AccountUsage, UsageLimit, UsageSeverity } from '@/model/account-usage'

export async function fetchCodexAccountUsage(refresh: boolean): Promise<AccountUsage> {
  const usage = await fetchCodexUsage(refresh)
  const limits: UsageLimit[] = []
  if (usage.status === 'OK') {
    if (usage.primary) limits.push(usageLimit('primary', 'Primary window', usage.primary))
    if (usage.secondary) limits.push(usageLimit('secondary', 'Secondary window', usage.secondary))
  }
  return { status: usage.status, limits, primary: limits[0] ?? null, fetchedAt: null, retryAt: null }
}

function usageLimit(key: string, fallback: string, window: CodexUsageWindow): UsageLimit {
  const percent = Math.min(100, Math.max(0, window.usedPercent))
  return {
    key,
    label: windowLabel(window.windowDurationMins, fallback),
    percent,
    severity: usageSeverity(percent),
    resetsAt: window.resetsAt === null ? null : new Date(window.resetsAt * 1000).toISOString()
  }
}

function usageSeverity(percent: number): UsageSeverity {
  if (percent >= 95) return 'CRITICAL'
  return percent >= 80 ? 'WARNING' : 'NORMAL'
}

function windowLabel(minutes: number | null, fallback: string): string {
  if (!minutes) return fallback
  if (minutes % 1440 === 0) return `${minutes / 1440}-day window`
  if (minutes % 60 === 0) return `${minutes / 60}-hour window`
  return `${minutes}-minute window`
}
