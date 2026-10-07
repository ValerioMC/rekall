import { StateEffect, StateField, type Extension, type TransactionSpec } from '@codemirror/state'
import { EditorView } from '@codemirror/view'

/** Whether the editor has focus, kept in its state so decorations can depend on it. */

export const focusChanged = StateEffect.define<boolean>()

export const editorFocus = StateField.define<boolean>({
  create: () => false,
  update: (focused, transaction) =>
    transaction.effects.reduce(
      (current, effect) => (effect.is(focusChanged) ? effect.value : current),
      focused
    )
})

export function trackEditorFocus(): Extension {
  return [editorFocus, EditorView.focusChangeEffect.of((_state, focusing) => focusChanged.of(focusing))]
}

/**
 * Puts the caret at `position` and marks the editor focused in the same step, so the block there
 * shows its source at once instead of after the browser's focus event comes round.
 */
export function caretInto(position: number): TransactionSpec {
  return { selection: { anchor: position }, effects: focusChanged.of(true), scrollIntoView: true }
}
