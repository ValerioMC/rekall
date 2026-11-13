import { describe, expect, it } from 'vitest'
import { branchesWithStatus, partitionTasks } from '@/features/catalog/lib/partition-tasks'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStatus } from '@/features/catalog/model/task-status'
import type { ProjectId, TaskId } from '@/model/branded'

const task = (id: string, status: TaskStatus): Task => ({
  id: id as TaskId,
  code: id, label: id,
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
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  graphLaidAt: null,
  releaseSkipped: false,
  kind: 'TASK',
  epicId: null, quick: false, autoAccepted: false, dependsOn: [], trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null, trackerContentSynced: true, pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null,
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

    expect(active.map((t) => t.id)).toEqual(expect.arrayContaining(['a', 'c', 'd']))
    expect(active).toHaveLength(3)
    expect(backlog.map((t) => t.id)).toEqual(['f'])
    expect(filed).toHaveLength(2)
  })

  it('lists each side newest first, by the number its code ends with', () => {
    const numbered = (number: number, status: TaskStatus): Task => ({ ...task(`t${number}`, status), code: `RK-${number}` })

    const { filed, active } = partitionTasks([numbered(2, 'DONE'), numbered(10, 'DONE'), numbered(9, 'TODO'), numbered(11, 'TODO')])

    expect(filed.map((t) => t.code)).toEqual(['RK-10', 'RK-2'])
    expect(active.map((t) => t.code)).toEqual(['RK-11', 'RK-9'])
  })

  it('gathers the tasks of an epic under it, whatever their status, and lists them nowhere else', () => {
    const epic: Task = { ...task('e', 'TODO'), kind: 'EPIC', code: 'RK-1' }
    const under = (id: string, status: TaskStatus, number: number): Task => ({ ...task(id, status), epicId: epic.id, code: `RK-${number}` })

    const { epics, active, filed } = partitionTasks([epic, under('a', 'IN_PROGRESS', 2), under('b', 'DONE', 3), task('c', 'TODO')])

    expect(epics.map((branch) => [branch.epic.id, branch.children.map((child) => child.id)])).toEqual([['e', ['b', 'a']]])
    expect(active.map((t) => t.id)).toEqual(['c'])
    expect(filed).toEqual([])
  })

  it('lists a task as an ordinary one when its epic is not in the list', () => {
    const orphan: Task = { ...task('a', 'TODO'), epicId: 'gone' as TaskId }

    const { epics, active } = partitionTasks([orphan])

    expect(epics).toEqual([])
    expect(active.map((t) => t.id)).toEqual(['a'])
  })

  it('keeps an epic with the status it has, for the section that lists it', () => {
    const { epics } = partitionTasks([{ ...task('a', 'IN_PROGRESS'), kind: 'EPIC' }, { ...task('b', 'DONE'), kind: 'EPIC' }])

    expect(branchesWithStatus(epics, 'DONE').map((branch) => branch.epic.id)).toEqual(['b'])
  })

  it('returns four empty lists for no tasks', () => {
    expect(partitionTasks([])).toEqual({ epics: [], active: [], backlog: [], filed: [] })
  })
})
