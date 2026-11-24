import type { ComputedRef } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import {
  layoutOf,
  planRelayout,
  planSessionMove,
  planStepAlone,
  planStepMove,
  sessionLanding,
  type PlanLayout,
  type Relayout,
  type SessionChange,
  type StepMove
} from '@/features/steps/lib/step-order'
import type { StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'

export interface PlanMoves {
  applySessionChanges: (changes: readonly SessionChange[]) => Promise<void>
  /** Puts an open step before or after another, regrouping the sessions around it as the move needs. */
  reorder: (step: TaskStep, target: TaskStep, after: boolean) => Promise<void>
  /** The keyboard's move: one place up or down, never past a step that has been started. */
  nudge: (step: TaskStep, by: number) => Promise<void>
  runIn: (step: TaskStep, session: StepSession) => Promise<void>
  /** Runs the step in a session of its own in front of `before`, or at the end. */
  placeAlone: (step: TaskStep, before: TaskStep | null) => Promise<void>
  moveSession: (session: StepSession, before: TaskStep | null) => Promise<void>
  /** Moves a whole session one session up or down. */
  nudgeSession: (session: StepSession, by: number) => Promise<void>
  promoteBeside: (draft: TaskStep, target: TaskStep, after: boolean) => Promise<void>
  promoteAlone: (draft: TaskStep, before: TaskStep | null) => Promise<void>
  promoteInto: (draft: TaskStep, session: StepSession) => Promise<void>
}

interface PlanMovesSource {
  readonly steps: ComputedRef<readonly TaskStep[]>
  readonly sessions: ComputedRef<readonly StepSession[]>
  readonly flush: () => void
}

function relayoutOf(steps: readonly TaskStep[], plan: StepMove, step: TaskStep): Relayout {
  const staysPut = steps.indexOf(step) === plan.position
  return { moves: staysPut ? [] : [{ step, position: plan.position }], sessionChanges: plan.sessionChanges }
}

/**
 * Every way of changing where an open step runs in the plan. A move is a list of lifts and of
 * session flags, applied one request at a time; each one that a person makes offers Undo, which
 * lays the checklist out as it was.
 */
export function usePlanMoves(source: PlanMovesSource): PlanMoves {
  const store = useConsoleStore()
  const toast = useToastStore()
  const { run } = useAsyncAction()

  async function applySessionChanges(changes: readonly SessionChange[]): Promise<void> {
    for (const change of changes) await store.beginStepSession(change.step.id, change.startsSession)
  }

  async function applyRelayout(plan: Relayout): Promise<void> {
    for (const move of plan.moves) await store.moveStep(move.step.id, move.position)
    await applySessionChanges(plan.sessionChanges)
  }

  async function restore(layout: PlanLayout): Promise<void> {
    const plan = planRelayout(source.steps.value, layout)
    if (plan) await applyRelayout(plan)
  }

  async function settle(plan: Relayout | null, message: string | null): Promise<void> {
    if (!plan) return
    const before = layoutOf(source.steps.value)
    source.flush()
    const outcome = await run(() => applyRelayout(plan))
    if (outcome === null || message === null) return
    toast.notify(message, { label: 'Undo', run: () => void run(() => restore(before)) })
  }

  async function moveBeside(step: TaskStep, target: TaskStep, after: boolean, message: string | null): Promise<void> {
    const plan = planStepMove(source.steps.value, step, target, after)
    await settle(plan && relayoutOf(source.steps.value, plan, step), message)
  }

  const reorder = (step: TaskStep, target: TaskStep, after: boolean): Promise<void> =>
    moveBeside(step, target, after, `Moved ${step.title}`)

  async function nudge(step: TaskStep, by: number): Promise<void> {
    const neighbour = source.steps.value[source.steps.value.indexOf(step) + by]
    if (step.state !== 'OPEN' || neighbour?.state !== 'OPEN') return
    await moveBeside(step, neighbour, by > 0, null)
  }

  async function runIn(step: TaskStep, session: StepSession): Promise<void> {
    const landing = sessionLanding(source.sessions.value, step, session)
    if (landing) await reorder(step, landing.target, landing.after)
  }

  const placeAlone = (step: TaskStep, before: TaskStep | null): Promise<void> =>
    settle(planStepAlone(source.steps.value, step, before), `${step.title} runs in a session of its own`)

  const moveSession = (session: StepSession, before: TaskStep | null): Promise<void> =>
    settle(planSessionMove(source.steps.value, session, before), `Moved Session ${session.number}`)

  async function nudgeSession(session: StepSession, by: number): Promise<void> {
    const sessions = source.sessions.value
    const index = sessions.findIndex((candidate) => candidate.number === session.number)
    if (index + by < 0 || index + by >= sessions.length) return
    const before = by < 0 ? sessions[index - 1]?.steps[0] : sessions[index + 2]?.steps[0]
    await moveSession(session, before ?? null)
  }

  async function promoteWhere(
    draft: TaskStep,
    planFor: (steps: readonly TaskStep[], promoted: TaskStep) => Relayout | null
  ): Promise<void> {
    source.flush()
    const before = layoutOf(source.steps.value)
    const promoted = await run(async () => {
      await store.promoteStep(draft.id)
      const placed = source.steps.value.find((step) => step.id === draft.id) ?? null
      const plan = placed ? planFor(source.steps.value, placed) : null
      if (plan) await applyRelayout(plan)
      return placed
    })
    if (!promoted) return
    toast.notify(`Promoted ${draft.title}`, {
      label: 'Undo',
      run: () =>
        void run(async () => {
          await store.returnStepToDraft(draft.id)
          await restore(before)
        })
    })
  }

  const promoteBeside = (draft: TaskStep, target: TaskStep, after: boolean): Promise<void> =>
    promoteWhere(draft, (steps, promoted) => {
      const plan = planStepMove(steps, promoted, target, after)
      return plan && relayoutOf(steps, plan, promoted)
    })

  const promoteAlone = (draft: TaskStep, before: TaskStep | null): Promise<void> =>
    promoteWhere(draft, (steps, promoted) => planStepAlone(steps, promoted, before))

  async function promoteInto(draft: TaskStep, session: StepSession): Promise<void> {
    const last = session.steps[session.steps.length - 1]
    if (last) await promoteBeside(draft, last, true)
  }

  return { applySessionChanges, reorder, nudge, runIn, placeAlone, moveSession, nudgeSession, promoteBeside, promoteAlone, promoteInto }
}
