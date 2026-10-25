import { describe, expect, it } from 'vitest'
import { blankGraph, freeTitle, releaseLoop } from '@/features/graph/lib/graph-presets'
import { chainOf, loopsBack } from '@/features/graph/lib/graph-chain'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectConnectionId, ProjectId } from '@/model/branded'

function binding(id: string, kind: ProjectConnection['kind']): ProjectConnection {
  return {
    id: `b-${id}` as ProjectConnectionId,
    connectionId: id as ConnectionId,
    projectId: 'p' as ProjectId,
    kind,
    label: id,
    title: id,
    baseUrl: null,
    kubeContext: null,
    target: null,
    credentialSetAt: null
  }
}

describe('graph-presets', () => {
  it('builds the release loop on the first connection of each kind, failures to the fix and back to the push', () => {
    const loop = releaseLoop('Release', [binding('jira', 'JIRA'), binding('gitlab', 'GITLAB'), binding('prod', 'KUBERNETES')])

    expect(loop.steps.map((step) => [step.title, step.connectionId])).toEqual([
      ['Push commit', 'gitlab'],
      ['Pipeline build', 'gitlab'],
      ['Deploy', 'prod'],
      ['Check', 'prod'],
      ['Fix', null]
    ])
    expect(loop.steps[3]!.routes).toEqual([{ outcome: 'healthy', toKey: null }, { outcome: 'error', toKey: 'fix' }])
    expect(loop.steps[4]!.routes).toEqual([{ outcome: 'fixed', toKey: 'push' }, { outcome: 'blocked', toKey: null }])
    expect(loop.steps.map((step) => step.maxVisits)).toEqual([null, null, null, null, 3])
    expect(loopsBack(loop.steps)).toBe(true)
  })

  it('leaves a step without a connection when the project binds none of its kind', () => {
    const loop = releaseLoop('Release', [binding('gitlab', 'GITLAB')])

    expect(loop.steps.filter((step) => step.connectionId === null).map((step) => step.title)).toEqual(['Deploy', 'Check', 'Fix'])
    expect(chainOf(loop.steps, () => 'GITLAB').map((link) => link.kind)).toEqual(['GITLAB', 'GITLAB', null, null, null])
  })

  it('starts a blank graph with one step that loops nowhere', () => {
    const blank = blankGraph('Hotfix')

    expect(blank.steps).toHaveLength(1)
    expect(loopsBack(blank.steps)).toBe(false)
  })

  it('numbers a title the graph already uses', () => {
    const steps = releaseLoop('Release', []).steps

    expect(freeTitle('Deploy', steps)).toBe('Deploy 2')
    expect(freeTitle('Smoke test', steps)).toBe('Smoke test')
  })
})
