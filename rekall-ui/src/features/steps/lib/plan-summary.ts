export interface PlanSummaryCounts {
  readonly done: number
  readonly claimed: number
  readonly total: number
  /** The checklist number of the step a session is working on, or null when none is. */
  readonly runningNumber: number | null
}

export interface PlanSummary {
  /** What waits on the reader or what is happening now; empty when the plan is only a tally. */
  readonly lead: string
  readonly tally: string
  readonly waitsForReader: boolean
}

function plural(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`
}

export function planSummary(counts: PlanSummaryCounts): PlanSummary {
  const tally = `${counts.done} of ${counts.total} done`
  if (counts.claimed > 0) {
    const verbs = counts.claimed === 1 ? 'is claimed and waits' : 'are claimed and wait'
    return { lead: `${plural(counts.claimed, 'step')} ${verbs} for you.`, tally, waitsForReader: true }
  }
  if (counts.runningNumber !== null) {
    return { lead: `A session is working on step ${counts.runningNumber}.`, tally, waitsForReader: false }
  }
  return { lead: '', tally, waitsForReader: false }
}
