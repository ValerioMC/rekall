import type { SessionProvider } from '@/features/claude/model/session-provider'

export type ProviderMenuAction = 'RUN_IN_QUEUE' | 'DISCUSS' | 'PLAN' | 'PLAN_FLOW'
export type RunMenuAction = SessionProvider | ProviderMenuAction | 'REPLAN'

export type RunMenuSelection =
  | { action: 'RUN' | ProviderMenuAction; provider: SessionProvider }
  | { action: 'REPLAN' }

export function opensProviderMenu(action: RunMenuAction): action is ProviderMenuAction {
  return action === 'RUN_IN_QUEUE' || action === 'DISCUSS' || action === 'PLAN' || action === 'PLAN_FLOW'
}
