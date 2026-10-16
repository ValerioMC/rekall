import { describe, expect, it } from 'vitest'
import { callTreeOf } from '@/features/diagram/lib/call-tree'
import { callsFor, graphSpotlight, sequenceSpotlight } from '@/features/diagram/lib/spotlight'
import type { SequenceScene } from '@/features/diagram/lib/scene/sequence-scene'
import type { GraphScene } from '@/features/diagram/lib/scene/graph-scene'

const SCENE: SequenceScene = {
  participants: [
    { id: 'U', label: 'User', isActor: true },
    { id: 'S', label: 'OrderService', isActor: false },
    { id: 'P', label: 'Payments', isActor: false }
  ],
  boxes: [],
  autonumber: false,
  steps: [
    { type: 'message', number: 1, from: 'U', to: 'S', text: 'place', line: 'solid', head: 'arrow' },
    { type: 'message', number: 2, from: 'S', to: 'P', text: 'charge', line: 'solid', head: 'arrow' },
    { type: 'message', number: 3, from: 'P', to: 'S', text: 'receipt', line: 'dashed', head: 'arrow' },
    { type: 'message', number: 4, from: 'S', to: 'U', text: 'done', line: 'dashed', head: 'arrow' }
  ]
}
const TREE = callTreeOf(SCENE)

describe('sequenceSpotlight', () => {
  it('a participant lights every message it takes part in', () => {
    const lit = sequenceSpotlight(SCENE, TREE, 'P', '')
    expect([...lit!.edges]).toEqual(['#2', '#3'])
    expect([...lit!.nodes].sort()).toEqual(['P', 'S'])
  })

  it('a message lights the calls made under it', () => {
    expect([...sequenceSpotlight(SCENE, TREE, '#1', '')!.edges].sort()).toEqual(['#1', '#2', '#3', '#4'])
    expect([...sequenceSpotlight(SCENE, TREE, '#3', '')!.edges].sort()).toEqual(['#2', '#3'])
  })

  it('a search lights what it matches and nothing below two characters', () => {
    expect([...sequenceSpotlight(SCENE, TREE, null, 'pay')!.nodes]).toEqual(['P'])
    expect(sequenceSpotlight(SCENE, TREE, null, 'p')).toBeNull()
  })

  it('the panel lists the calls into a participant or under a message', () => {
    expect(callsFor(TREE, 'P').map((node) => node.call.text)).toEqual(['charge'])
    expect(callsFor(TREE, '#4').map((node) => node.call.text)).toEqual(['place'])
  })
})

describe('graphSpotlight', () => {
  const scene: GraphScene = {
    kind: 'flowchart',
    direction: 'TB',
    groups: [],
    nodes: ['a', 'b', 'c'].map((id) => ({ id, linkKey: id, label: id.toUpperCase(), role: 'step', outline: 'rect', parentId: null, annotations: [], compartments: [] })),
    edges: [
      { key: 'ab', from: 'a', to: 'b', label: null, line: 'solid', startEnd: 'none', endEnd: 'arrow', startLabel: null, endLabel: null },
      { key: 'bc', from: 'b', to: 'c', label: null, line: 'solid', startEnd: 'none', endEnd: 'arrow', startLabel: null, endLabel: null }
    ]
  }

  it('a node lights its neighbours and the edges between them', () => {
    const lit = graphSpotlight(scene, 'a', '')
    expect([...lit!.nodes].sort()).toEqual(['a', 'b'])
    expect([...lit!.edges]).toEqual(['ab'])
  })
})
