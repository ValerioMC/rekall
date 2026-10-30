import type { ToolInput } from '../model/chat-event'

export type ToolIcon = 'terminal' | 'pencil' | 'file-plus' | 'file' | 'search' | 'tool'

export interface ToolDescription {
  readonly name: string
  readonly target: string
  readonly icon: ToolIcon
}

const PATH_LIMIT = 48
const TEXT_LIMIT = 80

/** What a tool card says when collapsed: the tool, and what it was pointed at. */
export function describeTool(input: ToolInput): ToolDescription {
  switch (input.tool) {
    case 'bash':
      return { name: 'Bash', target: firstLine(input.command), icon: 'terminal' }
    case 'edit':
      return { name: 'Edit', target: trimPathStart(input.filePath), icon: 'pencil' }
    case 'write':
      return { name: 'Write', target: trimPathStart(input.filePath), icon: 'file-plus' }
    case 'read':
      return { name: 'Read', target: trimPathStart(input.filePath), icon: 'file' }
    case 'search':
      return { name: input.name, target: searchTarget(input.pattern, input.path), icon: 'search' }
    case 'other':
      return { name: input.name, target: cut(firstLine(input.text), TEXT_LIMIT), icon: 'tool' }
  }
}

function firstLine(text: string): string {
  return text.split('\n', 1)[0] ?? ''
}

function cut(text: string, limit: number): string {
  return text.length <= limit ? text : `${text.slice(0, limit - 1)}…`
}

function trimPathStart(path: string): string {
  return path.length <= PATH_LIMIT ? path : `…${path.slice(path.length - PATH_LIMIT + 1)}`
}

function searchTarget(pattern: string, path: string | null): string {
  return path ? `${pattern} in ${trimPathStart(path)}` : pattern
}
