import { describe, expect, it } from 'vitest'
import { fitCamera, isInView, MAX_SCALE, zoomAt } from '@/common/diagram/viewport'
import { arrivalAngle, midpointAlong, smoothPath } from '@/common/diagram/edge-path'
import { wrapText } from '@/common/diagram/text-wrap'

describe('camera', () => {
  it('zooming keeps the point under the pointer still', () => {
    const camera = zoomAt({ x: 10, y: 20, k: 1 }, 2, { x: 110, y: 120 })
    const worldBefore = { x: (110 - 10) / 1, y: (120 - 20) / 1 }
    expect(worldBefore.x * camera.k + camera.x).toBeCloseTo(110)
    expect(worldBefore.y * camera.k + camera.y).toBeCloseTo(120)
    expect(zoomAt(camera, 100, { x: 0, y: 0 }).k).toBe(MAX_SCALE)
  })

  it('fitting centres the content and never magnifies a small diagram past 1.1', () => {
    const camera = fitCamera({ x: 100, y: 100, width: 200, height: 100 }, { width: 1000, height: 800 }, 40)
    expect(camera.k).toBe(1.1)
    expect(isInView(camera, { x: 100, y: 100, width: 200, height: 100 }, { width: 1000, height: 800 }, 39)).toBe(true)
  })
})

describe('edge paths', () => {
  it('a route smooths into one path, stops short for the arrowhead and knows its arrival angle', () => {
    const route = [{ x: 0, y: 0 }, { x: 50, y: 0 }, { x: 100, y: 0 }]
    expect(smoothPath(route, 6)).toMatch(/^M0\.0,0\.0C.* 94\.0,0\.0$/)
    expect(arrivalAngle(route)).toBe(0)
    expect(midpointAlong(route)).toEqual({ x: 50, y: 0 })
  })
})

describe('title wrapping', () => {
  it('keeps short titles whole and ends long ones with an ellipsis on the last line', () => {
    expect(wrapText('Receive order', 150, 12.5, 2)).toEqual(['Receive order'])
    const long = wrapText('Validate the order against the catalogue, the stock and the customer credit limit', 130, 12.5, 2)
    expect(long).toHaveLength(2)
    expect(long[1]!.endsWith('…')).toBe(true)
  })
})
