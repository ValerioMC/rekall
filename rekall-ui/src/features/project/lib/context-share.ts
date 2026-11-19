import { estimatedTokens } from '@/features/project/lib/section-size'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectSection } from '@/features/project/model/project-section'

export type ContextPartKey = 'blueprint' | 'components' | 'connections' | 'pages'

export interface ContextPart {
  readonly key: ContextPartKey
  readonly label: string
  readonly detail: string
  readonly tokens: number
}

export interface ContextShare {
  readonly parts: readonly ContextPart[]
  readonly totalTokens: number
  readonly heldBackCharacters: number
}

export interface ContextShareInput {
  readonly blueprint: string | null
  readonly components: readonly ProjectComponent[]
  readonly connections: readonly ProjectConnection[]
  readonly sections: readonly ProjectSection[]
}

function counted(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`
}

function sumOf(lengths: readonly number[]): number {
  return lengths.reduce((sum, length) => sum + length, 0)
}

/** What a session that loads this project is handed, estimated: the blueprint, a summary per component, a line per binding and a title per page. The text of a page is never part of it. */
export function contextShareOf(input: ContextShareInput): ContextShare {
  const parts: ContextPart[] = [
    {
      key: 'blueprint',
      label: 'Blueprint',
      detail: 'Read in every context of the project',
      tokens: estimatedTokens((input.blueprint ?? '').length)
    },
    {
      key: 'components',
      label: 'Components',
      detail: `${counted(input.components.length, 'summary', 'summaries')}; a description only when a task names it`,
      tokens: estimatedTokens(sumOf(input.components.map((component) => component.name.length + component.folder.length + component.summary.length)))
    },
    {
      key: 'connections',
      label: 'Connections',
      detail: `${counted(input.connections.length, 'binding', 'bindings')}, labels and targets`,
      tokens: estimatedTokens(sumOf(input.connections.map((binding) => binding.label.length + binding.kind.length + (binding.target ?? '').length)))
    },
    {
      key: 'pages',
      label: 'Pages',
      detail: `${counted(input.sections.length, 'title', 'titles')}, one line each`,
      tokens: estimatedTokens(sumOf(input.sections.map((section) => section.title.length)))
    }
  ]
  return {
    parts,
    totalTokens: sumOf(parts.map((part) => part.tokens)),
    heldBackCharacters: sumOf(input.sections.map((section) => section.bodyMarkdown.length))
  }
}
