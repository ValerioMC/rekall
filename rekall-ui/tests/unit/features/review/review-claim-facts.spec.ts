import { describe, expect, it } from 'vitest'
import { claimFacts } from '@/features/review/lib/review-claim-facts'
import type { CheckResult } from '@/features/check/model/check'

function run(id: string, passed: boolean): CheckResult {
  return { id, checkId: id, taskId: 't', stepId: 's', label: id, command: id, passed, timedOut: false, exitCode: passed ? 0 : 1, ranAt: '2026-10-01T10:00:00Z' } as unknown as CheckResult
}

describe('claimFacts', () => {
  it('says nothing ran when no check was run, and counts the commits', () => {
    expect(claimFacts({ runs: [], commitCount: 2, hasDeviation: false })).toEqual([
      { id: 'checks', tone: 'plain', text: 'No checks' },
      { id: 'commits', tone: 'plain', text: '2 commits' }
    ])
  })

  it('reports a failing check in danger, naming how many failed', () => {
    const [checks] = claimFacts({ runs: [run('a', true), run('b', false)], commitCount: 1, hasDeviation: false })

    expect(checks).toEqual({ id: 'checks', tone: 'bad', text: '1 of 2 checks failed' })
  })

  it('reports passing checks as ok', () => {
    const [checks] = claimFacts({ runs: [run('a', true), run('b', true)], commitCount: 0, hasDeviation: false })

    expect(checks).toEqual({ id: 'checks', tone: 'ok', text: '2 checks passed' })
  })

  it('names a single commit in the singular and none as none', () => {
    expect(claimFacts({ runs: [], commitCount: 1, hasDeviation: false })[1]?.text).toBe('1 commit')
    expect(claimFacts({ runs: [], commitCount: 0, hasDeviation: false })[1]?.text).toBe('No commits')
  })

  it('adds the deviation in warn when the step left the plan', () => {
    const facts = claimFacts({ runs: [], commitCount: 1, hasDeviation: true })

    expect(facts.at(-1)).toEqual({ id: 'deviation', tone: 'warn', text: 'Deviation' })
  })
})
