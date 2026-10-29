import { StateEffect, StateField } from '@codemirror/state'
import { NO_REFERENCE_TARGETS, type ReferenceTargets } from '@/common/markdown/reference-chip'

/** Sent by the editor's owner when the project's components, connections or tasks change. */
export const referenceTargetsChanged = StateEffect.define<ReferenceTargets>()

/** What `[[...]]` can name, kept in the editor's state so decorations can depend on it. */
export const referenceTargets = StateField.define<ReferenceTargets>({
  create: () => NO_REFERENCE_TARGETS,
  update: (targets, transaction) =>
    transaction.effects.reduce((current, effect) => (effect.is(referenceTargetsChanged) ? effect.value : current), targets)
})
