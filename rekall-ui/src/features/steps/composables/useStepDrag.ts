import { computed, ref, type ComputedRef, type Ref } from 'vue'
import type { StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

type DropTarget =
  | { readonly kind: 'edge'; readonly id: TaskStepId; readonly after: boolean }
  | { readonly kind: 'slot'; readonly before: TaskStepId | null }

export interface StepDropHandlers {
  /** A step or draft was dropped on the edge of an open step. */
  readonly onEdge: (moving: TaskStep, target: TaskStep, after: boolean) => void
  /** A step or draft was dropped on the slot in front of a session, or at the end when `before` is null. */
  readonly onSlot: (moving: TaskStep, before: TaskStep | null) => void
  /** A whole session was dropped on a slot. */
  readonly onSessionSlot: (session: StepSession, before: TaskStep | null) => void
}

export interface StepDrag {
  /** The step or draft being held. */
  readonly dragged: Ref<TaskStep | null>
  readonly draggedSession: Ref<StepSession | null>
  readonly holding: ComputedRef<boolean>
  /** Whether the step can take the dragged one next to itself: an open step other than that one. */
  accepts: (step: TaskStep) => boolean
  edgeOf: (step: TaskStep) => 'before' | 'after' | null
  slotActive: (before: TaskStep | null) => boolean
  start: (step: TaskStep) => void
  startSession: (session: StepSession) => void
  hover: (step: TaskStep, after: boolean) => void
  hoverSlot: (before: TaskStep | null) => void
  drop: (step: TaskStep) => void
  dropOnSlot: (before: TaskStep | null) => void
  end: () => void
}

/**
 * The state of dragging in the plan: a step, a draft off the shelf or a whole session is held, and
 * the pointer is over the edge of a step or over the slot between two sessions. A drop reports
 * where, and the handlers move it.
 */
export function useStepDrag(handlers: StepDropHandlers): StepDrag {
  const dragged = ref<TaskStep | null>(null)
  const draggedSession = ref<StepSession | null>(null)
  const over = ref<DropTarget | null>(null)
  const holding = computed(() => dragged.value !== null || draggedSession.value !== null)

  function accepts(step: TaskStep): boolean {
    return dragged.value !== null && step.state === 'OPEN' && step.id !== dragged.value.id
  }

  function edgeOf(step: TaskStep): 'before' | 'after' | null {
    if (over.value?.kind !== 'edge' || over.value.id !== step.id) return null
    return over.value.after ? 'after' : 'before'
  }

  function slotActive(before: TaskStep | null): boolean {
    return over.value?.kind === 'slot' && over.value.before === (before?.id ?? null)
  }

  function start(step: TaskStep): void {
    dragged.value = step
  }

  function startSession(session: StepSession): void {
    draggedSession.value = session
  }

  function hover(step: TaskStep, after: boolean): void {
    if (accepts(step)) over.value = { kind: 'edge', id: step.id, after }
  }

  function hoverSlot(before: TaskStep | null): void {
    if (holding.value) over.value = { kind: 'slot', before: before?.id ?? null }
  }

  function end(): void {
    dragged.value = null
    draggedSession.value = null
    over.value = null
  }

  function drop(step: TaskStep): void {
    const moving = dragged.value
    const edge = over.value
    end()
    if (moving && edge?.kind === 'edge' && edge.id === step.id) handlers.onEdge(moving, step, edge.after)
  }

  function dropOnSlot(before: TaskStep | null): void {
    const moving = dragged.value
    const session = draggedSession.value
    end()
    if (moving) handlers.onSlot(moving, before)
    else if (session) handlers.onSessionSlot(session, before)
  }

  return { dragged, draggedSession, holding, accepts, edgeOf, slotActive, start, startSession, hover, hoverSlot, drop, dropOnSlot, end }
}
