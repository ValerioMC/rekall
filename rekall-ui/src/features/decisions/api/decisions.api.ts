import { apiClient, request } from '../../../api/client'
import { DecisionResponseSchema, TaskDecisionsResponseSchema } from './schemas/decision.schema'
import type { Decision, DecisionEdit, NewDecision, TaskDecisions } from '@/features/decisions/model/decision'
import type { DecisionId, TaskId } from '@/model/branded'

type Answered = ReturnType<typeof DecisionResponseSchema.parse>

function ofTask(taskId: TaskId, response: Answered): Decision {
  return { ...response, taskId }
}

export async function fetchDecisions(taskId: TaskId): Promise<TaskDecisions> {
  return request(async () => {
    const parsed = TaskDecisionsResponseSchema.parse(await apiClient(`/api/tasks/${taskId}/decisions`))
    return { decisions: parsed.decisions.map((decision) => ofTask(taskId, decision)), sessionBlock: parsed.sessionBlock }
  })
}

/** A decision the person writes by hand: answered from the start. */
export async function addDecision(taskId: TaskId, decision: NewDecision): Promise<Decision> {
  return request(async () =>
    ofTask(
      taskId,
      DecisionResponseSchema.parse(await apiClient(`/api/tasks/${taskId}/decisions`, { method: 'POST', body: decision }))
    )
  )
}

export async function editDecision(taskId: TaskId, id: DecisionId, edit: DecisionEdit): Promise<Decision> {
  return request(async () =>
    ofTask(
      taskId,
      DecisionResponseSchema.parse(await apiClient(`/api/decisions/${id}`, { method: 'PUT', body: edit }))
    )
  )
}

export async function removeDecision(id: DecisionId): Promise<void> {
  await request(async () => {
    await apiClient(`/api/decisions/${id}`, { method: 'DELETE' })
  })
}
