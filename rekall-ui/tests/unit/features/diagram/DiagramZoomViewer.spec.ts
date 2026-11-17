import { afterEach, describe, expect, it } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import DiagramFrame from '@/features/diagram/components/static/DiagramFrame.vue'
import DiagramZoomViewer from '@/features/diagram/components/static/DiagramZoomViewer.vue'
import type { SheetDrawing } from '@/features/diagram/lib/sheet-drawing'

const DRAWING: SheetDrawing = { drawing: 'native', svg: '<g />', bounds: { x: 0, y: 0, width: 320, height: 180 } }
const mounted: VueWrapper[] = []

function viewer(): VueWrapper {
  const wrapper = mount(DiagramZoomViewer, { props: { drawing: DRAWING }, attachTo: document.body })
  mounted.push(wrapper)
  return wrapper
}

function inBody(testid: string): HTMLElement | null {
  return document.body.querySelector<HTMLElement>(`[data-testid="${testid}"]`)
}

afterEach(() => mounted.splice(0).forEach((wrapper) => wrapper.unmount()))

describe('DiagramZoomViewer', () => {
  it('opens as a dialog at the actual size and zooms with its keys and the keyboard', async () => {
    const wrapper = viewer()
    expect(inBody('diagram-zoom')?.getAttribute('role')).toBe('dialog')
    expect(inBody('diagram-zoom-readout')?.textContent).toBe('100%')

    inBody('diagram-zoom-in')?.click()
    await new Promise((resolve) => setTimeout(resolve, 400))
    expect(inBody('diagram-zoom-readout')?.textContent).toBe('125%')

    inBody('diagram-zoom')?.dispatchEvent(new KeyboardEvent('keydown', { key: '-', bubbles: true }))
    await new Promise((resolve) => setTimeout(resolve, 400))
    expect(inBody('diagram-zoom-readout')?.textContent).toBe('100%')
    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('closes on Escape and on its close key', async () => {
    const wrapper = viewer()

    inBody('diagram-zoom')?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    inBody('diagram-zoom-close')?.click()

    expect(wrapper.emitted('close')).toHaveLength(2)
  })
})

describe('DiagramFrame', () => {
  it('opens the enlarged diagram from its key or from a click on the drawing, and shuts it again', async () => {
    const wrapper = mount(DiagramFrame, { props: { drawing: DRAWING }, attachTo: document.body })
    mounted.push(wrapper)
    expect(inBody('diagram-zoom')).toBeNull()

    await wrapper.get('[data-testid="diagram-expand"]').trigger('click')
    expect(inBody('diagram-zoom')).not.toBeNull()

    inBody('diagram-zoom-close')?.click()
    await wrapper.vm.$nextTick()
    expect(inBody('diagram-zoom')).toBeNull()

    await wrapper.get('svg.static-diagram').trigger('click')
    expect(inBody('diagram-zoom')).not.toBeNull()
  })
})
