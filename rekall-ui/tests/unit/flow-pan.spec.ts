import { describe, expect, it } from 'vitest'
import { isCanvasBackground } from '@/composables/useFlowPan'

describe('where a press starts a canvas pan', () => {
  const scroller = document.createElement('div')
  const surface = document.createElement('div')
  const step = document.createElement('button')
  scroller.append(surface)
  surface.append(step)

  it('pans from the scroller itself', () => {
    expect(isCanvasBackground(scroller, scroller, surface)).toBe(true)
  })

  it('pans from the drawing surface', () => {
    expect(isCanvasBackground(surface, scroller, surface)).toBe(true)
  })

  it('leaves a press on a step or a route to its own handler', () => {
    expect(isCanvasBackground(step, scroller, surface)).toBe(false)
  })

  it('pans from nothing when no target is known', () => {
    expect(isCanvasBackground(null, scroller, surface)).toBe(false)
  })
})
