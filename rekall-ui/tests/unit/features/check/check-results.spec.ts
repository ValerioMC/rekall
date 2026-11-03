import { describe, expect, it } from 'vitest'
import { failedRuns, latestRunsOfStep, runDuration, runOutcome } from '@/features/check/lib/check-results'
import { STEP, result } from './fixtures'
import type { TaskStepId } from '@/model/branded'

describe('the runs of a step', () => {
  it('keeps the last run of each check, in the order they ran', () => {
    const results = [
      result('r3', 'lint', true, '2026-11-03T10:03:00Z'),
      result('r1', 'tests', false, '2026-11-03T10:01:00Z'),
      result('r2', 'tests', true, '2026-11-03T10:02:00Z')
    ]

    expect(latestRunsOfStep(results, STEP).map((run) => run.id)).toEqual(['r2', 'r3'])
  })

  it('leaves out the runs of another step and of a task with no steps', () => {
    const results = [
      result('r1', 'tests', true, '2026-11-03T10:01:00Z', 'other' as TaskStepId),
      result('r2', 'tests', true, '2026-11-03T10:02:00Z', null)
    ]

    expect(latestRunsOfStep(results, STEP)).toEqual([])
  })

  it('names the failed ones', () => {
    const runs = [result('r1', 'tests', false, '2026-11-03T10:01:00Z'), result('r2', 'lint', true, '2026-11-03T10:02:00Z')]

    expect(failedRuns(runs).map((run) => run.id)).toEqual(['r1'])
  })

  it('says how a run ended: passed, exited with a code, or timed out', () => {
    const exited = result('r1', 'tests', false, '2026-11-03T10:01:00Z')

    expect(runOutcome(result('r2', 'lint', true, '2026-11-03T10:02:00Z'))).toBe('Passed')
    expect(runOutcome(exited)).toBe('Failed, exit 101')
    expect(runOutcome({ ...exited, timedOut: true, exitCode: null })).toBe('Timed out')
  })

  it('writes a duration in milliseconds, seconds or minutes', () => {
    expect(runDuration(420)).toBe('420 ms')
    expect(runDuration(1500)).toBe('1.5 s')
    expect(runDuration(125_000)).toBe('2 min 5 s')
  })
})
