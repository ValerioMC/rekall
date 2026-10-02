import { describe, expect, it } from 'vitest'
import { diagramsByTask, taskDiagrams } from '@/common/diagram/task-diagrams'
import type { DiagramSummary } from '@/model/diagram'
import type { DiagramId, ProjectId, TaskId } from '@/model/branded'

const TASK = 't-1' as TaskId
const OTHER = 't-2' as TaskId

function summary(id: string, taskId: TaskId | null, updatedAt: string): DiagramSummary {
  return {
    id: id as DiagramId,
    projectId: 'p-1' as ProjectId,
    taskId,
    title: id,
    question: '',
    nodeCount: 3,
    edgeCount: 2,
    createdAt: updatedAt,
    updatedAt
  }
}

describe('diagramsByTask', () => {
  it('groups by task, newest first, and leaves out diagrams tied to no task', () => {
    const grouped = diagramsByTask([
      summary('old', TASK, '2026-09-01T10:00:00Z'),
      summary('loose', null, '2026-09-03T10:00:00Z'),
      summary('new', TASK, '2026-09-02T10:00:00Z')
    ])
    expect(grouped.get(TASK)?.map((diagram) => diagram.id)).toEqual(['new', 'old'])
    expect(grouped.size).toBe(1)
  })
})

describe('taskDiagrams', () => {
  const byTask = diagramsByTask([summary('d', TASK, '2026-09-01T10:00:00Z')])

  it('is none for a task with no diagram and no generation', () => {
    expect(taskDiagrams(OTHER, byTask, new Set()).phase).toBe('none')
  })

  it('is ready once a diagram exists', () => {
    expect(taskDiagrams(TASK, byTask, new Set()).phase).toBe('ready')
  })

  it('is generating while a session works, even before any diagram exists', () => {
    expect(taskDiagrams(OTHER, byTask, new Set([OTHER])).phase).toBe('generating')
  })

  it('keeps the earlier diagrams openable while a regeneration runs', () => {
    const result = taskDiagrams(TASK, byTask, new Set([TASK]))
    expect(result.phase).toBe('generating')
    expect(result.diagrams).toHaveLength(1)
  })
})
