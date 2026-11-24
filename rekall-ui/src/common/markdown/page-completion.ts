import type { Completion, CompletionContext, CompletionResult, CompletionSource } from '@codemirror/autocomplete'
import { insertReference, isInsideCode } from '@/common/markdown/component-completion'
import { PAGE_REFERENCE_PREFIX, pageTitleKey, type ReferencePage } from '@/common/markdown/page-reference'
import { referenceCompletionType } from '@/common/markdown/reference-completion-icon'

const PAGE_OPENING = /\[\[page:[^[\]\n]*$/i

/** The pages whose title contains what was typed after `[[page:`, in the order the project keeps them. */
export function pageOptions(pages: readonly ReferencePage[], typed: string): Completion[] {
  const wanted = pageTitleKey(typed)
  return pages
    .filter((page) => pageTitleKey(page.title).includes(wanted))
    .map((page) => ({
      label: `${PAGE_REFERENCE_PREFIX}${page.title}`,
      detail: page.contextTitle,
      type: referenceCompletionType('page'),
      section: { name: 'Pages', rank: 3 },
      apply: insertReference(`${PAGE_REFERENCE_PREFIX}${page.title}]]`)
    }))
}

/**
 * Typing `[[page:` offers the project's pages, written as `[[page:Deployment]]`. Nothing opens
 * before the prefix, without pages, inside code, or with no match.
 */
export function pageCompletion(pages: () => readonly ReferencePage[] | null): CompletionSource {
  return (context: CompletionContext): CompletionResult | null => {
    const all = pages()
    if (!all || all.length === 0) return null
    const opening = context.matchBefore(PAGE_OPENING)
    if (!opening || isInsideCode(context)) return null
    const from = opening.from + 2
    const typed = context.state.sliceDoc(from + PAGE_REFERENCE_PREFIX.length, context.pos)
    const options = pageOptions(all, typed)
    return options.length === 0 ? null : { from, options, filter: false }
  }
}
