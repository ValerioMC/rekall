import { describe, expect, it } from 'vitest'
import { actionOfKey, cameraAfter, ZOOM_STEP } from '@/features/diagram/lib/viewer-keys'

describe('the keys of an enlarged diagram', () => {
  it('zooms with + and −, fits with 0, closes with Escape and leaves other keys alone', () => {
    expect(actionOfKey('+')).toEqual({ kind: 'zoom', factor: ZOOM_STEP })
    expect(actionOfKey('=')).toEqual({ kind: 'zoom', factor: ZOOM_STEP })
    expect(actionOfKey('-')).toEqual({ kind: 'zoom', factor: 1 / ZOOM_STEP })
    expect(actionOfKey('0')).toEqual({ kind: 'fit' })
    expect(actionOfKey('Escape')).toEqual({ kind: 'close' })
    expect(actionOfKey('a')).toBeNull()
  })

  it('moves the drawing the way the arrow points the view', () => {
    const camera = { x: 100, y: 100, k: 1 }
    const viewport = { width: 800, height: 600 }

    expect(cameraAfter(actionOfKey('ArrowLeft')!, camera, viewport)).toEqual({ x: 180, y: 100, k: 1 })
    expect(cameraAfter(actionOfKey('ArrowDown')!, camera, viewport)).toEqual({ x: 100, y: 20, k: 1 })
  })

  it('zooms around the middle of the viewport, so what is there stays put', () => {
    const viewport = { width: 800, height: 600 }
    const camera = { x: 0, y: 0, k: 1 }
    const zoomed = cameraAfter({ kind: 'zoom', factor: 2 }, camera, viewport)

    expect(zoomed.k).toBe(2)
    expect({ x: 400 * zoomed.k + zoomed.x, y: 300 * zoomed.k + zoomed.y }).toEqual({ x: 400, y: 300 })
  })
})
