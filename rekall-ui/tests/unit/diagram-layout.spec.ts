import { describe, expect, it } from 'vitest'
import { project } from '@/common/diagram/graph-projection'
import { borderPoint, layoutProjection, NODE_WIDTH, preferredDirection } from '@/common/diagram/graph-layout'
import { SemanticGraphSchema } from '@/api/schemas/diagram.schema'
import { orderGraph } from './diagram-graph.fixture'
import terminalOpen from './diagram-terminal-open.graph.json'

describe('graph layout', () => {
  it('places every node, frames every cluster around its members and routes every edge', () => {
    const projection = project(orderGraph(), { lens: 'code', collapsed: new Set() })
    const layout = layoutProjection(projection, 'LR')

    expect([...layout.nodes.keys()].sort()).toEqual(['capture', 'done', 'orders', 'paid', 'receive', 'validate'])
    expect([...layout.clusters.keys()].sort()).toEqual(['charge', 'process'])
    const frame = layout.clusters.get('process')!
    for (const id of ['receive', 'validate', 'paid']) {
      const box = layout.nodes.get(id)!
      expect(box.width).toBe(NODE_WIDTH)
      expect(box.x).toBeGreaterThanOrEqual(frame.x)
      expect(box.y).toBeGreaterThan(frame.y)
      expect(box.x + box.width).toBeLessThanOrEqual(frame.x + frame.width)
      expect(box.y + box.height).toBeLessThanOrEqual(frame.y + frame.height)
    }
    expect(layout.edges).toHaveLength(projection.edges.length)
    expect(layout.edges.every((edge) => edge.points.length >= 2)).toBe(true)
  })

  it('is the same layout every time for the same graph', () => {
    const projection = project(orderGraph(), { lens: 'concept', collapsed: new Set() })
    expect(layoutProjection(projection, 'TB')).toEqual(layoutProjection(projection, 'TB'))
  })

  it('reads left to right when asked to', () => {
    const projection = project(orderGraph(), { lens: 'concept', collapsed: new Set() })
    const layout = layoutProjection(projection, 'LR')
    expect(layout.nodes.get('receive')!.x).toBeLessThan(layout.nodes.get('done')!.x)
  })

  it('a border point lies on the box edge facing the target', () => {
    expect(borderPoint({ x: 0, y: 0, width: 100, height: 40 }, { x: 500, y: 20 })).toEqual({ x: 100, y: 20 })
    expect(borderPoint({ x: 0, y: 0, width: 100, height: 40 }, { x: 50, y: -300 })).toEqual({ x: 50, y: 0 })
  })

  it('keeps every element outside a frame clear of it, title band included, in both directions', () => {
    const graph = SemanticGraphSchema.parse(terminalOpen)
    const projection = project(graph, { lens: 'code', collapsed: new Set() })
    for (const direction of ['LR', 'TB'] as const) {
      const layout = layoutProjection(projection, direction)
      for (const frame of layout.clusters.values()) {
        const members = new Set(projection.nodes.filter((node) => node.parentId === frame.id).map((node) => node.id))
        const intruders = [...layout.nodes.values()].filter(
          (box) =>
            !members.has(box.id) &&
            box.x < frame.x + frame.width &&
            box.x + box.width > frame.x &&
            box.y < frame.y + frame.height &&
            box.y + box.height > frame.y
        )
        expect(intruders.map((box) => `${direction}:${box.id}`)).toEqual([])
      }
    }
  })

  it('reads a long single flow top to bottom when that is what fits a screen', () => {
    const graph = SemanticGraphSchema.parse(terminalOpen)
    expect(preferredDirection(project(graph, { lens: 'concept', collapsed: new Set() }))).toBe('TB')
  })
})
