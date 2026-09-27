import { KNOWN_NODE_KINDS, type KnownNodeKind, type NodeKind, type RelationKind } from '@/model/diagram'

/**
 * How each kind of node is drawn. The shape carries the kind (a reader who ignores colour can
 * still tell a decision from a state); the tint is a second channel, from the `--color-kind-*`
 * tokens, none of which is amber, cyan or a status colour.
 */
export type NodeShape =
  | 'soft'
  | 'rounded'
  | 'lozenge'
  | 'stadium'
  | 'signal'
  | 'cylinder'
  | 'subsystem'
  | 'block'

export interface KindStyle {
  readonly label: string
  readonly tint: string
  readonly shape: NodeShape
  readonly glyph: KnownNodeKind | 'custom'
}

const STYLES: Readonly<Record<KnownNodeKind, KindStyle>> = {
  concept: { label: 'Concept', tint: 'var(--color-kind-concept)', shape: 'soft', glyph: 'concept' },
  action: { label: 'Action', tint: 'var(--color-kind-action)', shape: 'rounded', glyph: 'action' },
  decision: { label: 'Decision', tint: 'var(--color-kind-decision)', shape: 'lozenge', glyph: 'decision' },
  state: { label: 'State', tint: 'var(--color-kind-state)', shape: 'stadium', glyph: 'state' },
  event: { label: 'Event', tint: 'var(--color-kind-event)', shape: 'signal', glyph: 'event' },
  data: { label: 'Data', tint: 'var(--color-kind-data)', shape: 'cylinder', glyph: 'data' },
  external_system: {
    label: 'External system',
    tint: 'var(--color-kind-external)',
    shape: 'subsystem',
    glyph: 'external_system'
  },
  code: { label: 'Code', tint: 'var(--color-kind-code)', shape: 'block', glyph: 'code' }
}

export function isKnownKind(kind: NodeKind): kind is KnownNodeKind {
  return (KNOWN_NODE_KINDS as readonly string[]).includes(kind)
}

export function kindStyle(kind: NodeKind): KindStyle {
  if (isKnownKind(kind)) return STYLES[kind]
  return { label: humanise(kind), tint: 'var(--color-kind-custom)', shape: 'rounded', glyph: 'custom' }
}

/**
 * What an edge means to a reader, which decides its stroke. Flow is the spine a person follows;
 * a condition is a fork in it; data flow is a side channel; structure is background.
 */
export type RelationFamily = 'flow' | 'conditional' | 'data' | 'structure'

const FAMILIES: Readonly<Record<string, RelationFamily>> = {
  leads_to: 'flow',
  transitions_to: 'flow',
  triggers: 'flow',
  calls: 'flow',
  conditionally_leads_to: 'conditional',
  reads: 'data',
  writes: 'data',
  produces: 'data',
  consumes: 'data',
  depends_on: 'structure',
  contains: 'structure'
}

export function relationFamily(relation: RelationKind): RelationFamily {
  return FAMILIES[relation] ?? 'structure'
}

/** `conditionally_leads_to` → `conditionally leads to`. */
export function humanise(name: string): string {
  return name.replace(/_/g, ' ')
}

const VERBS: Readonly<Record<string, string>> = {
  leads_to: 'then',
  conditionally_leads_to: 'if',
  transitions_to: 'becomes',
  triggers: 'triggers',
  calls: 'calls',
  reads: 'reads',
  writes: 'writes',
  produces: 'produces',
  consumes: 'consumes',
  depends_on: 'depends on',
  contains: 'holds'
}

/** The short verb a relation reads as in a list: `if → Paid?`. */
export function relationVerb(relation: RelationKind): string {
  return VERBS[relation] ?? humanise(relation)
}
