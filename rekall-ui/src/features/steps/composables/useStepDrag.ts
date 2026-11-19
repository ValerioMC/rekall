import { computed, ref, type ComputedRef, type Ref } from 'vue'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

interface DropEdge {
  readonly id: TaskStepId
  readonly after: boolean
}

export interface StepDrag {
  readonly dragged: Ref<TaskStep | null>
  /** Whether the step can take the dragged one next to itself: an open step other than that one. */
  accepts: (step: TaskStep) => boolean
  edgeOf: (step: TaskStep) => 'before' | 'after' | null
  start: (step: TaskStep) => void
  hover: (step: TaskStep, after: boolean) => void
  drop: (step: TaskStep) => void
  end: () => void
}

/**
 * The state of dragging an open step to another place in the plan: which step is held, and the
 * edge of which step it would land on. A drop reports where, and the caller moves the step.
 */
export function useStepDrag(onDrop: (moving: TaskStep, target: TaskStep, after: boolean) => void): StepDrag {
  const dragged = ref<TaskStep | null>(null)
  const over = ref<DropEdge | null>(null)
  const held: ComputedRef<TaskStepId | null> = computed(() => dragged.value?.id ?? null)

  function accepts(step: TaskStep): boolean {
    return held.value !== null && step.state === 'OPEN' && step.id !== held.value
  }

  function edgeOf(step: TaskStep): 'before' | 'after' | null {
    if (over.value?.id !== step.id) return null
    return over.value.after ? 'after' : 'before'
  }

  function start(step: TaskStep): void {
    dragged.value = step
  }

  function hover(step: TaskStep, after: boolean): void {
    if (accepts(step)) over.value = { id: step.id, after }
  }

  function end(): void {
    dragged.value = null
    over.value = null
  }

  function drop(step: TaskStep): void {
    const moving = dragged.value
    const edge = over.value
    end()
    if (moving && edge?.id === step.id) onDrop(moving, step, edge.after)
  }

  return { dragged, accepts, edgeOf, start, hover, drop, end }
}
