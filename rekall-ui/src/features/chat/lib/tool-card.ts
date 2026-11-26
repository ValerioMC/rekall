import { fileDiffs } from './tool-diff'
import type { ToolInput } from '../model/chat-event'
import type { TimelineItem, ToolItem } from '../model/timeline'

export type CommandItem = ToolItem & { readonly input: Extract<ToolInput, { tool: 'bash' }> }

export function isCommand(item: TimelineItem): item is CommandItem {
  return item.kind === 'tool' && item.input.tool === 'bash'
}

/** The chat tells a command by its description alone, so one without it is left out. */
export function isShownInChat(item: TimelineItem): boolean {
  return !isCommand(item) || Boolean(item.input.description)
}

/**
 * A change to files is read in place, a todo list is the progress of the work and an Agent is
 * watched while it runs; every other call waits closed.
 */
export function opensByDefault(call: ToolItem): boolean {
  return changesFiles(call) || call.input.tool === 'todos' || isRunningAgent(call)
}

/** An Agent card folds when its result arrives. */
export function foldsOnResult(call: ToolItem): boolean {
  return call.input.tool === 'agent' && call.state !== 'running'
}

function isRunningAgent(call: ToolItem): boolean {
  return call.input.tool === 'agent' && call.state === 'running'
}

function changesFiles(call: ToolItem): boolean {
  return fileDiffs(call.input).length > 0
}

/** A change to files shows its lines instead of its output, unless it failed and said why. */
export function showsOutput(call: ToolItem): boolean {
  const { result } = call
  if (!result) return false
  return !changesFiles(call) || (result.isError && result.excerpt !== '')
}
