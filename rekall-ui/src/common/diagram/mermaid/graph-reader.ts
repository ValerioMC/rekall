import { z } from 'zod'
import { asRecord, requireGetters } from './mermaid-db'
import { plainLabel } from './mermaid-text'
import type {
  CompartmentRow,
  EdgeEnd,
  EdgeLine,
  ElementRole,
  GraphDirection,
  GraphEdge,
  GraphGroup,
  GraphNode,
  GraphScene,
  NodeOutline
} from '../scene/graph-scene'

/**
 * Mermaid's layout data for a flowchart, state, class or ER sheet read into a `GraphScene`.
 * Every one of those types hands its parser's result over as the same nodes and edges, which is
 * what this reads; how each type names a shape or an end is translated here and nowhere else.
 */

type GraphKind = GraphScene['kind']

const MemberSchema = z.object({
  visibility: z.string().default(''),
  id: z.string(),
  parameters: z.string().optional(),
  returnType: z.string().optional(),
  memberType: z.string().optional()
})

const AttributeSchema = z.object({
  type: z.string().default(''),
  name: z.string(),
  keys: z.array(z.string()).default([])
})

const NodeSchema = z.object({
  id: z.string(),
  label: z.unknown().optional(),
  shape: z.string().optional(),
  isGroup: z.boolean().default(false),
  parentId: z.string().optional(),
  annotations: z.array(z.string()).default([]),
  members: z.array(MemberSchema).default([]),
  methods: z.array(MemberSchema).default([]),
  attributes: z.array(AttributeSchema).default([])
})

type RawNode = z.infer<typeof NodeSchema>

const EdgeSchema = z.object({
  id: z.string(),
  start: z.string(),
  end: z.string(),
  label: z.unknown().optional(),
  arrowTypeStart: z.string().optional(),
  arrowTypeEnd: z.string().optional(),
  pattern: z.string().optional(),
  thickness: z.string().optional(),
  startLabelRight: z.string().optional(),
  endLabelLeft: z.string().optional(),
  classes: z.string().optional()
})

type RawEdge = z.infer<typeof EdgeSchema>

const LayoutDataSchema = z.object({ nodes: z.array(NodeSchema), edges: z.array(EdgeSchema) })

const ENDS: Readonly<Record<string, EdgeEnd>> = {
  arrow_point: 'arrow',
  arrow_barb: 'arrow',
  arrow_cross: 'cross',
  arrow_circle: 'circle',
  dependency: 'open-arrow',
  extension: 'triangle',
  composition: 'diamond-filled',
  aggregation: 'diamond',
  lollipop: 'circle',
  only_one: 'one',
  zero_or_one: 'zero-or-one',
  one_or_more: 'one-or-more',
  zero_or_more: 'zero-or-more'
}

const FLOW_OUTLINES: Readonly<Record<string, NodeOutline>> = {
  roundedRect: 'rounded',
  stadium: 'stadium',
  subroutine: 'subroutine',
  diamond: 'diamond',
  hexagon: 'hexagon',
  circle: 'circle',
  doublecircle: 'circle',
  cylinder: 'cylinder',
  lean_right: 'slant',
  lean_left: 'slant',
  trapezoid: 'slant',
  inv_trapezoid: 'slant',
  odd: 'flag'
}

const FLOW_ROLES: Readonly<Partial<Record<NodeOutline, ElementRole>>> = {
  diamond: 'decision',
  cylinder: 'data',
  circle: 'terminal',
  stadium: 'terminal'
}

const STATE_SHAPES: Readonly<Record<string, { role: ElementRole; outline: NodeOutline }>> = {
  stateStart: { role: 'pseudo', outline: 'start' },
  stateEnd: { role: 'pseudo', outline: 'end' },
  fork: { role: 'pseudo', outline: 'bar' },
  join: { role: 'pseudo', outline: 'bar' },
  choice: { role: 'decision', outline: 'diamond' },
  note: { role: 'note', outline: 'note' }
}

/** State notes hang in a group of their own, which is layout bookkeeping, not a frame to draw. */
const NOTE_GROUP = 'noteGroup'

