import { splitLines } from './lines'
import type { ToolInput } from '../model/chat-event'
import { SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'

export function describePermission(input: ToolInput, provider: SessionProvider = 'CLAUDE'): string {
  const label = provider === 'CLAUDE' ? 'Claude' : SESSION_PROVIDER_LABEL[provider]
  switch (input.tool) {
    case 'bash':
      return `${label} wants to run a shell command`
    case 'edit':
      return `${label} wants to edit ${input.filePath}`
    case 'write':
      return `${label} wants to write ${input.filePath} (${splitLines(input.content).length} lines)`
    case 'read':
      return `${label} wants to read ${input.filePath}`
    case 'search':
      return input.path
        ? `${label} wants to search for ${input.pattern} in ${input.path}`
        : `${label} wants to search for ${input.pattern}`
    case 'todos':
      return `${label} wants to update its todo list`
    case 'agent':
      return `${label} wants to start an agent: ${input.description}`
    case 'web':
      return `${label} wants to use ${input.name} on ${input.target}`
    case 'mcp':
      return `${label} wants to use ${input.toolName} of ${input.server}`
    case 'other':
      return `${label} wants to use ${input.name}`
  }
}
