import { describe, expect, it } from 'vitest'
import { conversionLosses, type EpicConversion } from '@/features/catalog/model/epic-conversion'
import type { TaskId } from '@/model/branded'

const nothingToLose: EpicConversion = {
  taskId: 't1' as TaskId,
  steps: 0,
  stepsDone: 0,
  routes: 0,
  hasWrapup: false,
  commits: 0,
  deviations: 0,
  waitsFor: 0,
  neededBy: 0,
  skills: 0,
  hasPullRequest: false,
  epicCode: null,
  epicTitle: null,
  queued: false
}

describe('what turning a task into an epic deletes', () => {
  it('lists nothing for a task that holds only what an epic holds', () => {
    expect(conversionLosses(nothingToLose)).toEqual([])
  })

  it('names the plan with its done steps and routes, and counts each part the task has', () => {
    const losses = conversionLosses({
      ...nothingToLose,
      steps: 12,
      stepsDone: 5,
      routes: 4,
      hasWrapup: true,
      commits: 3,
      deviations: 1,
      waitsFor: 2,
      neededBy: 1,
      epicCode: 'RK-3',
      epicTitle: 'Platform hardening'
    })

    expect(losses.map((loss) => [loss.key, loss.count])).toEqual([
      ['plan', 12],
      ['wrapup', 1],
      ['commits', 3],
      ['deviations', 1],
      ['dependencies', 3],
      ['epic', 1]
    ])
    expect(losses[0]!.detail).toBe('12 steps (5 done) and the flow of 4 routes between them')
    expect(losses[4]!.detail).toBe('waits for 2 tasks; 1 task waits for it')
    expect(losses[5]!.detail).toBe('Leaves Platform hardening; an epic belongs to none')
  })
})
