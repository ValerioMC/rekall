import { describe, expect, it } from 'vitest'
import { partitionTasks } from '@/common/catalog/partition-tasks'
import type { Task, TaskStatus } from '@/model/catalog'
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
  stepsDone: 0,
  hasWrapup: false,
  anchor: `project:vega task:${id}`,
  updatedAt: '2026-09-04T10:00:00Z'
})

describe('partitionTasks', () => {
  it('files DONE tasks and keeps the rest active', () => {
    const { active, filed } = partitionTasks([
      task('a', 'IN_PROGRESS'),
      task('b', 'DONE'),
      task('c', 'TODO'),
      task('d', 'BLOCKED'),
      task('e', 'DONE')
    ])

    expect(active.map((t) => t.id)).toEqual(['a', 'c', 'd'])
    expect(filed.map((t) => t.id)).toEqual(['b', 'e'])
  })

  it('preserves the incoming order within each side', () => {
    const { filed } = partitionTasks([task('z', 'DONE'), task('a', 'DONE')])

    expect(filed.map((t) => t.id)).toEqual(['z', 'a'])
  })

  it('returns two empty lists for no tasks', () => {
    expect(partitionTasks([])).toEqual({ active: [], filed: [] })
  })
})
