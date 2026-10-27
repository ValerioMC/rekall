import { describe, expect, it } from 'vitest'
import { partitionTasks } from '@/features/catalog/lib/partition-tasks'
import type { Task, TaskStatus } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const task = (id: string, status: TaskStatus): Task => ({
  id: id as TaskId,
  label: id,
  title: id,
  status,
  description: null,
  projectId: 'p1' as ProjectId,
  projectLabel: 'vega',
  projectTitle: 'Vega',
  companyName: 'acme',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0, draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  graphLaidAt: null,
  releaseSkipped: false,
  kind: 'TASK',
  epicId: null,
  anchor: `project:vega task:${id}`,
  updatedAt: '2026-09-04T10:00:00Z'
})

describe('partitionTasks', () => {
  it('files DONE tasks, shelves BACKLOG ones and keeps the rest active', () => {
    const { active, backlog, filed } = partitionTasks([
      task('a', 'IN_PROGRESS'),
      task('b', 'DONE'),
      task('c', 'TODO'),
      task('d', 'BLOCKED'),
      task('e', 'DONE'),
      task('f', 'BACKLOG')
    ])

    expect(active.map((t) => t.id)).toEqual(['a', 'c', 'd'])
    expect(backlog.map((t) => t.id)).toEqual(['f'])
    expect(filed.map((t) => t.id)).toEqual(['b', 'e'])
  })

  it('preserves the incoming order within each side', () => {
    const { filed } = partitionTasks([task('z', 'DONE'), task('a', 'DONE')])

    expect(filed.map((t) => t.id)).toEqual(['z', 'a'])
  })

  it('sets the epics apart whatever their status', () => {
    const { epics, active, filed } = partitionTasks([
      { ...task('a', 'IN_PROGRESS'), kind: 'EPIC' },
      { ...task('b', 'DONE'), kind: 'EPIC' },
      task('c', 'TODO')
    ])

    expect(epics.map((t) => t.id)).toEqual(['a', 'b'])
    expect(active.map((t) => t.id)).toEqual(['c'])
    expect(filed).toEqual([])
  })

  it('returns four empty lists for no tasks', () => {
    expect(partitionTasks([])).toEqual({ epics: [], active: [], backlog: [], filed: [] })
  })
})
