import { describe, expect, it } from 'vitest'
import {
  draftOf,
  freeOutcome,
  insertStep,
  moveStep,
  removeStep,
  sameDraft,
  updateRoute
} from '@/features/graph/lib/graph-draft'
import type { DeploymentGraph, DraftStep, GraphDraft } from '@/features/graph/model/deployment-graph'
import type { DeploymentGraphId, GraphStepId, ProjectId } from '@/model/branded'

function step(key: string, routes: DraftStep['routes'] = []): DraftStep {
  return { key, title: key, detail: '', connectionId: null, maxVisits: null, flowX: null, flowY: null, routes }
}

function graphOf(...steps: DraftStep[]): GraphDraft {
  return { title: 'Release', description: '', steps }
}

describe('graph-draft', () => {
  it('keys a saved graph by its step ids, routes by the id they lead to', () => {
    const graph: DeploymentGraph = {
      id: 'g' as DeploymentGraphId,
      projectId: 'p' as ProjectId,
      title: 'Release',
      description: '',
      problem: null,
      createdAt: '',
      updatedAt: '',
      steps: [
        { id: 's1' as GraphStepId, title: 'Push', detail: null, connectionId: null, maxVisits: 3, flowX: null, flowY: null, routes: [{ outcome: 'pushed', toStepId: 's2' as GraphStepId }] },
        { id: 's2' as GraphStepId, title: 'Check', detail: 'Look.', connectionId: null, maxVisits: null, flowX: null, flowY: null, routes: [{ outcome: 'healthy', toStepId: null }] }
      ]
    }

    const draft = draftOf(graph)

    expect(draft.steps.map((s) => s.key)).toEqual(['s1', 's2'])
    expect(draft.steps[0]!.routes).toEqual([{ outcome: 'pushed', toKey: 's2' }])
    expect(draft.steps[0]!.detail).toBe('')
  })

  it('inserts after the picked step and routes a step that routed nowhere to it', () => {
    const draft = insertStep(graphOf(step('push'), step('check')), step('build'), 'push')

    expect(draft.steps.map((s) => s.key)).toEqual(['push', 'build', 'check'])
    expect(draft.steps[0]!.routes).toEqual([{ outcome: 'done', toKey: 'build' }])
  })

  it('leaves a step that already routes as it was, and adds last when nothing is picked', () => {
    const routed = step('push', [{ outcome: 'pushed', toKey: null }])
    const draft = insertStep(graphOf(routed), step('check'), null)

    expect(draft.steps.map((s) => s.key)).toEqual(['push', 'check'])
    expect(draft.steps[0]!.routes).toEqual([{ outcome: 'pushed', toKey: null }])
  })

  it('removes a step with every route that led to it', () => {
    const draft = removeStep(graphOf(step('push', [{ outcome: 'pushed', toKey: 'check' }, { outcome: 'skip', toKey: null }]), step('check')), 'check')

    expect(draft.steps.map((s) => s.key)).toEqual(['push'])
    expect(draft.steps[0]!.routes).toEqual([{ outcome: 'skip', toKey: null }])
  })

  it('moves a step within the graph and not past its ends', () => {
    const draft = graphOf(step('a'), step('b'))

    expect(moveStep(draft, 'b', 'up').steps.map((s) => s.key)).toEqual(['b', 'a'])
    expect(moveStep(draft, 'a', 'up')).toBe(draft)
  })

  it('edits one route and names a free outcome for the next', () => {
    const push = step('push', [{ outcome: 'next', toKey: 'b' }, { outcome: 'next-2', toKey: 'c' }])
    const draft = updateRoute(graphOf(push), 'push', 1, { toKey: null })

    expect(draft.steps[0]!.routes[1]).toEqual({ outcome: 'next-2', toKey: null })
    expect(freeOutcome(push, false)).toBe('next-3')
    expect(freeOutcome(push, true)).toBe('done')
  })

  it('tells two drafts apart only by what they would save', () => {
    expect(sameDraft(graphOf(step('a')), graphOf(step('a')))).toBe(true)
    expect(sameDraft(graphOf(step('a')), graphOf(step('b')))).toBe(false)
  })
})
