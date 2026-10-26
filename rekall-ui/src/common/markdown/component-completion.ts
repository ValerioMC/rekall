import type { Completion, CompletionContext, CompletionResult, CompletionSource } from '@codemirror/autocomplete'
import type { EditorView } from '@codemirror/view'
import { syntaxTree } from '@codemirror/language'
import type { SyntaxNode } from '@lezer/common'
import { componentNameKey } from '@/common/markdown/component-reference'
import { referenceCompletionType } from '@/common/markdown/reference-completion-icon'
import type { ProjectComponent } from '@/features/project/model/project-component'

export const OPENING = /\[\[[^[\]\n]*$/
const CODE_NODES = new Set(['FencedCode', 'CodeBlock', 'InlineCode'])
const DETAIL_MAX = 60

export function isInsideCode(context: CompletionContext): boolean {
  for (let node: SyntaxNode | null = syntaxTree(context.state).resolveInner(context.pos, -1); node; node = node.parent) {
    if (CODE_NODES.has(node.name)) return true
  }
  return false
}

/** Writes `inserted` (the name and its closing `]]`) and leaves the caret after it. A `]]` already after the caret is replaced, not doubled. */
export function insertReference(inserted: string): (view: EditorView, completion: Completion, from: number, to: number) => void {
  return (view, _completion, from, to) => {
    const closed = view.state.sliceDoc(to, to + 2) === ']]'
    view.dispatch({
      changes: { from, to: closed ? to + 2 : to, insert: inserted },
      selection: { anchor: from + inserted.length },
      userEvent: 'input.complete'
    })
  }
}

function firstLine(description: string): string {
  const line = description.split('\n').find((candidate) => candidate.trim() !== '')?.trim() ?? ''
  return line.length > DETAIL_MAX ? `${line.slice(0, DETAIL_MAX)}…` : line
}

/** The components whose name contains what was typed after `[[`, in the project's order. A name holding `]]` cannot be referenced and is left out. */
export function componentOptions(components: readonly ProjectComponent[], typed: string): Completion[] {
  const wanted = componentNameKey(typed)
  return components
    .filter((component) => !component.name.includes(']]') && componentNameKey(component.name).includes(wanted))
    .map((component) => ({
      label: component.name,
      detail: firstLine(component.description),
      type: referenceCompletionType('component'),
      section: { name: 'Components', rank: 0 },
      apply: insertReference(`${component.name}]]`)
    }))
}

/**
 * Typing `[[` offers the project's components. Nothing opens without components (a note with no
 * project), inside a code span or a fenced block, or when no component matches what was typed.
 */
export function componentCompletion(components: () => readonly ProjectComponent[] | null): CompletionSource {
  return (context): CompletionResult | null => {
    const all = components()
    if (!all || all.length === 0) return null
    const opening = context.matchBefore(OPENING)
    if (!opening || isInsideCode(context)) return null
    const from = opening.from + 2
    const options = componentOptions(all, context.state.sliceDoc(from, context.pos))
    return options.length === 0 ? null : { from, options, filter: false }
  }
}
