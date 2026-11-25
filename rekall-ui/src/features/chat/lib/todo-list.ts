import type { TodoItem } from '../model/chat-event'

export function inProgressFirst(items: readonly TodoItem[]): readonly TodoItem[] {
  const current = items.filter((item) => item.status === 'inProgress')
  const rest = items.filter((item) => item.status !== 'inProgress')
  return [...current, ...rest]
}
