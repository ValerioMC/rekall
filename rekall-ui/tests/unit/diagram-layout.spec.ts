import { describe, expect, it } from 'vitest'
import { layoutGraph } from '@/features/diagram/lib/layout/graph-layout'
import { layoutSequence } from '@/features/diagram/lib/layout/sequence-layout'
import type { GraphEdge, GraphNode, GraphScene } from '@/features/diagram/lib/scene/graph-scene'
import type { SequenceScene } from '@/features/diagram/lib/scene/sequence-scene'
import type { Box } from '@/features/diagram/lib/viewport'

function node(id: string, parentId: string | null = null): GraphNode {
  return { id, linkKey: id, label: id.toUpperCase(), role: 'state', outline: 'rounded', parentId, annotations: [], compartments: [] }
}

function edge(from: string, to: string): GraphEdge {
  return { key: `${from}-${to}`, from, to, label: null, line: 'solid', startEnd: 'none', endEnd: 'arrow', startLabel: null, endLabel: null }
}

function inside(inner: Box, outer: Box): boolean {
  return inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.width <= outer.x + outer.width && inner.y + inner.height <= outer.y + outer.height
}

describe('layoutGraph', () => {
  const scene: GraphScene = {
    kind: 'state',
    direction: 'TB',
    nodes: [node('asked'), node('reading', 'drawing'), node('writing', 'drawing'), node('stored')],
    groups: [{ id: 'drawing', label: 'Drawing', parentId: null }],
    edges: [edge('asked', 'drawing'), edge('reading', 'writing'), edge('drawing', 'stored')]
  }
  const layout = layoutGraph(scene)

  it('frames a group around its members', () => {
    const group = layout.groups.get('drawing')!
    expect(inside(layout.nodes.get('reading')!, group)).toBe(true)
    expect(inside(layout.nodes.get('writing')!, group)).toBe(true)
  })

  it('ranks what flows into and out of a group around it', () => {
    const group = layout.groups.get('drawing')!
    expect(layout.nodes.get('asked')!.y).toBeLessThan(group.y)
    expect(layout.nodes.get('stored')!.y).toBeGreaterThan(group.y + group.height)
  })

  it('routes every edge, those touching a group from border to border', () => {
    expect(layout.edges.map((placed) => placed.key).sort()).toEqual(['asked-drawing', 'drawing-stored', 'reading-writing'])
    const into = layout.edges.find((placed) => placed.key === 'asked-drawing')!
    expect(into.points.at(-1)!.y).toBeCloseTo(layout.groups.get('drawing')!.y, 0)
  })

  it('is the same every time for the same sheet', () => {
    expect(layoutGraph(scene)).toEqual(layout)
  })
})

describe('layoutSequence', () => {
  const scene: SequenceScene = {
    participants: [
      { id: 'U', label: 'User', isActor: true },
      { id: 'S', label: 'OrderService', isActor: false },
      { id: 'P', label: 'Payments', isActor: false }
    ],
    boxes: [{ label: 'Backend', participantIds: ['S', 'P'] }],
    autonumber: true,
    steps: [
      { type: 'message', number: 1, from: 'U', to: 'S', text: 'place', line: 'solid', head: 'arrow' },
      { type: 'activate', participantId: 'S' },
      { type: 'fragment-start', kind: 'alt', label: 'card' },
      { type: 'message', number: 2, from: 'S', to: 'P', text: 'charge the card with a long label', line: 'solid', head: 'arrow' },
      { type: 'fragment-section', label: 'cash' },
      { type: 'message', number: 3, from: 'S', to: 'S', text: 'record', line: 'solid', head: 'arrow' },
      { type: 'fragment-end' },
      { type: 'note', placement: 'over', participantIds: ['S', 'P'], text: 'stored' },
      { type: 'message', number: 4, from: 'S', to: 'U', text: 'done', line: 'dashed', head: 'arrow' },
      { type: 'deactivate', participantId: 'S' }
    ]
  }
  const layout = layoutSequence(scene)

  it('puts participants left to right in order, with room for the labels between them', () => {
    const [u, s, p] = ['U', 'S', 'P'].map((id) => layout.participants.get(id)!)
    expect(u!.centre).toBeLessThan(s!.centre)
    expect(p!.centre - s!.centre).toBeGreaterThan(200)
  })

  it('places messages top to bottom, a self-call on its own lifeline', () => {
    const ys = [1, 2, 3, 4].map((number) => layout.messages.get(number)!.y)
    expect([...ys].sort((a, b) => a - b)).toEqual(ys)
    expect(layout.messages.get(3)!.self).toBe(true)
  })

  it('starts an activation at the message that opens it and leaves from its bar', () => {
    const bar = layout.activations.find((activation) => activation.participantId === 'S')!
    expect(bar.y).toBe(layout.messages.get(1)!.y)
    expect(layout.messages.get(2)!.fromX).toBeGreaterThan(layout.participants.get('S')!.centre)
  })

  it('frames a fragment around its messages with a rule per section', () => {
    const [fragment] = layout.fragments
    expect(fragment!.kind).toBe('alt')
    expect(fragment!.sections.map((section) => section.label)).toEqual(['cash'])
    expect(fragment!.y).toBeLessThan(layout.messages.get(2)!.y)
    expect(fragment!.y + fragment!.height).toBeGreaterThan(layout.messages.get(3)!.y)
  })

  it('draws a note over two participants across both, and a box around its participants', () => {
    const [note] = layout.notes
    expect(note!.x).toBeLessThan(layout.participants.get('S')!.centre)
    expect(note!.x + note!.width).toBeGreaterThan(layout.participants.get('P')!.centre)
    expect(inside(layout.participants.get('P')!, layout.boxes[0]!)).toBe(true)
  })
})
