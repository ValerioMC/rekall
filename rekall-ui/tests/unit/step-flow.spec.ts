import { describe, expect, it } from 'vitest'
import { readFlow, routesInto, routeTargets } from '@/common/flow/step-flow'
import { flowStep, releaseFlow } from './flow.fixtures'

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
})
