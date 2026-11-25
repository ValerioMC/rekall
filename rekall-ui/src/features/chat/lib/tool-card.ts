import type { ToolItem } from '../model/timeline'

/** A todo list is the progress of the work and an Agent is watched while it runs; every other call waits closed. */
export function opensByDefault(call: ToolItem): boolean {
  return call.input.tool === 'todos' || isRunningAgent(call)
}

/** An Agent card folds when its result arrives. */
export function foldsOnResult(call: ToolItem): boolean {
  return call.input.tool === 'agent' && call.state !== 'running'
}

function isRunningAgent(call: ToolItem): boolean {
  return call.input.tool === 'agent' && call.state === 'running'
}

/** An edit or a write shows its lines instead of its output, unless it failed. */
export function showsOutput(call: ToolItem): boolean {
  const { input, result } = call
  if (!result) return false
  const showsLines = input.tool === 'edit' || input.tool === 'write'
  return !showsLines || result.isError
}
