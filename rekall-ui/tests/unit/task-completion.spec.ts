import { CompletionContext, type Completion, type CompletionResult } from '@codemirror/autocomplete'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { ensureSyntaxTree } from '@codemirror/language'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { describe, expect, it } from 'vitest'
import { taskCompletion } from '@/common/markdown/task-completion'
import type { Task, TaskStatus } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'
import { task as taskFixture } from './features/diagram/diagram.fixtures'

/** Only `[[task:` offers the project's tasks, and choosing one writes `[[task:label]]`. */
function task(label: string, title: string, status: TaskStatus = 'TODO'): Task {
  return { ...taskFixture(label as TaskId, label, 'p-1' as ProjectId), title, status }
}

const TASKS = [task('report-export', 'Export the report', 'IN_PROGRESS'), task('auth', 'Sign in with SSO'), task('billing', 'Charge the card')]

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

  it('offers every task by label once [[task: is typed', () => {
    expect(labels(completeAt('See [[task:|'))).toEqual(['task:auth', 'task:billing', 'task:report-export'])
  })

  it('filters by the label or the title typed after the prefix, ignoring case', () => {
    expect(labels(completeAt('[[task:EXP|'))).toEqual(['task:report-export'])
    expect(labels(completeAt('[[task:card|'))).toEqual(['task:billing'])
  })

  it('shows the title and the status beside the label', () => {
    expect(completeAt('[[task:report|')?.options[0]?.detail).toBe('Export the report · In progress')
  })

  it('opens nothing without tasks, for text with no project, or inside code', () => {
    expect(completeAt('[[task:|', [])).toBeNull()
    expect(completeAt('[[task:|', null)).toBeNull()
    expect(completeAt('Write `[[task:au|` here`')).toBeNull()
  })

  it('writes the prefixed label and the closing brackets, replacing a ]] already there', () => {
    const text = 'Build on [[task:bil|]] now'
    const pos = text.indexOf('|')
    const view = new EditorView({ state: EditorState.create({ doc: text.replace('|', '') }) })
    const result = completeAt(text)
    const option = result?.options[0] as Completion
    if (typeof option.apply !== 'function') throw new Error('the option applies by function')

    option.apply(view, option, result!.from, pos)

    expect(view.state.doc.toString()).toBe('Build on [[task:billing]] now')
  })
})
