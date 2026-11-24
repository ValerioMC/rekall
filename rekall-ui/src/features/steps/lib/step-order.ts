import type { StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

export interface SessionChange {
  readonly step: TaskStep
  readonly startsSession: boolean
}

export interface StepMove {
  /** Where the step lands among the checklist once it is lifted out. */
  readonly position: number
  readonly sessionChanges: readonly SessionChange[]
}

/**
 * Taking a step that opens a session out of its place hands the opening to the step behind it, so
 * the steps that followed do not slide into the session above.
 */
export function openerHandover(steps: readonly TaskStep[], leaving: TaskStep): SessionChange[] {
  const index = steps.findIndex((step) => step.id === leaving.id)
  const successor = steps[index + 1]
  if (index <= 0 || !leaving.startsSession || !successor || successor.startsSession) return []
  return [{ step: successor, startsSession: true }]
}

/**
 * Puts `moving` right before or after `target`. It joins the session it lands in, and lands at the
 * head of that session when it goes in front of the step that opens it. Answers null when the step
 * would change nothing, neither its place nor a session.
 */
export function planStepMove(steps: readonly TaskStep[], moving: TaskStep, target: TaskStep, after: boolean): StepMove | null {
  if (moving.id === target.id) return null
  const flags = new Map<string, boolean>(steps.map((step) => [step.id, step.startsSession]))
  openerHandover(steps, moving).forEach((change) => flags.set(change.step.id, change.startsSession))

  const remaining = steps.filter((step) => step.id !== moving.id)
  const targetIndex = remaining.findIndex((step) => step.id === target.id)
  const position = after ? targetIndex + 1 : targetIndex

  const targetOpens = flags.get(target.id) === true
  flags.set(moving.id, !after && targetOpens && position > 0)
  if (!after && targetOpens) flags.set(target.id, false)

  const sessionChanges = steps
    .filter((step) => flags.get(step.id) !== step.startsSession)
    .map((step) => ({ step, startsSession: flags.get(step.id) === true }))
  const staysPut = position === steps.findIndex((step) => step.id === moving.id)
  return staysPut && !sessionChanges.length ? null : { position, sessionChanges }
}

export interface LayoutEntry {
  readonly id: TaskStepId
  readonly startsSession: boolean
}

/** The checklist as it is laid out: the steps in order and which of them open a session. */
export type PlanLayout = readonly LayoutEntry[]

export interface StepRelocation {
  readonly step: TaskStep
  /** Where the step lands among the checklist once it is lifted out; apply them in the order given. */
  readonly position: number
}

export interface Relayout {
  readonly moves: readonly StepRelocation[]
  readonly sessionChanges: readonly SessionChange[]
}

export function layoutOf(steps: readonly TaskStep[]): PlanLayout {
  return steps.map((step) => ({ id: step.id, startsSession: step.startsSession }))
}

/** The positions of the longest run that is already in order: the steps that need not move. */
function stayingPositions(positions: readonly number[]): Set<number> {
  const lengths: number[] = []
  const previous: number[] = []
  let best = -1
  positions.forEach((position, index) => {
    lengths[index] = 1
    previous[index] = -1
    for (let before = 0; before < index; before++) {
      if (positions[before]! < position && lengths[before]! + 1 > lengths[index]!) {
        lengths[index] = lengths[before]! + 1
        previous[index] = before
      }
    }
    if (best < 0 || lengths[index]! > lengths[best]!) best = index
  })
  const staying = new Set<number>()
  for (let index = best; index >= 0; index = previous[index]!) staying.add(index)
  return staying
}

/**
 * The moves and session changes that turn the checklist into `layout`: the fewest lifts, each
 * setting a step down behind the one that precedes it in the layout. The first step opens its
 * session whatever it says, so its flag is never compared. Answers null when nothing differs.
 */
export function planRelayout(steps: readonly TaskStep[], layout: PlanLayout): Relayout | null {
  const byId = new Map(steps.map((step) => [step.id, step]))
  const wanted = layout.flatMap((entry, index) => {
    const step = byId.get(entry.id)
    return step ? [{ step, index, startsSession: entry.startsSession }] : []
  })
  const staying = stayingPositions(wanted.map(({ step }) => steps.indexOf(step)))
  const current = [...steps]
  const moves: StepRelocation[] = []
  wanted.forEach(({ step }, index) => {
    if (staying.has(index)) return
    current.splice(current.indexOf(step), 1)
    const predecessor = wanted[index - 1]?.step
    const position = predecessor ? current.indexOf(predecessor) + 1 : 0
    current.splice(position, 0, step)
    moves.push({ step, position })
  })
  const sessionChanges = wanted
    .filter(({ step, index, startsSession }) => index > 0 && step.startsSession !== startsSession)
    .map(({ step, startsSession }) => ({ step, startsSession }))
  return moves.length || sessionChanges.length ? { moves, sessionChanges } : null
}

function lastStartedIndex(steps: readonly TaskStep[]): number {
  return steps.reduce((last, step, index) => (step.state === 'OPEN' ? last : index), -1)
}

/**
 * Lifts `block` out and sets it down in front of `before`, or at the end. It opens a session of
 * its own and the step behind it opens the next, so it joins neither neighbour. Nothing lands
 * above a step that has been started.
 */
function planBlockMove(
  steps: readonly TaskStep[],
  block: readonly TaskStep[],
  before: TaskStep | null,
  handover: readonly SessionChange[]
): Relayout | null {
  const flags = new Map<string, boolean>(steps.map((step) => [step.id, step.startsSession]))
  handover.forEach((change) => flags.set(change.step.id, change.startsSession))
  const lifted = new Set<string>(block.map((step) => step.id))
  const remaining = steps.filter((step) => !lifted.has(step.id))
  const index = before ? remaining.findIndex((step) => step.id === before.id) : remaining.length
  if (index < 0 || index <= lastStartedIndex(remaining)) return null
  const order = [...remaining.slice(0, index), ...block, ...remaining.slice(index)]
  flags.set(block[0]!.id, true)
  const successor = order[index + block.length]
  if (successor) flags.set(successor.id, true)
  return planRelayout(
    steps,
    order.map((step) => ({ id: step.id, startsSession: flags.get(step.id) === true }))
  )
}

/** Runs one step in a session of its own in front of `before`, or at the end. Putting it in front of itself splits its session there. */
export function planStepAlone(steps: readonly TaskStep[], moving: TaskStep, before: TaskStep | null): Relayout | null {
  const index = steps.findIndex((step) => step.id === moving.id)
  const anchor = before?.id === moving.id ? (steps[index + 1] ?? null) : before
  return planBlockMove(steps, [moving], anchor, openerHandover(steps, moving))
}

/** Runs a whole session in front of `before`, or last, keeping its steps together in their order. */
export function planSessionMove(steps: readonly TaskStep[], session: StepSession, before: TaskStep | null): Relayout | null {
  return planBlockMove(steps, session.steps, before, [])
}

/** A session can be lifted only while none of its steps has been started. */
export function sessionIsMovable(session: StepSession): boolean {
  return session.steps.every((step) => step.state === 'OPEN')
}

export interface Landing {
  readonly target: TaskStep
  readonly after: boolean
}

/** Where a step lands to join `session`: behind the last step of an earlier one, at the head of a later one. */
export function sessionLanding(sessions: readonly StepSession[], moving: TaskStep, session: StepSession): Landing | null {
  const own = sessions.find((candidate) => candidate.steps.some((step) => step.id === moving.id))
  const earlier = own !== undefined && own.number > session.number
  const target = earlier ? session.steps[session.steps.length - 1] : session.steps[0]
  return target ? { target, after: earlier } : null
}
