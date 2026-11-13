import type { TaskId, TimeEntryId } from '../../../model/branded'

export interface TimeEntry {
  readonly id: TimeEntryId
  readonly taskId: TaskId
  readonly taskCode: string
  readonly taskTitle: string
  readonly projectLabel: string
  readonly anchor: string
  readonly startedAt: string
  readonly stoppedAt: string | null
  readonly createdAt: string
  readonly updatedAt: string
}