export function readGraph(kind: GraphKind, db: unknown): GraphScene {
  const source = requireGetters(db, ['getData', 'getDirection'])
  const data = LayoutDataSchema.parse(source.getData())
  const entityNames = kind === 'er' ? entityNamesOf(db) : new Map<string, string>()
  const noteGroups = new Set(data.nodes.filter((node) => node.shape === NOTE_GROUP).map((node) => node.id))
  const parentOf = (node: RawNode): string | null => (node.parentId && !noteGroups.has(node.parentId) ? node.parentId : null)

  const groups: GraphGroup[] = data.nodes
    .filter((node) => node.isGroup && !noteGroups.has(node.id))
    .map((node) => ({ id: node.id, label: plainLabel(labelOf(node)), parentId: parentOf(node) }))
  const nodes: GraphNode[] = data.nodes
    .filter((node) => !node.isGroup)
    .map((node) => ({ ...readNode(kind, node), parentId: parentOf(node), linkKey: entityNames.get(node.id) ?? node.id }))
  return { kind, direction: directionOf(source.getDirection()), nodes, groups, edges: data.edges.map(readEdge) }
}

function readNode(kind: GraphKind, node: RawNode): Omit<GraphNode, 'parentId' | 'linkKey'> {
  const label = plainLabel(labelOf(node))
  const base = { id: node.id, label, annotations: node.annotations, compartments: [] }
  switch (kind) {
    case 'flowchart': {
      const outline = FLOW_OUTLINES[node.shape ?? ''] ?? 'rect'
      return { ...base, role: FLOW_ROLES[outline] ?? 'step', outline }
    }
    case 'state': {
      const shape = STATE_SHAPES[node.shape ?? '']
      if (!shape) return { ...base, role: 'state', outline: 'rounded' }
      return { ...base, label: shape.role === 'pseudo' ? '' : label, ...shape }
    }
    case 'class':
      return { ...base, role: 'class', outline: 'rect', compartments: [node.members.map(memberRow), node.methods.map(memberRow)] }
    case 'er':
      return { ...base, role: 'entity', outline: 'rect', compartments: [node.attributes.map(attributeRow)] }
  }
}

function readEdge(edge: RawEdge): GraphEdge {
  const isNote = (edge.classes ?? '').includes('note-edge')
  return {
    key: edge.id,
    from: edge.start,
    to: edge.end,
    label: plainLabel(typeof edge.label === 'string' ? edge.label : '') || null,
    line: isNote ? 'dashed' : lineOf(edge),
    startEnd: isNote ? 'none' : ENDS[edge.arrowTypeStart ?? ''] ?? 'none',
    endEnd: isNote ? 'none' : ENDS[edge.arrowTypeEnd ?? ''] ?? 'none',
    startLabel: plainLabel(edge.startLabelRight) || null,
    endLabel: plainLabel(edge.endLabelLeft) || null
  }
}

function lineOf(edge: RawEdge): EdgeLine {
  if (edge.pattern === 'dotted') return 'dotted'
  if (edge.pattern === 'dashed') return 'dashed'
  if (edge.pattern === 'thick' || edge.thickness === 'thick') return 'thick'
  return 'solid'
}

function memberRow(member: z.infer<typeof MemberSchema>): CompartmentRow {
  if (member.memberType === 'method' || member.parameters !== undefined) {
    return { text: plainLabel(`${member.visibility}${member.id}(${member.parameters ?? ''})`), detail: plainLabel(member.returnType) || null }
  }
  return { text: plainLabel(`${member.visibility}${member.id}`), detail: null }
}

function attributeRow(attribute: z.infer<typeof AttributeSchema>): CompartmentRow {
  const detail = [attribute.type, ...attribute.keys].filter((part) => part.length > 0).join(' ')
  return { text: attribute.name, detail: detail || null }
}

function labelOf(node: RawNode): string {
  return typeof node.label === 'string' ? node.label : node.id
}

function directionOf(raw: unknown): GraphDirection {
  return raw === 'LR' || raw === 'RL' || raw === 'BT' ? raw : 'TB'
}

/** An ER node's id is Mermaid's own (`entity-ORDER-1`); a link names the entity (`ORDER`). */
function entityNamesOf(db: unknown): Map<string, string> {
  const entities = z.record(z.object({ id: z.string() })).parse(asRecord(requireGetters(db, ['getEntities']).getEntities()))
  return new Map(Object.entries(entities).map(([name, entity]) => [entity.id, name]))
}
