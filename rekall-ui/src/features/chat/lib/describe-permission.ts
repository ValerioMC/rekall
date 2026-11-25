import { splitLines } from './lines'
import type { ToolInput } from '../model/chat-event'

/** What Claude is asking leave for, in a sentence a person can answer without reading the input. */
export function describePermission(input: ToolInput): string {
  switch (input.tool) {
    case 'bash':
      return 'Claude wants to run a shell command'
    case 'edit':
      return `Claude wants to edit ${input.filePath}`
    case 'write':
      return `Claude wants to write ${input.filePath} (${splitLines(input.content).length} lines)`
    case 'read':
      return `Claude wants to read ${input.filePath}`
    case 'search':
      return input.path
        ? `Claude wants to search for ${input.pattern} in ${input.path}`
        : `Claude wants to search for ${input.pattern}`
    case 'todos':
      return 'Claude wants to update its todo list'
    case 'agent':
      return `Claude wants to start an agent: ${input.description}`
    case 'web':
      return `Claude wants to use ${input.name} on ${input.target}`
    case 'mcp':
      return `Claude wants to use ${input.toolName} of ${input.server}`
    case 'other':
      return `Claude wants to use ${input.name}`
  }
}
