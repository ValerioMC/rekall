import type { Completion, CompletionContext, CompletionResult, CompletionSource } from '@codemirror/autocomplete'
import { isInsideCode, OPENING } from '@/common/markdown/component-completion'
import { CONNECTION_REFERENCE_PREFIX } from '@/common/markdown/connection-reference'
import { sortedByKind } from '@/features/connection/lib/connection-order'
import { CONNECTION_KIND_LABEL, type ProjectConnection } from '@/features/connection/model/connection'

function detailOf(binding: ProjectConnection): string {
  return [CONNECTION_KIND_LABEL[binding.kind], binding.target].filter((part) => !!part).join(' · ')
}

/**
 * The bindings whose `connection:label` contains what was typed after `[[`, by kind. A label
 * holding `]]` cannot be referenced and is left out.
 */
export function connectionOptions(connections: readonly ProjectConnection[], typed: string): Completion[] {
  const wanted = typed.trim().toLowerCase()
  return sortedByKind(connections)
    .filter(
      (binding) =>
        !binding.label.includes(']]') && `${CONNECTION_REFERENCE_PREFIX}${binding.label}`.toLowerCase().includes(wanted)
    )
    .map((binding) => ({
      label: `${CONNECTION_REFERENCE_PREFIX}${binding.label}`,
      detail: detailOf(binding),
      type: 'variable',
      apply: (view, _completion, from, to) => {
        const closed = view.state.sliceDoc(to, to + 2) === ']]'
        const insert = `${CONNECTION_REFERENCE_PREFIX}${binding.label}]]`
        view.dispatch({
          changes: { from, to: closed ? to + 2 : to, insert },
          selection: { anchor: from + insert.length },
          userEvent: 'input.complete'
        })
      }
    }))
}

/**
 * Typing `[[` offers the project's connections after its components, written as
 * `[[connection:label]]`. Nothing opens without connections, inside code, or with no match.
 */
export function connectionCompletion(connections: () => readonly ProjectConnection[] | null): CompletionSource {
  return (context: CompletionContext): CompletionResult | null => {
    const all = connections()
    if (!all || all.length === 0) return null
    const opening = context.matchBefore(OPENING)
    if (!opening || isInsideCode(context)) return null
    const from = opening.from + 2
    const options = connectionOptions(all, context.state.sliceDoc(from, context.pos))
    return options.length === 0 ? null : { from, options, filter: false }
  }
}
