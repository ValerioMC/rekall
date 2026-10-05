import type { Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/** A task on project `vega` whose label is its id, for specs that only care that it exists. */
export function taskFixture(id: TaskId): Task {
  return {
    id,
    label: id,
    title: id,
    status: 'TODO',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'acme',
    projectRepoFolder: null,
    documentCount: 0,
    stepCount: 0,
    stepsDone: 0,
    draftStepCount: 0,
    hasWrapup: false,
    reviewState: 'OPEN',
    reviewActive: false,
    claimedAt: null,
    acceptedAt: null,
    reviewNote: null,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    anchor: `project:vega task:${id}`,
    updatedAt: '2026-08-12T10:00:00Z'
  }
}
