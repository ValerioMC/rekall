import type { ToolInput } from '../model/chat-event'
import type { ToolItem } from '../model/timeline'

/** A todo list is the progress of the work, so it is read open; every other call waits closed. */
export function opensByDefault(input: ToolInput): boolean {
  return input.tool === 'todos'
}

/** An edit or a write shows its lines instead of its output, unless it failed. */
export function showsOutput(call: ToolItem): boolean {
  const { input, result } = call
  if (!result) return false
  const showsLines = input.tool === 'edit' || input.tool === 'write'
  return !showsLines || result.isError
}
