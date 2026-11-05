import type { FeedbackKind, WrittenFeedbackKind } from '@/features/feedback/model/feedback'

const KIND_LABELS: Readonly<Record<FeedbackKind, string>> = {
  REVISION: 'Revision',
  IMPLEMENTATION: 'Feedback on the work',
  PLAN: 'Feedback on the plan'
}

const SWITCH_LABELS: Readonly<Record<WrittenFeedbackKind, string>> = {
  IMPLEMENTATION: 'On the work',
  PLAN: 'On the plan'
}

export function kindLabel(kind: FeedbackKind): string {
  return KIND_LABELS[kind]
}

export function switchLabel(kind: WrittenFeedbackKind): string {
  return SWITCH_LABELS[kind]
}

export const WRITTEN_KINDS: readonly WrittenFeedbackKind[] = ['IMPLEMENTATION', 'PLAN']
