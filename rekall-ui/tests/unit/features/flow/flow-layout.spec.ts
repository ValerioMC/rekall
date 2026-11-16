import { describe, expect, it } from 'vitest'
import { layoutFlow, NODE_HEIGHT, NODE_WIDTH, placeSteps, routeBetween } from '@/features/flow/lib/flow-layout'
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

describe('a long flow wrapped into columns', () => {
  const longFlow = () =>
    Array.from({ length: 9 }, (_, index) =>
      flowStep(`s${index}`, index, 'OPEN', index === 3 ? [route('s3', 's1', 'again'), route('s3', 's4', 'done')] : [])
    )

  it('leaves a short flow as it is, and a long one as one strip unless asked to wrap', () => {
    const short = [flowStep('one', 0), flowStep('two', 1), flowStep('three', 2)]

    expect(layoutFlow(short, new Map(), true)).toEqual(layoutFlow(short))
    expect(layoutFlow(longFlow()).width).toBeLessThan(layoutFlow(longFlow(), new Map(), true).width)
  })

  it('starts each column at the top and reads on to the right', () => {
    const layout = layoutFlow(longFlow(), new Map(), true)
    const first = layout.nodes.get('s0' as never)!
    const last = layout.nodes.get('s8' as never)!

    expect(last.x).toBeGreaterThan(first.x + NODE_WIDTH)
    expect(layout.height).toBeLessThan(layoutFlow(longFlow()).height)
    for (const box of [...layout.nodes.values(), layout.start, layout.end]) {
      expect(box.x + box.width).toBeLessThanOrEqual(layout.width)
      expect(box.y + box.height).toBeLessThanOrEqual(layout.height)
    }
  })

  it('draws a route between columns out of the side that faces the other column', () => {
    const layout = layoutFlow(longFlow(), new Map(), true)
    const crossing = layout.routes.find((placed) => placed.kind === 'route' && placed.key === 's3-done')!
    const from = layout.nodes.get('s3' as never)!
    const to = layout.nodes.get('s4' as never)!

    if (to.x > from.x + NODE_WIDTH) {
      expect(crossing.points[0]!.x).toBe(from.x + from.width)
      expect(crossing.points.at(-1)!.x).toBe(to.x)
    } else {
      expect(crossing.points.length).toBeGreaterThanOrEqual(2)
    }
  })

  it('leaves the routes inside a column where dagre drew them', () => {
    const wrapped = layoutFlow(longFlow(), new Map(), true)

    expect(wrapped.routes.every((placed) => placed.points.length >= 2)).toBe(true)
    expect(wrapped.routes.map((placed) => placed.key).sort()).toEqual(layoutFlow(longFlow()).routes.map((placed) => placed.key).sort())
  })
})

describe('a route between boxes side by side', () => {
  const left = { x: 0, y: 0, width: 100, height: 60 }
  const right = { x: 300, y: 200, width: 100, height: 60 }

  it('leaves the facing side of each box and arrives flat on the other', () => {
    const forward = routeBetween(left, right)

    expect(forward[0]).toEqual({ x: 100, y: 30 })
    expect(forward.at(-1)).toEqual({ x: 300, y: 230 })
    expect(forward.at(-2)!.y).toBe(230)
    expect(forward[1]!.y).toBe(30)

    const back = routeBetween(right, left)
    expect(back[0]).toEqual({ x: 300, y: 230 })
    expect(back.at(-1)).toEqual({ x: 100, y: 30 })
  })

  it('turns in the gap between the boxes, never past either', () => {
    for (const point of routeBetween(left, right)) {
      expect(point.x).toBeGreaterThanOrEqual(100)
      expect(point.x).toBeLessThanOrEqual(300)
    }
  })

  it('is a straight line when the boxes sit level', () => {
    expect(routeBetween(left, { ...right, y: 0 })).toEqual([
      { x: 100, y: 30 },
      { x: 300, y: 30 }
    ])
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

  it('grows the canvas to hold the loop of a step dragged to its edge', () => {
    const layout = layoutFlow(releaseFlow())

    const placed = placeSteps(
      layout,
      new Map([
        ['fix' as never, { x: layout.width, y: layout.height }],
        ['deploy' as never, { x: layout.width, y: 40 }]
      ])
    )

    const loop = placed.routes.find((r) => r.key === 'fix-fixed')!
    expect(loop.points[1]!.x).toBeGreaterThan(layout.width + NODE_WIDTH)
    for (const point of loop.points) {
      expect(point.x).toBeLessThanOrEqual(placed.width)
      expect(point.y).toBeLessThanOrEqual(placed.height)
    }
  })

  it('returns the same layout when nothing was moved', () => {
    const layout = layoutFlow(releaseFlow())

    expect(placeSteps(layout, new Map())).toBe(layout)
  })
})
