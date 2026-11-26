import { syntaxTree } from '@codemirror/language'
import type { EditorState } from '@codemirror/state'
import type { EditorView } from '@codemirror/view'
import type { SyntaxNode } from '@lezer/common'

/** The address of the picture whose markdown starts at `position`, which tells it from its neighbours. */
function addressNodeAt(state: EditorState, position: number): SyntaxNode | null {
  let node: SyntaxNode | null = syntaxTree(state).resolveInner(position, 1)
  while (node && node.name !== 'Image') node = node.parent
  return node?.from === position ? node.getChild('URL') : null
}

/** Rewrites one picture's address as one undo step, leaving the rest of its markdown as written. */
export function rewritePictureAddress(view: EditorView, position: number, rewrite: (address: string) => string): void {
  const address = addressNodeAt(view.state, position)
  if (!address) return
  const written = view.state.sliceDoc(address.from, address.to)
  const next = rewrite(written)
  if (next === written) return
  view.dispatch({ changes: { from: address.from, to: address.to, insert: next }, userEvent: 'input.picture' })
}
