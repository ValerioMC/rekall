import type { FileChange, TodoItem, ToolInput } from '../model/chat-event'

export type ToolIcon =
  | 'terminal'
  | 'pencil'
  | 'file-plus'
  | 'file'
  | 'search'
  | 'checklist'
  | 'agent'
  | 'web'
  | 'plug'
  | 'tool'

export interface ToolDescription {
  readonly name: string
  readonly target: string
  readonly icon: ToolIcon
  /** The whole path of a call on one file, whose `target` may be cut. */
  readonly path?: string
}

const PATH_LIMIT = 48
const TEXT_LIMIT = 80

/** What a tool card says when collapsed: the tool, and what it was pointed at. */
export function describeTool(input: ToolInput): ToolDescription {
  switch (input.tool) {
    case 'bash':
      return { name: 'Bash', target: firstLine(input.command), icon: 'terminal' }
    case 'edit':
      return { name: 'Edit', target: trimPathStart(input.filePath), icon: 'pencil', path: input.filePath }
    case 'write':
      return { name: 'Write', target: trimPathStart(input.filePath), icon: 'file-plus', path: input.filePath }
    case 'fileChanges':
      return describeFileChanges(input.changes)
    case 'read':
      return { name: 'Read', target: trimPathStart(input.filePath), icon: 'file' }
    case 'search':
      return { name: input.name, target: searchTarget(input.pattern, input.path), icon: 'search' }
    case 'todos':
      return { name: 'Todos', target: todosProgress(input.items), icon: 'checklist' }
    case 'agent':
      return { name: 'Agent', target: cut(firstLine(input.description), TEXT_LIMIT), icon: 'agent' }
    case 'web':
      return { name: input.name, target: webTarget(input.target), icon: 'web' }
    case 'mcp':
      return {
        name: `${input.server} · ${input.toolName}`,
        target: cut(firstLine(input.text), TEXT_LIMIT),
        icon: 'plug'
      }
    case 'other':
      return { name: input.name, target: cut(firstLine(input.text), TEXT_LIMIT), icon: 'tool' }
  }
}

function describeFileChanges(changes: readonly FileChange[]): ToolDescription {
  const only = changes.length === 1 ? changes[0] : undefined
  const target = only ? trimPathStart(only.path) : `${changes.length} files`
  return { name: 'File changes', target, icon: 'pencil', path: only?.path }
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

function todosProgress(items: readonly TodoItem[]): string {
  const done = items.filter((item) => item.status === 'completed').length
  return `${done} of ${items.length} done`
}

function webTarget(target: string): string {
  try {
    return new URL(target).host
  } catch {
    return cut(target, TEXT_LIMIT)
  }
}
