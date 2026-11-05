import type { Feedback } from '@/features/feedback/model/feedback'
import type { FeedbackId, TaskId } from '@/model/branded'

export const TASK = 't-1' as TaskId

export function entry(seq: number, extra: Partial<Feedback> = {}): Feedback {
  return {
    id: `f-${seq}` as FeedbackId,
    taskId: TASK,
    seq,
    kind: 'IMPLEMENTATION',
    state: 'PENDING',
    quotedMarkdown: null,
    bodyMarkdown: `Feedback number ${seq}`,
    replyMarkdown: null,
    createdAt: '2026-11-04T10:00:00Z',
    appliedAt: null,
    pass: null,
    takenAt: null,
    followsSeq: null,
    ...extra
  }
}

export function revision(seq: number, quoted: string, body: string, extra: Partial<Feedback> = {}): Feedback {
  return entry(seq, { kind: 'REVISION', quotedMarkdown: quoted, bodyMarkdown: body, ...extra })
}

export function applied(seq: number, reply: string, extra: Partial<Feedback> = {}): Feedback {
  return entry(seq, { state: 'APPLIED', replyMarkdown: reply, appliedAt: '2026-11-04T11:00:00Z', ...extra })
}

export function running(seq: number, pass: number, extra: Partial<Feedback> = {}): Feedback {
  return entry(seq, { state: 'RUNNING', pass, takenAt: '2026-11-04T10:30:00Z', ...extra })
}

export function appliedInPass(seq: number, pass: number, reply: string, extra: Partial<Feedback> = {}): Feedback {
  return applied(seq, reply, { pass, takenAt: '2026-11-04T10:30:00Z', ...extra })
}
