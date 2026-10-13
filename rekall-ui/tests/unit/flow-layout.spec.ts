import { describe, expect, it } from 'vitest'
import { layoutFlow, NODE_HEIGHT, NODE_WIDTH } from '@/common/flow/flow-layout'
import { releaseFlow } from './flow.fixtures'

describe('the flow layout', () => {
  it('places every step and routes every link, loops included', () => {
    const layout = layoutFlow(releaseFlow())

    expect(layout.nodes.size).toBe(5)
    expect(layout.routes.map((placed) => placed.id).sort()).toEqual([
      'build-built',
      'deploy-deployed',
      'fix-fixed',
      'test-app-error',
      'test-passed'
    ])
    expect(layout.routes.every((placed) => placed.points.length >= 2)).toBe(true)
  })

  it('reads top to bottom in checklist order', () => {
    const layout = layoutFlow(releaseFlow())

    expect(layout.nodes.get('build' as never)!.y).toBeLessThan(layout.nodes.get('deploy' as never)!.y)
    expect(layout.nodes.get('deploy' as never)!.y).toBeLessThan(layout.nodes.get('test' as never)!.y)
  })

  it('fits every node inside its bounds', () => {
    const layout = layoutFlow(releaseFlow())

    for (const box of layout.nodes.values()) {
      expect(box.width).toBe(NODE_WIDTH)
      expect(box.height).toBe(NODE_HEIGHT)
      expect(box.x + box.width).toBeLessThanOrEqual(layout.width)
      expect(box.y + box.height).toBeLessThanOrEqual(layout.height)
    }
  })

  it('draws the same flow the same way twice', () => {
    expect(layoutFlow(releaseFlow())).toEqual(layoutFlow(releaseFlow()))
  })
})

describe('a checklist with no routes yet', () => {
  it('stacks its steps in checklist order', async () => {
    const { flowStep } = await import('./flow.fixtures')
    const layout = layoutFlow([flowStep('one', 0), flowStep('two', 1), flowStep('three', 2)])
    const ys = ['one', 'two', 'three'].map((id) => layout.nodes.get(id as never)!.y)

    expect(ys[0]).toBeLessThan(ys[1]!)
    expect(ys[1]).toBeLessThan(ys[2]!)
    expect(layout.routes).toEqual([])
  })
})
