import type { Decision } from '@/features/decisions/model/decision'
import type { DecisionId, TaskId } from '@/model/branded'

export const TASK = 't-1' as TaskId

export function decision(key: string, extra: Partial<Decision> = {}): Decision {
  return {
    id: `d-${key}` as DecisionId,
    taskId: TASK,
    key,
    topic: null,
    question: `Question of ${key}?`,
    why: null,
    stepNumber: null,
    stepTitle: null,
    state: 'OPEN',
    answer: null,
    holdNote: null,
    previousAnswer: null,
    changedAt: null,
    source: 'TERMINAL',
    options: [],
    lines: [],
    details: [],
    ...extra
  }
}

export function answered(key: string, answer: string, extra: Partial<Decision> = {}): Decision {
  return decision(key, { state: 'ANSWERED', answer, ...extra })
}

export function onHold(key: string, note: string, extra: Partial<Decision> = {}): Decision {
  return decision(key, { state: 'ON_HOLD', holdNote: note, ...extra })
}
