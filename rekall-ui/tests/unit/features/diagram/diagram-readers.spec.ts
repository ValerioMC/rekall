import { describe, expect, it } from 'vitest'
import { readSheet } from '@/features/diagram/lib/mermaid/sheet-reader'
import { messagesOf } from '@/features/diagram/lib/scene/sequence-scene'
import type { DiagramKind, DiagramSheet } from '@/features/diagram/model/diagram'

function sheet(kind: DiagramKind, mermaid: string): DiagramSheet {
  return { kind, title: null, mermaid, links: {} }
}

const CHECKOUT = `sequenceDiagram
  autonumber
  actor U as User
  box Aqua Backend
    participant S as OrderService
    participant P as Payments
  end
  U->>+S: place(order)
  S->>+P: charge(card)
  alt accepted
    P-->>S: receipt
  else declined
    P--xS: refused
  end
  deactivate P
  Note right of S: stores it
  S-)U: done`

describe('readSheet', () => {
  it('reads a sequence into participants, numbered messages, notes, fragments and activations', async () => {
    const read = await readSheet(sheet('sequence', CHECKOUT))
    if (read.drawing !== 'sequence') throw new Error('expected a sequence')
    const { scene } = read
    expect(scene.autonumber).toBe(true)
    expect(scene.participants.map((p) => [p.id, p.label, p.isActor])).toEqual([
      ['U', 'User', true],
      ['S', 'OrderService', false],
      ['P', 'Payments', false]
    ])
    expect(scene.boxes).toEqual([{ label: 'Backend', participantIds: ['S', 'P'] }])
    expect(messagesOf(scene).map((m) => [m.number, m.from, m.to, m.text, m.line, m.head])).toEqual([
      [1, 'U', 'S', 'place(order)', 'solid', 'arrow'],
      [2, 'S', 'P', 'charge(card)', 'solid', 'arrow'],
      [3, 'P', 'S', 'receipt', 'dashed', 'arrow'],
      [4, 'P', 'S', 'refused', 'dashed', 'cross'],
      [5, 'S', 'U', 'done', 'solid', 'async']
    ])
    expect(scene.steps.map((step) => step.type)).toEqual([
      'message', 'activate', 'message', 'activate', 'fragment-start', 'message', 'fragment-section', 'message',
      'fragment-end', 'deactivate', 'note', 'message'
    ])
  })

  it('reads a flowchart with shapes, groups, labels and line styles', async () => {
    const read = await readSheet(
      sheet('flowchart', 'flowchart LR\n  a[Start] --> b{Ok?}\n  b -- yes --> c[(Orders)]\n  b -.->|no| d((Done))\n  subgraph g [Store]\n    c\n  end')
    )
    if (read.drawing !== 'graph') throw new Error('expected a graph')
    const { scene } = read
    expect(scene.direction).toBe('LR')
    expect(scene.nodes.map((n) => [n.id, n.label, n.role, n.outline, n.parentId])).toEqual([
      ['a', 'Start', 'step', 'rect', null],
      ['b', 'Ok?', 'decision', 'diamond', null],
      ['c', 'Orders', 'data', 'cylinder', 'g'],
      ['d', 'Done', 'terminal', 'circle', null]
    ])
    expect(scene.groups).toEqual([{ id: 'g', label: 'Store', parentId: null }])
    expect(scene.edges.map((e) => [e.from, e.to, e.label, e.line, e.endEnd])).toEqual([
      ['a', 'b', null, 'solid', 'arrow'],
      ['b', 'c', 'yes', 'solid', 'arrow'],
      ['b', 'd', 'no', 'dotted', 'arrow']
    ])
  })

  it('reads a state diagram with pseudo states, a composite state and a note', async () => {
    const read = await readSheet(
      sheet('state', 'stateDiagram-v2\n  [*] --> Pending\n  Pending --> Paid: pay\n  note right of Paid: money in\n  state Paid {\n    [*] --> Shipping\n  }')
    )
    if (read.drawing !== 'graph') throw new Error('expected a graph')
    const roles = Object.fromEntries(read.scene.nodes.map((n) => [n.id, `${n.role}/${n.outline}/${n.parentId ?? '-'}`]))
    expect(roles).toMatchObject({ root_start: 'pseudo/start/-', Pending: 'state/rounded/-', Shipping: 'state/rounded/Paid' })
    expect(Object.values(roles)).toContain('note/note/-')
    expect(read.scene.groups.map((g) => g.id)).toEqual(['Paid'])
    expect(read.scene.edges.find((e) => e.to === 'Paid')?.label).toBe('pay')
  })

  it('reads classes with their members and relations, and entities by name', async () => {
    const classes = await readSheet(sheet('class', 'classDiagram\n  class Order {\n    +Id id\n    +place(item) Result\n  }\n  Order <|-- BigOrder\n  Order "1" *-- "many" Item : holds'))
    if (classes.drawing !== 'graph') throw new Error('expected a graph')
    const order = classes.scene.nodes.find((n) => n.id === 'Order')
    expect(order?.compartments).toEqual([[{ text: '+Id id', detail: null }], [{ text: '+place(item)', detail: 'Result' }]])
    expect(classes.scene.edges.map((e) => [e.from, e.to, e.startEnd, e.startLabel, e.endLabel, e.label])).toEqual([
      ['Order', 'BigOrder', 'triangle', null, null, null],
      ['Order', 'Item', 'diamond-filled', '1', 'many', 'holds']
    ])

    const entities = await readSheet(sheet('er', 'erDiagram\n  CUSTOMER ||--o{ ORDER : places\n  ORDER {\n    string id PK\n  }'))
    if (entities.drawing !== 'graph') throw new Error('expected a graph')
    expect(entities.scene.nodes.map((n) => n.linkKey)).toEqual(['CUSTOMER', 'ORDER'])
    expect(entities.scene.nodes[1]?.compartments).toEqual([[{ text: 'id', detail: 'string PK' }]])
    expect(entities.scene.edges.map((e) => [e.startEnd, e.endEnd, e.label])).toEqual([['one', 'zero-or-more', 'places']])
  })

  it('leaves every other type to Mermaid', async () => {
    expect(await readSheet(sheet('mindmap', 'mindmap\n  root'))).toEqual({ drawing: 'native' })
  })
})
