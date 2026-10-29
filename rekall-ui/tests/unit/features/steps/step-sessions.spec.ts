import { describe, expect, it } from 'vitest'
import { groupBySession, opensSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/catalog/model/catalog'
import type { TaskId, TaskStepId } from '@/model/branded'

function step(id: string, startsSession: boolean): TaskStep {
  return { id: id as TaskStepId, taskId: 't' as TaskId, startsSession } as TaskStep
}

describe('groupBySession', () => {
  it('keeps a checklist with no new session in one group', () => {
    const groups = groupBySession([step('a', false), step('b', false)])

    expect(groups).toHaveLength(1)
    expect(groups[0]!.steps.map((member) => member.id)).toEqual(['a', 'b'])
  })

  it('cuts the list at every step that starts a session and numbers the groups from one', () => {
    const groups = groupBySession([step('a', false), step('b', false), step('c', true), step('d', false), step('e', true)])

    expect(groups.map((group) => [group.number, group.steps.map((member) => member.id)])).toEqual([
      [1, ['a', 'b']],
      [2, ['c', 'd']],
      [3, ['e']]
    ])
  })

  it('lets the first step open the first session whatever it says', () => {
    const groups = groupBySession([step('a', true), step('b', false)])

    expect(groups).toHaveLength(1)
    expect(opensSession([step('a', true)], 0)).toBe(false)
  })

  it('has no groups for no steps', () => {
    expect(groupBySession([])).toEqual([])
  })
})
