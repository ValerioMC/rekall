import type { WrapupVersion, WrapupVersionDetail } from '@/features/wrapup/model/wrapup-version'
import type { TaskId, WrapupVersionId } from '@/model/branded'

export const TASK = 't-1' as TaskId

export function version(number: number, writtenBy: 'CLAUDE' | 'HAND' = 'CLAUDE', commitHash: string | null = null): WrapupVersion {
  return {
    id: `v-${number}` as WrapupVersionId,
    taskId: TASK,
    number,
    writtenBy,
    writtenAt: '2026-11-03T10:00:00Z',
    length: 20 + number,
    preview: `State ${number}`,
    commitHash
  }
}

export function detail(number: number): WrapupVersionDetail {
  return { ...version(number), bodyMarkdown: `Whole text of state ${number}` }
}
