import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import FlowCanvas from '@/features/flow/components/FlowCanvas.vue'
import { layoutFlow } from '@/features/flow/lib/flow-layout'
import { gateAt, pillBox, placeGates } from '@/features/flow/lib/route-gate'
import { readFlow } from '@/features/flow/lib/step-flow'
import type { TaskStep } from '@/features/steps/model/task-step'
import { releaseFlow } from './flow.fixtures'

function canvasOf(steps: TaskStep[]) {
  return mount(FlowCanvas, { props: { flow: readFlow(steps), selectedId: null } })
}

function releaseFlowWithSessionAtTest(): TaskStep[] {
  return releaseFlow().map((step) => (step.id === 'test' ? { ...step, startsSession: true } : step))
}

describe('the session gates on the flow routes', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('shows a gate on every route into the step that opens a session, and on a route back into another', () => {
    const canvas = canvasOf(releaseFlowWithSessionAtTest())

    expect(canvas.findAll('[data-testid="flow-gate"]').map((gate) => gate.text())).toEqual(['S2', 'S1'])
    expect(canvas.findAll('[data-testid="flow-gate"]').map((gate) => gate.attributes('aria-label'))).toEqual([
      'Enters session 2, a new Claude session',
      'Moves into session 1'
    ])
  })

  it('gives each route into an opener its own gate', () => {
    const steps = releaseFlowWithSessionAtTest().map((step) =>
      step.id === 'build'
        ? { ...step, links: [...step.links, { ...step.links[0]!, id: 'build-skip' as never, outcome: 'skip', toStepId: 'test' as never }] }
        : step
    )

    const gates = canvasOf(steps).findAll('[data-testid="flow-gate"]')

    expect(
      gates.filter((gate) => gate.attributes('aria-label') === 'Enters session 2, a new Claude session')
    ).toHaveLength(2)
  })

  it('tints the routes that stay inside a session and names them in a title', () => {
    const canvas = canvasOf(releaseFlowWithSessionAtTest())
    const shared = canvas.findAll('g.flow-route-shared')

    expect(shared).toHaveLength(3)
    expect(shared.every((route) => route.get('title').text() === 'Same Claude session')).toBe(true)
  })

  it('draws no gate and no tint when the task runs in one session', () => {
    const canvas = canvasOf(releaseFlow())

    expect(canvas.find('[data-testid="flow-gate"]').exists()).toBe(false)
    expect(canvas.find('.flow-route-shared').exists()).toBe(false)
  })

  it('keeps the walked, pending and live route classes', () => {
    const canvas = canvasOf(releaseFlowWithSessionAtTest())

    expect(canvas.findAll('g.flow-route-live')).toHaveLength(1)
    expect(canvas.findAll('g.flow-route-walked').length).toBeGreaterThan(0)
    expect(canvas.findAll('g.flow-route-pending').length).toBeGreaterThan(0)
  })
})

describe('where a gate sits on its route', () => {
  it('lies on the route and stays clear of the outcome pill', () => {
    const layout = layoutFlow(releaseFlow())

    for (const placed of layout.routes.filter((route) => route.kind === 'route')) {
      const gate = gateAt(placed.points)
      expect(Math.hypot(gate.x - placed.labelAt.x, gate.y - placed.labelAt.y)).toBeGreaterThanOrEqual(20)
    }
  })

  it('follows the route when it is redrawn between moved steps', () => {
    expect(gateAt([{ x: 0, y: 0 }, { x: 0, y: 100 }])).toEqual({ x: 0, y: 26 })
    expect(gateAt([{ x: 0, y: 0 }, { x: 0, y: 200 }])).toEqual({ x: 0, y: 70 })
    expect(gateAt([{ x: 10, y: 10 }, { x: 10, y: 10 }])).toEqual({ x: 10, y: 10 })
  })
})

describe('placing gates among the other marks of the graph', () => {
  const down = [
    { x: 100, y: 0 },
    { x: 100, y: 100 }
  ]

  it('keeps the default spot when nothing is in the way', () => {
    expect(placeGates([{ key: 'a', points: down, label: 'S2' }], []).get('a')).toEqual(gateAt(down))
  })

  it('moves along the route to a free spot when a pill or another gate is in the way', () => {
    const pill = pillBox(gateAt(down), 'fixed')

    const spots = placeGates(
      [
        { key: 'a', points: down, label: 'S2' },
        { key: 'b', points: down, label: 'S3' }
      ],
      [pill]
    )

    expect(spots.get('a')).not.toEqual(gateAt(down))
    expect(spots.get('a')!.x).toBe(100)
    expect(spots.get('b')).not.toEqual(spots.get('a'))
  })

  it('keeps the default spot when the whole route is blocked', () => {
    const blocked = { x: 0, y: -50, width: 300, height: 300 }

    expect(placeGates([{ key: 'a', points: down, label: 'S2' }], [blocked]).get('a')).toEqual(gateAt(down))
  })
})
