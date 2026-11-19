import type { FeedbackKind } from '@/features/feedback/model/feedback'

const KIND_LABELS: Readonly<Record<FeedbackKind, string>> = {
  IMPLEMENTATION: 'On the work',
  PLAN: 'On the plan'
}

export function kindLabel(kind: FeedbackKind): string {
  return KIND_LABELS[kind]
}

export const TARGETS: readonly FeedbackKind[] = ['IMPLEMENTATION', 'PLAN']
