import { apiClient, request } from '../../../api/client'
import { FeedbackListSchema, FeedbackSchema } from './schemas/feedback.schema'
import type { Feedback, WrittenFeedbackKind } from '@/features/feedback/model/feedback'
import type { FeedbackId, TaskId } from '@/model/branded'

export async function fetchFeedback(taskId: TaskId): Promise<Feedback[]> {
  return request(async () => FeedbackListSchema.parse(await apiClient(`/api/tasks/${taskId}/feedback`)))
}

export async function addFeedback(taskId: TaskId, kind: WrittenFeedbackKind, bodyMarkdown: string): Promise<Feedback> {
  return request(async () =>
    FeedbackSchema.parse(await apiClient(`/api/tasks/${taskId}/feedback`, { method: 'POST', body: { kind, bodyMarkdown } }))
  )
}

export async function editFeedback(id: FeedbackId, bodyMarkdown: string): Promise<Feedback> {
  return request(async () =>
    FeedbackSchema.parse(await apiClient(`/api/feedback/${id}`, { method: 'PATCH', body: { bodyMarkdown } }))
  )
}

export async function deleteFeedback(id: FeedbackId): Promise<void> {
  await request(async () => {
    await apiClient(`/api/feedback/${id}`, { method: 'DELETE' })
  })
}

export async function discardRevision(id: FeedbackId): Promise<void> {
  await request(async () => {
    await apiClient(`/api/feedback/${id}/discard`, { method: 'POST' })
  })
}
