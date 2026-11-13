import { CompletionContext, type Completion, type CompletionResult } from '@codemirror/autocomplete'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { ensureSyntaxTree } from '@codemirror/language'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { describe, expect, it } from 'vitest'
import { taskCompletion } from '@/common/markdown/task-completion'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStatus } from '@/features/catalog/model/task-status'
import type { ProjectId, TaskId } from '@/model/branded'
import { task as taskFixture } from './features/diagram/diagram.fixtures'

/** Only `[[task:` offers the project's tasks, and choosing one writes `[[task:RK-12]]`. */
function task(code: string, title: string, status: TaskStatus = 'TODO', description: string | null = null): Task {
  return { ...taskFixture(code as TaskId, code, 'p-1' as ProjectId), code, title, status, description }
}

const TASKS = [
  task('RK-3', 'Export the report', 'IN_PROGRESS', 'Stream the rows as CSV.'),
  task('RK-10', 'Sign in with SSO', 'TODO', 'Okta through SAML.'),
  task('RK-7', 'Charge the card')
]

function completeAt(text: string, tasks: readonly Task[] | null = TASKS): CompletionResult | null {
  const pos = text.indexOf('|')
  const state = EditorState.create({ doc: text.replace('|', ''), extensions: markdown({ base: markdownLanguage }) })
  ensureSyntaxTree(state, state.doc.length, 1000)
  return taskCompletion(() => tasks)(new CompletionContext(state, pos, false)) as CompletionResult | null
}

function labels(result: CompletionResult | null): string[] {
  return result?.options.map((option) => option.label) ?? []
}

describe('taskCompletion', () => {
  it('opens nothing after a bare [[ or a partial prefix', () => {
    expect(completeAt('See [[|')).toBeNull()
    expect(completeAt('See [[tas|')).toBeNull()
  })

  it('offers every task by code, newest first, once [[task: is typed', () => {
    expect(labels(completeAt('See [[task:|'))).toEqual(['task:RK-10', 'task:RK-7', 'task:RK-3'])
  })

  it('filters by the code, the title or the description typed after the prefix, ignoring case', () => {
    expect(labels(completeAt('[[task:rk-7|'))).toEqual(['task:RK-7'])
    expect(labels(completeAt('[[task:card|'))).toEqual(['task:RK-7'])
    expect(labels(completeAt('[[task:SAML|'))).toEqual(['task:RK-10'])
    expect(labels(completeAt('[[task:csv|'))).toEqual(['task:RK-3'])
  })

  it('shows the start of the description as the detail of an option that has one', () => {
    expect(completeAt('[[task:csv|')?.options[0]?.info).toBe('Stream the rows as CSV.')
    expect(completeAt('[[task:card|')?.options[0]?.info).toBeUndefined()
  })

  it('shows the title and the status beside the label', () => {
    expect(completeAt('[[task:RK-3|')?.options[0]?.detail).toBe('Export the report · In progress')
  })

  it('opens nothing without tasks, for text with no project, or inside code', () => {
    expect(completeAt('[[task:|', [])).toBeNull()
    expect(completeAt('[[task:|', null)).toBeNull()
    expect(completeAt('Write `[[task:RK|` here`')).toBeNull()
  })

  it('writes the prefixed code and the closing brackets, replacing a ]] already there', () => {
    const text = 'Build on [[task:RK-7|]] now'
    const pos = text.indexOf('|')
    const view = new EditorView({ state: EditorState.create({ doc: text.replace('|', '') }) })
    const result = completeAt(text)
    const option = result?.options[0] as Completion
    if (typeof option.apply !== 'function') throw new Error('the option applies by function')

    option.apply(view, option, result!.from, pos)

    expect(view.state.doc.toString()).toBe('Build on [[task:RK-7]] now')
  })
})
