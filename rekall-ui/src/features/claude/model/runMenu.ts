import type { SessionProvider } from '@/features/claude/model/session-provider'

export type RunMenuAction = SessionProvider | 'RUN_IN_QUEUE' | 'DISCUSS' | 'PLAN' | 'PLAN_FLOW' | 'REPLAN'
