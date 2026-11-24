import { checksLine, failedRuns } from '@/features/check/lib/check-results'
import type { CheckResult } from '@/features/check/model/check'

export type ClaimFactTone = 'plain' | 'ok' | 'bad' | 'warn'

export interface ClaimFact {
  readonly id: 'checks' | 'commits' | 'deviation'
  readonly tone: ClaimFactTone
  readonly text: string
}

export interface ClaimEvidence {
  readonly runs: readonly CheckResult[]
  readonly commitCount: number
  readonly hasDeviation: boolean
}

/** What a claim shows under its title, so a person can weigh it without opening it. */
export function claimFacts({ runs, commitCount, hasDeviation }: ClaimEvidence): ClaimFact[] {
  const facts = [checkFact(runs), commitFact(commitCount)]
  if (hasDeviation) facts.push({ id: 'deviation', tone: 'warn', text: 'Deviation' })
  return facts
}

function checkFact(runs: readonly CheckResult[]): ClaimFact {
  const line = checksLine(runs)
  if (line === null) return { id: 'checks', tone: 'plain', text: 'No checks' }
  return { id: 'checks', tone: failedRuns(runs).length ? 'bad' : 'ok', text: line }
}

function commitFact(count: number): ClaimFact {
  const text = count === 0 ? 'No commits' : `${count} ${count === 1 ? 'commit' : 'commits'}`
  return { id: 'commits', tone: 'plain', text }
}
