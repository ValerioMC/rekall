import { APP_LOCALE } from '@/common/format/app-locale'
import type { TurnContext } from '../model/chat-event'

/** The share of the window held, a whole percent capped at 100; 0 when the window is unknown. */
export function contextPercent(context: TurnContext): number {
  if (context.windowTokens <= 0) return 0
  return Math.min(100, Math.round((context.usedTokens / context.windowTokens) * 100))
}

export function contextTokens(context: TurnContext): string {
  const format = (count: number): string => count.toLocaleString(APP_LOCALE)
  return `${format(context.usedTokens)} of ${format(context.windowTokens)} tokens`
}

export function describeContext(context: TurnContext): string {
  return `${contextTokens(context)} of the context window`
}
