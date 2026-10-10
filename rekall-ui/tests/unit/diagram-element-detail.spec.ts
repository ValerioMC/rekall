import { describe, expect, it } from 'vitest'
import { connectionsOf, graphElement, sequenceElement } from '@/common/diagram/element-detail'
import { mix, nativeThemeVariables } from '@/common/diagram/mermaid/native-theme'
import { placeNativeSvg } from '@/common/diagram/mermaid/native-svg'
import { plainLabel } from '@/common/diagram/mermaid/mermaid-text'
import { baseOf, folderOf, spanLabel } from '@/common/diagram/source-label'
import type { GraphScene } from '@/common/diagram/scene/graph-scene'
import type { SequenceScene } from '@/common/diagram/scene/sequence-scene'

const SEQUENCE: SequenceScene = {
  participants: [
    { id: 'S', label: 'OrderService', isActor: false },
    { id: 'P', label: 'P', isActor: false }
  ],
  boxes: [],
  autonumber: false,
  steps: [{ type: 'message', number: 1, from: 'S', to: 'P', text: 'charge', line: 'dashed', head: 'arrow' }]
}

describe('element details', () => {
  it('names a participant by its label and keeps its id when they differ', () => {
    expect(sequenceElement(SEQUENCE, 'S')).toMatchObject({ title: 'OrderService', subtitle: 'S', linkKey: 'S' })
    expect(sequenceElement(SEQUENCE, 'P')?.subtitle).toBeNull()
  })

  it('names a message by its text, its kind and who it goes between', () => {
    expect(sequenceElement(SEQUENCE, '#1')).toMatchObject({ title: 'charge', subtitle: 'OrderService → P', linkKey: '#1' })
    expect(sequenceElement(SEQUENCE, '#1')?.look.label).toBe('Reply #1')
    expect(sequenceElement(SEQUENCE, '#2')).toBeNull()
  })

  it('links an entity by its name and lists a node’s connections both ways', () => {
    const scene: GraphScene = {
      kind: 'er',
      direction: 'TB',
      groups: [],
      nodes: [
        { id: 'entity-ORDER-0', linkKey: 'ORDER', label: 'ORDER', role: 'entity', outline: 'rect', parentId: null, annotations: [], compartments: [] },
        { id: 'entity-LINE-1', linkKey: 'LINE', label: 'LINE', role: 'entity', outline: 'rect', parentId: null, annotations: [], compartments: [] }
      ],
      edges: [{ key: 'e', from: 'entity-ORDER-0', to: 'entity-LINE-1', label: 'holds', line: 'solid', startEnd: 'one', endEnd: 'zero-or-more', startLabel: null, endLabel: null }]
    }
    expect(graphElement(scene, 'entity-ORDER-0')?.linkKey).toBe('ORDER')
    expect(connectionsOf(scene, 'entity-LINE-1').incoming.map((row) => [row.otherTitle, row.label])).toEqual([['ORDER', 'holds']])
    expect(connectionsOf(scene, 'entity-LINE-1').outgoing).toEqual([])
  })
})

describe('labels and sources', () => {
  it('reads a Mermaid label as plain text', () => {
    expect(plainLabel('a<br/>b &amp; <b>c</b> List~Order~')).toBe('a b & c List<Order>')
  })

  it('names a span and splits a path so the file name survives', () => {
    expect(spanLabel({ file: 'a.rs', startLine: 3, endLine: 9, symbol: null })).toBe('L3–9')
    expect(spanLabel({ file: 'a.rs', startLine: null, endLine: null, symbol: null })).toBe('whole file')
    expect([folderOf('src/pty/manager.rs'), baseOf('src/pty/manager.rs')]).toEqual(['src/pty/', 'manager.rs'])
  })
})

describe('Mermaid’s own drawings', () => {
  it('builds a theme from hex tokens and leaves out what is not hex', () => {
    const tokens: Record<string, string> = { '--color-surface-raised': '#000000', '--color-text': '#ffffff', '--color-kind-action': '#ffffff', '--color-kind-decision': 'oklch(1 0 0)' }
    const theme = nativeThemeVariables((token) => tokens[token] ?? '')
    expect(theme.primaryTextColor).toBe('#ffffff')
    expect(theme.cScale0).toBe(mix('#ffffff', '#000000', 0.38))
    expect(theme.cScale1).toBeUndefined()
    expect(mix('#ffffff', '#000000', 0.5)).toBe('#808080')
  })

  it('sizes an SVG to its view box so the camera can fit it', () => {
    const placed = placeNativeSvg('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 120" width="100%" style="max-width: 300px"><g/></svg>')
    expect(placed.bounds).toEqual({ x: 0, y: 0, width: 300, height: 120 })
    expect(placed.markup).toContain('width="300"')
    expect(placed.markup).not.toContain('max-width')
  })
})
