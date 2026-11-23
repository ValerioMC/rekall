import { isAnswered, isOnHold, isOpen, type Decision } from '@/features/decisions/model/decision'

export interface DecisionCounts {
  readonly total: number
  readonly answered: number
  readonly open: number
  readonly onHold: number
}

export function countDecisions(decisions: readonly Decision[]): DecisionCounts {
  return {
    total: decisions.length,
    answered: decisions.filter(isAnswered).length,
    open: decisions.filter(isOpen).length,
    onHold: decisions.filter(isOnHold).length
  }
}

function plural(count: number, word: string): string {
  return `${count} ${word}${count === 1 ? '' : 's'}`
}

/** What the band says under the progress: how many are settled, and whether the task is ready to run. */
export function describeTally(counts: DecisionCounts): { headline: string; rest: string } {
  if (counts.total === 0) return { headline: 'Nothing recorded yet.', rest: '' }
  const onHold = counts.onHold > 0 ? `, ${counts.onHold} on hold` : ''
  const rest = `${counts.answered} of ${counts.total} answered${onHold}.`
  if (counts.open > 0) return { headline: `${plural(counts.open, 'question')} still open.`, rest: `${rest} Loaded when it runs.` }
  if (counts.onHold > 0) return { headline: 'Nothing is open.', rest: `${rest} Loaded when it runs.` }
  return { headline: 'Everything is settled.', rest: `${rest} The task runs with all of it.` }
}
