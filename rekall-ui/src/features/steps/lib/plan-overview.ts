import { relativeTime } from '@/common/format/relative-time'
import type { TaskStep } from '@/features/steps/model/task-step'

export interface PlanGroups {
  readonly waiting: readonly TaskStep[]
  readonly working: readonly TaskStep[]
  readonly upNext: readonly TaskStep[]
  readonly done: readonly TaskStep[]
}

export interface QueueFacts {
  /** What the checks found, as `checksLine` writes it; null when none ran. */
  readonly checks: string | null
  readonly commits: number
}

/** The checklist steps sorted by what they ask of the reader, each group in checklist order. */
export function groupSteps(steps: readonly TaskStep[]): PlanGroups {
  const inState = (state: TaskStep['state']) => steps.filter((step) => step.state === state)
  return { waiting: inState('CLAIMED'), working: inState('RUNNING'), upNext: inState('OPEN'), done: inState('DONE') }
}

function plural(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`
}

/** What the overview leads with: what waits on the reader, else what is happening, else where the plan stands. */
export function overviewTitle(groups: PlanGroups, runningNumber: number | null): string {
  if (groups.waiting.length) return `${plural(groups.waiting.length, 'step')} ${groups.waiting.length === 1 ? 'waits' : 'wait'} for you`
  if (runningNumber !== null) return `A session is working on step ${runningNumber}`
  if (!groups.upNext.length) return 'Every step is done'
  return 'Nothing waits for you'
}

function firstLineOf(markdown: string | null): string | null {
  const line = markdown?.split('\n').find((candidate) => candidate.trim() !== '')
  return line ? line.trim() : null
}

/** The one line a step's row in the overview says under its title, from what the step holds. */
export function queueNote(step: TaskStep, facts: QueueFacts): string {
  const commits = facts.commits ? plural(facts.commits, 'commit') : null
  switch (step.state) {
    case 'CLAIMED':
      return [
        facts.checks,
        commits,
        `pass ${step.passes.length + 1}`,
        step.claimedAt ? `claimed ${relativeTime(step.claimedAt)}` : null
      ]
        .filter((part): part is string => part !== null)
        .join(' · ')
    case 'RUNNING':
      return [commits ? `${commits} so far` : null, 'a session is on it'].filter((part): part is string => part !== null).join(' · ')
    case 'DONE':
      return step.doneAt ? `Done ${relativeTime(step.doneAt)}` : 'Done'
    default:
      return firstLineOf(step.bodyMarkdown) ?? 'No detail written yet'
  }
}
