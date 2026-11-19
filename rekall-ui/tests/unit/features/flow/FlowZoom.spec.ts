import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import FlowCanvas from '@/features/flow/components/FlowCanvas.vue'
import { readFlow } from '@/features/flow/lib/step-flow'
import { releaseFlow } from './flow.fixtures'

function canvas() {
  return mount(FlowCanvas, { props: { flow: readFlow(releaseFlow()), selectedId: null } })
}

describe('the flow canvas zoom', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('opens fitted, zooms in and out by steps, and fits again on demand', async () => {
    const pane = canvas()
    const level = () => pane.get('[data-testid="flow-zoom-level"]').text()
    expect(level()).toBe('100%')

    await pane.get('[data-testid="flow-zoom-in"]').trigger('click')
    expect(level()).toBe('120%')

    await pane.get('[data-testid="flow-zoom-out"]').trigger('click')
    await pane.get('[data-testid="flow-zoom-out"]').trigger('click')
    expect(level()).toBe('83%')

    await pane.get('[data-testid="flow-fit"]').trigger('click')
    expect(level()).toBe('100%')
  })

  it('scales the drawing and keeps its scrollable size in step with it', async () => {
    const pane = canvas()
    await pane.get('[data-testid="flow-zoom-out"]').trigger('click')

    const surface = pane.get('[data-testid="flow-canvas"] > div > div')
    expect(surface.attributes('style')).toContain('scale(0.8333')
  })
})
