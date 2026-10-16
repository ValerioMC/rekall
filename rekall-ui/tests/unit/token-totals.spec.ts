import { describe, expect, it } from 'vitest'
import { projectTokens, stepTokens, taskTokens, withTaskTallies } from '@/features/tokens/lib/token-totals'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'
import type { Task } from '@/features/catalog/model/catalog'
import type { TokenTally } from '@/features/tokens/model/token-usage'

function tally(taskId: string, stepId: string | null, outputTokens: number): TokenTally {
  return {
    taskId: taskId as TaskId,
    stepId: stepId as TaskStepId | null,
    inputTokens: 1,
    outputTokens,
    cacheCreationTokens: 10,
    cacheReadTokens: 100
  }
}

const task = (id: string, projectId: string): Task => ({ id, projectId }) as unknown as Task

const TALLIES = [tally('t1', null, 5), tally('t1', 's1', 7), tally('t2', 's2', 11), tally('t3', null, 13)]

describe('token totals', () => {
  it('sums a step on its own', () => {
    expect(stepTokens(TALLIES, 's1' as TaskStepId)).toEqual({ generated: 7, input: 1, cacheWrite: 10, cacheRead: 100 })
  })

  it('sums a task over its own work and its steps', () => {
    expect(taskTokens(TALLIES, 't1' as TaskId).generated).toBe(12)
  })

  it('sums a project over its tasks and nothing else', () => {
    const tasks = [task('t1', 'p1'), task('t2', 'p1'), task('t3', 'p2')]
    expect(projectTokens(TALLIES, tasks, 'p1' as ProjectId)).toEqual({
      generated: 23,
      input: 3,
      cacheWrite: 30,
      cacheRead: 300
    })
  })

  it('is zero for something nothing was spent on', () => {
    expect(taskTokens(TALLIES, 'nowhere' as TaskId).generated).toBe(0)
  })

  it('replaces one task’s tallies and keeps the others', () => {
    const replaced = withTaskTallies(TALLIES, 't1' as TaskId, [tally('t1', null, 99)])
    expect(replaced.filter((t) => t.taskId === 't1').map((t) => t.outputTokens)).toEqual([99])
    expect(replaced).toHaveLength(3)
  })
})
