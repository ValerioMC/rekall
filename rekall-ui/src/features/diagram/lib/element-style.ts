import type { ElementRole } from './scene/graph-scene'

/**
 * How each element is drawn. The tint comes from the `--color-kind-*` tokens, none of which is
 * amber, cyan or a status colour; the glyph repeats it for a reader who ignores colour.
 */
export type ElementGlyph = ElementRole | 'message' | 'external'

export interface ElementStyle {
  readonly label: string
  readonly tint: string
  readonly glyph: ElementGlyph
}

const STYLES: Readonly<Record<ElementRole, ElementStyle>> = {
  participant: { label: 'Participant', tint: 'var(--color-kind-action)', glyph: 'participant' },
  actor: { label: 'Actor', tint: 'var(--color-kind-concept)', glyph: 'actor' },
  step: { label: 'Step', tint: 'var(--color-kind-action)', glyph: 'step' },
  decision: { label: 'Decision', tint: 'var(--color-kind-decision)', glyph: 'decision' },
  terminal: { label: 'Terminal', tint: 'var(--color-kind-event)', glyph: 'terminal' },
  data: { label: 'Data', tint: 'var(--color-kind-data)', glyph: 'data' },
  state: { label: 'State', tint: 'var(--color-kind-state)', glyph: 'state' },
  pseudo: { label: 'Pseudo state', tint: 'var(--color-kind-state)', glyph: 'pseudo' },
  class: { label: 'Class', tint: 'var(--color-kind-concept)', glyph: 'class' },
  entity: { label: 'Entity', tint: 'var(--color-kind-data)', glyph: 'entity' },
  note: { label: 'Note', tint: 'var(--color-kind-custom)', glyph: 'note' }
}

export function roleStyle(role: ElementRole): ElementStyle {
  return STYLES[role]
}

export const MESSAGE_STYLE: ElementStyle = { label: 'Message', tint: 'var(--color-kind-external)', glyph: 'message' }

/** A participant outside the project's code (a client, a database, an HTTP API), as a walkthrough names it. */
export const EXTERNAL_STYLE: ElementStyle = { label: 'External system', tint: 'var(--color-kind-external)', glyph: 'external' }

/** A participant a walkthrough names as `external` is drawn and described as an external system. */
export function participantStyle(isActor: boolean, external: boolean): ElementStyle {
  if (external) return EXTERNAL_STYLE
  return roleStyle(isActor ? 'actor' : 'participant')
}
