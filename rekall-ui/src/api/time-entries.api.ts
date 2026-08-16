import { apiClient, request } from './client'
import {
  TimeEntryListSchema,
  TimeEntrySchema,
  TimeEntryStartResultSchema
} from './schemas/catalog.schema'
import type { TimeEntry } from '@/model/catalog'
import type { TaskId, TimeEntryId } from '@/model/branded'

/** What starting a timer produced: the session now open, and whatever it had to close. */
export interface TimeEntryStartResult {
  started: TimeEntry
  stoppedElsewhere: TimeEntry | null
}

export interface TimeEntryEdit {
  startedAt: string
  stoppedAt: string | null
}

/** Every session, loaded whole for the same reason the wrapups are: no per-task round trip. */
export async function fetchTimeEntries(): Promise<TimeEntry[]> {
  return request(async () => TimeEntryListSchema.parse(await apiClient('/api/time-entries')))
}

/** Opens a session on this task, closing whatever else was running. */
export async function startTimeEntry(taskId: TaskId): Promise<TimeEntryStartResult> {
  return request(async () =>
    TimeEntryStartResultSchema.parse(
      await apiClient(`/api/tasks/${taskId}/time-entries/start`, { method: 'POST' })
    )
  )
}

/** Closes the session open on this task. */
export async function stopTimeEntry(taskId: TaskId): Promise<TimeEntry> {
  return request(async () =>
    TimeEntrySchema.parse(
      await apiClient(`/api/tasks/${taskId}/time-entries/stop`, { method: 'POST' })
    )
  )
}

export async function editTimeEntry(id: TimeEntryId, input: TimeEntryEdit): Promise<TimeEntry> {
  return request(async () =>
    TimeEntrySchema.parse(
      await apiClient(`/api/time-entries/${id}`, { method: 'PATCH', body: input })
    )
  )
}

export async function deleteTimeEntry(id: TimeEntryId): Promise<void> {
  await request(() => apiClient(`/api/time-entries/${id}`, { method: 'DELETE' }))
}
