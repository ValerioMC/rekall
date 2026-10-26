import { CompletionContext, type Completion, type CompletionResult } from '@codemirror/autocomplete'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { ensureSyntaxTree } from '@codemirror/language'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { describe, expect, it } from 'vitest'
import { connectionCompletion } from '@/common/markdown/connection-completion'
import type { ConnectionKind, ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

/** Typing `[[` offers the project's connections too, and choosing one writes `[[connection:label]]`. */
function binding(label: string, kind: ConnectionKind, target: string | null = null): ProjectConnection {
  return {
    id: label as ProjectConnectionId,
    connectionId: label as ConnectionId,
    projectId: 'p-1' as ProjectId,
    kind,
    label,
    title: label,
    baseUrl: null,
    kubeContext: null,
    target,
    credentialSetAt: null
  }
}

const CONNECTIONS = [
  binding('board', 'JIRA', 'OPS'),
  binding('api-repo', 'GITLAB', 'team/api'),
  binding('prod', 'KUBERNETES', 'payments'),
  binding('dev', 'KUBERNETES')
]

function completeAt(text: string, connections: readonly ProjectConnection[] | null = CONNECTIONS): CompletionResult | null {
  const pos = text.indexOf('|')
  const state = EditorState.create({ doc: text.replace('|', ''), extensions: markdown({ base: markdownLanguage }) })
  ensureSyntaxTree(state, state.doc.length, 1000)
  return connectionCompletion(() => connections)(new CompletionContext(state, pos, false)) as CompletionResult | null
}

function labels(result: CompletionResult | null): string[] {
  return result?.options.map((option) => option.label) ?? []
}

describe('connectionCompletion', () => {
  it('offers every connection right after [[, clusters first, then GitLab, then Jira', () => {
    expect(labels(completeAt('See [[|'))).toEqual([
      'connection:dev',
      'connection:prod',
      'connection:api-repo',
      'connection:board'
    ])
  })

  it('filters by the label or the prefix typed, ignoring case', () => {
    expect(labels(completeAt('[[PRO|'))).toEqual(['connection:prod'])
    expect(labels(completeAt('[[connection:b|'))).toEqual(['connection:board'])
  })

  it('shows the kind and the target beside the label', () => {
    expect(completeAt('[[prod|')?.options[0]?.detail).toBe('Kubernetes · payments')
  })

  it('opens nothing for a project with no connections, for text with no project, or inside code', () => {
    expect(completeAt('[[|', [])).toBeNull()
    expect(completeAt('[[|', null)).toBeNull()
    expect(completeAt('Write `[[pro|` here`')).toBeNull()
  })

  it('writes the prefixed label and the closing brackets, replacing a ]] already there', () => {
    const text = 'Deploy on [[pro|]] now'
    const pos = text.indexOf('|')
    const view = new EditorView({ state: EditorState.create({ doc: text.replace('|', '') }) })
    const result = completeAt(text)
    const option = result?.options[0] as Completion
    if (typeof option.apply !== 'function') throw new Error('the option applies by function')

    option.apply(view, option, result!.from, pos)

    expect(view.state.doc.toString()).toBe('Deploy on [[connection:prod]] now')
  })
})
