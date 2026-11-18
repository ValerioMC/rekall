import { fetchClaudeUsage } from '@/features/claude/api/claude.api'
import { claudeSessionUsage } from '@/features/claude/model/claude'
import type { AccountUsage } from '@/model/account-usage'

export async function fetchClaudeAccountUsage(refresh: boolean): Promise<AccountUsage> {
  const usage = await fetchClaudeUsage(refresh)
  return { ...usage, primary: claudeSessionUsage(usage) }
}
