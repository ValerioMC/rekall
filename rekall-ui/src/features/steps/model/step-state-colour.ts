import type { TaskStepState } from './task-step'

/** One colour per step state, never shared: a claimed step waits for a person (accent), a running one is a session at work. */
export const STEP_STATE_DOT: Readonly<Record<TaskStepState, string>> = {
  DRAFT: 'bg-border-strong',
  OPEN: 'bg-text-subtle',
  RUNNING: 'bg-session',
  CLAIMED: 'bg-accent',
  DONE: 'bg-safe'
}

export const STEP_STATE_STANDING_TEXT: Readonly<Record<'RUNNING' | 'CLAIMED', string>> = {
  RUNNING: 'text-session-strong',
  CLAIMED: 'font-medium text-accent'
}

export function standingTextClass(state: TaskStepState): string {
  return state === 'RUNNING' || state === 'CLAIMED' ? STEP_STATE_STANDING_TEXT[state] : 'text-text-subtle'
}
