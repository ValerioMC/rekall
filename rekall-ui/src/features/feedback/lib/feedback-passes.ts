import { relativeTime } from '@/common/format/relative-time'
import { isRunning, isWaiting, type Feedback } from '@/features/feedback/model/feedback'

/** The entries one Run revision took, with whether a session is still on them. */
export interface PassGroup {
  readonly pass: number
  readonly entries: readonly Feedback[]
  readonly running: boolean
  readonly takenAt: string | null
  readonly finishedAt: string | null
}

/**
 * The thread read as groups: what waits for the next run, each pass newest first, and the applied
 * entries from before passes existed.
 */
export interface ThreadGroups {
  readonly next: readonly Feedback[]
  readonly passes: readonly PassGroup[]
  readonly before: readonly Feedback[]
}

export function groupByPass(entries: readonly Feedback[]): ThreadGroups {
  const next = entries.filter(isWaiting)
  const before = entries.filter((entry) => entry.pass === null && !isWaiting(entry))
  const numbers = [...new Set(entries.flatMap((entry) => (entry.pass === null ? [] : [entry.pass])))]
  const passes = numbers.sort((a, b) => b - a).map((pass) => passGroup(pass, entries.filter((entry) => entry.pass === pass)))
  return { next, passes, before }
}

function passGroup(pass: number, entries: readonly Feedback[]): PassGroup {
  const running = entries.some(isRunning)
  return { pass, entries, running, takenAt: earliest(entries.map((entry) => entry.takenAt)), finishedAt: running ? null : latest(entries.map((entry) => entry.appliedAt)) }
}

function earliest(moments: readonly (string | null)[]): string | null {
  const present = moments.filter((moment): moment is string => moment !== null)
  return present.length === 0 ? null : present.reduce((first, moment) => (moment < first ? moment : first))
}

function latest(moments: readonly (string | null)[]): string | null {
  const present = moments.filter((moment): moment is string => moment !== null)
  return present.length === 0 ? null : present.reduce((last, moment) => (moment > last ? moment : last))
}

/** When a pass was started while it runs, and when it was finished once every entry is answered. */
export function passDetail(group: PassGroup, now: number = Date.now()): string {
  if (group.running) return group.takenAt === null ? 'in progress' : `started ${relativeTime(group.takenAt, now)}`
  return group.finishedAt === null ? '' : `finished ${relativeTime(group.finishedAt, now)}`
}

export function answeredCount(group: PassGroup): number {
  return group.entries.filter((entry) => !isRunning(entry)).length
}
