import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ElementPanel from '@/features/diagram/components/viewer/ElementPanel.vue'
import SheetTabs from '@/features/diagram/components/viewer/SheetTabs.vue'
import { callTreeOf } from '@/features/diagram/lib/call-tree'
import { sequenceElement } from '@/features/diagram/lib/element-detail'
import type { SequenceScene } from '@/features/diagram/lib/scene/sequence-scene'
import type { DiagramSheet } from '@/features/diagram/model/diagram'
import type { DiagramId } from '@/model/branded'

const SHEETS: DiagramSheet[] = [
  { kind: 'sequence', title: 'Checkout', mermaid: 'sequenceDiagram', links: {} },
  { kind: 'state', title: null, mermaid: 'stateDiagram-v2', links: {} }
]

const SCENE: SequenceScene = {
  participants: [
    { id: 'S', label: 'OrderService', isActor: false },
    { id: 'P', label: 'Payments', isActor: false }
  ],
  boxes: [],
  autonumber: false,
  steps: [
    { type: 'message', number: 1, from: 'S', to: 'P', text: 'charge', line: 'solid', head: 'arrow' },
    { type: 'message', number: 2, from: 'P', to: 'S', text: 'receipt', line: 'dashed', head: 'arrow' }
  ]
}

describe('the sheet tabs', () => {
  it('names a sheet by its title or its type, and moves with the arrow keys', async () => {
    const wrapper = mount(SheetTabs, { props: { sheets: SHEETS, active: 0 }, attachTo: document.body })
    const tabs = wrapper.findAll('[role="tab"]')
    expect(tabs.map((tab) => tab.text())).toEqual(['Checkout', 'State'])
    expect(tabs[0]!.attributes('aria-selected')).toBe('true')

    await wrapper.find('[role="tablist"]').trigger('keydown', { key: 'ArrowRight' })
    await wrapper.find('[role="tablist"]').trigger('keydown', { key: 'ArrowLeft' })
    expect(wrapper.emitted('select')).toEqual([[1], [1]])
    wrapper.unmount()
  })
})

describe('the element panel', () => {
  const tree = callTreeOf(SCENE)

  it('says when an element opens no code, and walks the call tree', async () => {
    const wrapper = mount(ElementPanel, {
      props: {
        diagramId: 'd1' as DiagramId,
        detail: sequenceElement(SCENE, 'P')!,
        sources: [],
        calls: tree,
        connections: null,
        labels: new Map([['P', 'Payments']]),
        links: { '#1': [{ file: 'src/pay.rs', startLine: 3, endLine: 9, symbol: 'charge' }] }
      }
    })
    expect(wrapper.text()).toContain('Not linked to a place in the code.')
    const row = wrapper.find('[data-testid="call-row"]')
    expect(row.text()).toContain('charge')
    expect(row.text()).toContain('↩ receipt')
    await row.trigger('click')
    expect(wrapper.emitted('select')).toEqual([['#1']])
    await wrapper.find('[data-testid="element-panel-close"]').trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(1)
  })
})
