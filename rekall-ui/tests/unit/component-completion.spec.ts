import { CompletionContext, type Completion, type CompletionResult } from '@codemirror/autocomplete'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { ensureSyntaxTree } from '@codemirror/language'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { describe, expect, it } from 'vitest'
import { componentCompletion, componentOptions } from '@/common/markdown/component-completion'
import type { ProjectComponent } from '@/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

/** Typing `[[` offers the project's components, and choosing one writes `[[Name]]`. */
function component(name: string, description = ''): ProjectComponent {
  return {
    id: name as ProjectComponentId,
    projectId: 'p-1' as ProjectId,
    name,
    folder: `/work/${name}`,
    description,
    position: 0,
    createdAt: '',
    updatedAt: ''
  }
}

const COMPONENTS = [
  component('auth-service', 'Issues and checks tokens.\nMore.'),
  component('Billing'),
  component('Audit log'),
  component('weird ]] name')
]

/** Completion at the `|` in the text, in a document the markdown parser has read. */
function completeAt(text: string, components: readonly ProjectComponent[] | null = COMPONENTS): CompletionResult | null {
  const pos = text.indexOf('|')
  const state = EditorState.create({
    doc: text.replace('|', ''),
    extensions: markdown({ base: markdownLanguage })
  })
  ensureSyntaxTree(state, state.doc.length, 1000)
  const result = componentCompletion(() => components)(new CompletionContext(state, pos, false))
  return result as CompletionResult | null
}

function labels(result: CompletionResult | null): string[] {
  return result?.options.map((option) => option.label) ?? []
}

describe('componentCompletion', () => {
  it('offers every component right after [[', () => {
    expect(labels(completeAt('See [[|'))).toEqual(['auth-service', 'Billing', 'Audit log'])
  })

  it('filters by what was typed, anywhere in the name and ignoring case', () => {
    expect(labels(completeAt('See [[au|'))).toEqual(['auth-service', 'Audit log'])
    expect(labels(completeAt('See [[LOG|'))).toEqual(['Audit log'])
  })

  it('starts the replaced range after the brackets', () => {
    expect(completeAt('See [[au|')?.from).toBe('See [['.length)
  })

  it('leaves out a name holding ]] since it cannot be referenced', () => {
    expect(labels(completeAt('[[weird|'))).toEqual([])
  })

  it('shows the first line of the description beside the name', () => {
    const [first] = completeAt('[[|')?.options ?? []

    expect(first?.detail).toBe('Issues and checks tokens.')
  })

  it('opens nothing without a trigger, once the reference is closed, or with nothing matching', () => {
    expect(completeAt('See au|')).toBeNull()
    expect(completeAt('See [[au]] and |')).toBeNull()
    expect(completeAt('See [[zzz|')).toBeNull()
  })

  it('opens nothing for a project with no components, or for text with no project', () => {
    expect(completeAt('See [[|', [])).toBeNull()
    expect(completeAt('See [[|', null)).toBeNull()
  })

  it('opens nothing inside inline code or a fenced block', () => {
    expect(completeAt('Write `[[au|` here`')).toBeNull()
    expect(completeAt('```\n[[au|\n```')).toBeNull()
  })
})

describe('choosing an option', () => {
  function choose(text: string, label: string): string {
    const pos = text.indexOf('|')
    const view = new EditorView({ state: EditorState.create({ doc: text.replace('|', '') }) })
    const result = completeAt(text)
    const option = result?.options.find((candidate) => candidate.label === label) as Completion
    if (typeof option.apply !== 'function') throw new Error('the option applies by function')
    option.apply(view, option, result!.from, pos)
    return `${view.state.sliceDoc(0, view.state.selection.main.head)}|${view.state.sliceDoc(view.state.selection.main.head)}`
  }

  it('writes the full name and the closing brackets and leaves the caret after them', () => {
    expect(choose('See [[au| now', 'auth-service')).toBe('See [[auth-service]]| now')
  })

  it('replaces a closing ]] that is already there instead of doubling it', () => {
    expect(choose('See [[au|]] now', 'auth-service')).toBe('See [[auth-service]]| now')
  })
})

describe('componentOptions', () => {
  it('keeps the project order and offers all on an empty query', () => {
    expect(componentOptions(COMPONENTS, '').map((option) => option.label)).toEqual(['auth-service', 'Billing', 'Audit log'])
  })
})
