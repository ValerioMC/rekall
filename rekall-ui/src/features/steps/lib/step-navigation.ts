import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

export interface StepNeighbours {
  readonly previous: TaskStep | null
  readonly next: TaskStep | null
}

/** The steps either side of the selected one in the list as shown; with none selected, the first is next. */
export function neighboursOf(shown: readonly TaskStep[], selectedId: TaskStepId | null): StepNeighbours {
  const index = shown.findIndex((step) => step.id === selectedId)
  return {
    previous: index > 0 ? (shown[index - 1] ?? null) : null,
    next: shown[index + 1] ?? null
  }
}

const TYPING_SURFACES = 'input, textarea, select, [contenteditable], [role="dialog"], .cm-editor'

/** Whether a key pressed on this element is meant for it, so a shortcut must leave it alone. */
export function isTypingTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest(TYPING_SURFACES) !== null
}
