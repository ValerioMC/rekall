import { describe, expect, it } from 'vitest'
import { layoutFlow, NODE_HEIGHT, NODE_WIDTH, placeSteps } from '@/features/flow/lib/flow-layout'
import { flowStep, releaseFlow, route } from './flow.fixtures'

describe('the flow layout', () => {
  it('places every step and routes every link, loops included', () => {
    const layout = layoutFlow(releaseFlow())

    expect(layout.nodes.size).toBe(5)
    expect(layout.routes.map((placed) => placed.key).sort()).toEqual([
      'build-built',
      'deploy-deployed',
      'entry',
      'exit:ship',
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
    expect(layout.routes.map((placed) => placed.key)).toEqual(['entry'])
  })
})

describe('Start and End', () => {
  it('puts Start above the first step and End below the last, even on a plain list', () => {
    const layout = layoutFlow([flowStep('one', 0), flowStep('two', 1)])

    expect(layout.start.y + layout.start.height).toBeLessThan(layout.nodes.get('one' as never)!.y)
    expect(layout.end.y).toBeGreaterThan(layout.nodes.get('two' as never)!.y)
  })

  it('leads Start to the first step and draws a route to the end of the flow into End', () => {
    const layout = layoutFlow([
      flowStep('test', 0, 'OPEN', [route('test', null, 'passed'), route('test', 'fix', 'app-error')]),
      flowStep('fix', 1, 'OPEN', [route('fix', 'test', 'fixed')])
    ])

    const entry = layout.routes.find((placed) => placed.kind === 'entry')!
    const finish = layout.routes.find((placed) => placed.key === 'test-passed')!
    expect([entry.from, entry.to]).toEqual(['flow:start', 'test'])
    expect([finish.from, finish.to]).toEqual(['test', 'flow:end'])
    expect(layout.routes.some((placed) => placed.kind === 'exit')).toBe(false)
  })

  it('joins a step with no routes to End on a flow, so reaching it reads as finishing', () => {
    const layout = layoutFlow(releaseFlow())

    const exit = layout.routes.find((placed) => placed.kind === 'exit')!
    expect([exit.from, exit.to]).toEqual(['ship', 'flow:end'])
  })
})

describe('steps dragged to a place', () => {
  it('moves the step and redraws only the routes that touch it between the boxes', () => {
    const layout = layoutFlow(releaseFlow())
    const untouched = layout.routes.find((placed) => placed.key === 'build-built')!

    const placed = placeSteps(layout, new Map([['fix' as never, { x: 600, y: 500 }]]))

    expect(placed.nodes.get('fix' as never)).toMatchObject({ x: 600, y: 500 })
    expect(placed.routes.find((r) => r.key === 'build-built')).toBe(untouched)
    const moved = placed.routes.find((r) => r.key === 'fix-fixed')!
    expect(moved.points).not.toEqual(layout.routes.find((r) => r.key === 'fix-fixed')!.points)
    expect(moved.points[0]!.x).toBeGreaterThanOrEqual(600)
  })

  it('grows the canvas to hold a step dragged past its edge', () => {
    const layout = layoutFlow(releaseFlow())

    const placed = placeSteps(layout, new Map([['ship' as never, { x: layout.width + 300, y: 40 }]]))

    expect(placed.width).toBeGreaterThan(layout.width + 300)
  })

  it('returns the same layout when nothing was moved', () => {
    const layout = layoutFlow(releaseFlow())

    expect(placeSteps(layout, new Map())).toBe(layout)
  })
})
