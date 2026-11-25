import { APP_LOCALE } from '@/common/format/app-locale'
import type { TurnContext } from '../model/chat-event'

/** The share of the window held, a whole percent capped at 100; 0 when the window is unknown. */
export function contextPercent(context: TurnContext): number {
  if (context.windowTokens <= 0) return 0
  return Math.min(100, Math.round((context.usedTokens / context.windowTokens) * 100))
}

export function describeContext(context: TurnContext): string {
  const format = (count: number): string => count.toLocaleString(APP_LOCALE)
  return `${format(context.usedTokens)} of ${format(context.windowTokens)} tokens of the context window`
}
