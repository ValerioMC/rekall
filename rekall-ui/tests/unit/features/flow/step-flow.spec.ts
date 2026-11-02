import { describe, expect, it } from 'vitest'
import type { TaskStep } from '@/features/catalog/model/catalog'
import {
  readFlow,
  routeDestination,
  routeSessionRelation,
  routesInto,
  routeTargets
} from '@/features/flow/lib/step-flow'
import { finishedFlow, flowStep, releaseFlow, route } from './flow.fixtures'

function startingSession(step: TaskStep): TaskStep {
  return { ...step, startsSession: true }
}

function releaseFlowWithSessionAt(stepId: string): TaskStep[] {
  return releaseFlow().map((step) => (step.id === stepId ? startingSession(step) : step))
}

describe('a checklist read as a flow', () => {
  it('stands on the step with the open visit and lists every move in order', () => {
    const flow = readFlow(releaseFlow())

    expect(flow.isFlow).toBe(true)
    expect(flow.currentId).toBe('deploy')
    expect(flow.trail.map((move) => `${move.ordinal}:${move.step.id}:${move.outcome ?? ''}`)).toEqual([
      '1:build:built',
      '2:deploy:deployed',
      '3:test:app-error',
      '4:fix:fixed',
      '5:deploy:'
    ])
  })

  it('knows which routes were walked and which one the last move came in by', () => {
    const flow = readFlow(releaseFlow())

    expect([...flow.walked].sort()).toEqual(['build-built', 'deploy-deployed', 'fix-fixed', 'test-app-error'])
    expect(flow.lastRouteId).toBe('fix-fixed')
  })

  it('leaves drafts out and begins at the first step on the checklist', () => {
    const flow = readFlow([flowStep('draft', 0, 'DRAFT'), flowStep('first', 1)])

    expect(flow.steps.map((step) => step.id)).toEqual(['first'])
    expect(flow.entryId).toBe('first')
    expect(flow.isFlow).toBe(false)
    expect(flow.currentId).toBeNull()
    expect(flow.lastRouteId).toBeNull()
  })

  it('names where each route leads and where each one arrives from', () => {
    const steps = releaseFlow()
    const [, deploy, test] = steps

    expect(routeTargets(test!, steps).map(({ link, target }) => `${link.outcome}->${target?.id}`)).toEqual([
      'app-error->fix',
      'passed->ship'
    ])
    expect(routesInto(deploy!, steps).map(({ source }) => source.id)).toEqual(['build', 'fix'])
  })

  it('is finished once the work leaves a step by a route to the end', () => {
    const flow = readFlow(finishedFlow())

    expect(flow.finished).toBe(true)
    expect(flow.currentId).toBeNull()
    expect(flow.endedFromId).toBe('ship')
    expect(flow.lastRouteId).toBe('ship-shipped')
  })

  it('is not finished while a step is current, or before the work has moved', () => {
    expect(readFlow(releaseFlow()).finished).toBe(false)
    expect(readFlow([flowStep('one', 0)]).finished).toBe(false)
  })

  it('names no target for a route to the end of the flow', () => {
    const steps = finishedFlow()
    const ship = steps.find((step) => step.id === 'ship')!

    expect(routeTargets(ship, steps).map(({ link, target }) => `${link.outcome}->${target}`)).toEqual([
      'shipped->null'
    ])
  })
})

describe('the sessions of a flow', () => {
  it('is one session with nothing marked when no step starts a session', () => {
    const flow = readFlow(releaseFlow())

    expect(flow.sessionCount).toBe(1)
    expect([...flow.sessionOf.values()]).toEqual([1, 1, 1, 1, 1])
    expect(flow.sessionOpeners.size).toBe(0)
    expect(flow.steps.flatMap((step) => step.links).map((link) => routeSessionRelation(link, flow))).toEqual(
      ['none', 'none', 'none', 'none', 'none']
    )
  })

  it('gives each step its session number and marks the step that opens a new one', () => {
    const flow = readFlow(releaseFlowWithSessionAt('test'))

    expect(flow.sessionCount).toBe(2)
    expect(flow.steps.map((step) => `${step.id}:${flow.sessionOf.get(step.id)}`)).toEqual([
      'build:1',
      'deploy:1',
      'test:2',
      'fix:2',
      'ship:2'
    ])
    expect([...flow.sessionOpeners]).toEqual(['test'])
  })

  it('does not count a first step that starts a session as a new session', () => {
    const flow = readFlow(releaseFlowWithSessionAt('build'))

    expect(flow.sessionCount).toBe(1)
    expect(flow.sessionOpeners.size).toBe(0)
  })

  it('tells a route across the boundary from a route inside a session', () => {
    const steps = releaseFlowWithSessionAt('test')
    const flow = readFlow(steps)
    const relation = (id: string): string =>
      routeSessionRelation(steps.flatMap((step) => step.links).find((link) => link.id === id)!, flow)

    expect(relation('deploy-deployed')).toBe('crosses')
    expect(relation('build-built')).toBe('within')
    expect(relation('test-passed')).toBe('within')
    expect(relation('fix-fixed')).toBe('crosses')
  })

  it('relates a route to the end, or to a step that is gone, to no session', () => {
    const flow = readFlow(releaseFlowWithSessionAt('test'))

    expect(routeSessionRelation(route('ship', null, 'shipped'), flow)).toBe('none')
    expect(routeSessionRelation(route('ship', 'missing', 'lost'), flow)).toBe('none')
  })
})

describe('where a route leads, in words', () => {
  it('names the step by its title, the end as end, and a step no longer there as gone', () => {
    const steps = [flowStep('test', 0, 'DRAFT'), flowStep('fix', 1, 'DRAFT')]

    expect(routeDestination(route('test', 'fix', 'app-error'), steps)).toBe('Fix')
    expect(routeDestination(route('test', null, 'passed'), steps)).toBe('end')
    expect(routeDestination(route('test', 'deploy', 'deployed'), steps)).toBe('a step that is gone')
  })
})
