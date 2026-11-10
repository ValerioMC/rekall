import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import StaticDiagram from '@/features/diagram/components/static/StaticDiagram.vue'
import { drawSource } from '@/features/diagram/lib/sheet-drawing'
import type * as MermaidRuntime from '@/features/diagram/lib/mermaid/mermaid-runtime'

vi.mock('@/features/diagram/lib/mermaid/mermaid-runtime', async (importOriginal) => ({
  ...(await importOriginal<typeof MermaidRuntime>()),
  renderMermaid: vi.fn(async () => '<svg viewBox="0 0 320 180"><g class="mindmap-root" /></svg>')
}))

const SEQUENCE = 'sequenceDiagram\n  participant S as OrderService\n  participant P as Payments\n  S->>P: charge'
const FLOWCHART = 'flowchart TD\n  A[Start] --> B[Done]'

describe('drawSource', () => {
  it('draws a sequence and a flowchart with the console components, and the rest with Mermaid', async () => {
    expect((await drawSource(SEQUENCE)).drawing).toBe('sequence')
    expect((await drawSource(FLOWCHART)).drawing).toBe('graph')

    const native = await drawSource('mindmap\n  root')
    expect(native.drawing).toBe('native')
    expect(native.bounds).toEqual({ x: 0, y: 0, width: 320, height: 180 })
  })

  it('fails for a source Mermaid cannot read, saying why', async () => {
    await expect(drawSource('sequenceDiagram\n  A->>>>B')).rejects.toThrow()
  })
})

describe('StaticDiagram', () => {
  it('draws a sequence at rest, its view box the drawing bounds', async () => {
    const drawing = await drawSource(SEQUENCE)
    const wrapper = mount(StaticDiagram, { props: { drawing } })

    const { width, height } = drawing.bounds
    const svg = wrapper.get('svg.static-diagram')
    expect(svg.attributes('viewBox')).toBe(`${drawing.bounds.x} ${drawing.bounds.y} ${width} ${height}`)
    expect(wrapper.text()).toContain('OrderService')
    expect(wrapper.text()).toContain('charge')
  })

  it('draws a flowchart with its elements', async () => {
    const wrapper = mount(StaticDiagram, { props: { drawing: await drawSource(FLOWCHART) } })

    expect(wrapper.text()).toContain('Start')
    expect(wrapper.text()).toContain('Done')
  })

  it("places Mermaid's own svg for the other types", async () => {
    const wrapper = mount(StaticDiagram, { props: { drawing: await drawSource('mindmap\n  root') } })

    expect(wrapper.find('.mindmap-root').exists()).toBe(true)
  })
})
