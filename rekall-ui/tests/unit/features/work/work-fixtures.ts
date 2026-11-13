import type { Project, Task } from '@/features/catalog/model/catalog'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { CompanyId, ProjectId, TaskId, TaskStepId } from '@/model/branded'

export function task(id: string, overrides: Partial<Task> = {}): Task {
  return {
    id: id as TaskId,
    code: id,
    label: id,
    title: `Task ${id}`,
    status: 'IN_PROGRESS',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'Acme',
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
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    graphLaidAt: null,
    releaseSkipped: false,
    kind: 'TASK',
    epicId: null,
    quick: false,
    autoAccepted: false,
    dependsOn: [],
    trackerIssueKey: null,
    trackerIssueUrl: null,
    trackerSyncError: null,
    trackerContentSynced: true,
    pullRequestNumber: null,
    pullRequestUrl: null,
    pullRequestFailure: null,
    pullRequestMergedAt: null,
    anchor: `project:vega task:${id}`,
    updatedAt: '2026-09-20T10:00:00Z',
    ...overrides
  }
}

export function step(id: string, taskId: string, state: TaskStepState): TaskStep {
  return {
    id: id as TaskStepId,
    taskId: taskId as TaskId,
    title: `Step ${id}`,
    bodyMarkdown: null,
    state,
    done: state === 'DONE',
    runningAt: null,
    claimedAt: null,
    doneAt: null,
    position: 0,
    flowX: null,
    flowY: null,
    maxVisits: null,
    startsSession: false,
    connectionId: null,
    passes: [],
    links: [],
    visits: [],
    createdAt: '2026-09-20T09:00:00Z',
    updatedAt: '2026-09-20T09:00:00Z'
  }
}

export function project(id: string, title: string, overrides: Partial<Project> = {}): Project {
  return {
    id: id as ProjectId,
    label: id,
    taskKey: 'PR',
    title,
    status: 'ACTIVE',
    icon: 'folder',
    iconImageVersion: null,
    description: null,
    blueprintMarkdown: null,
    repoFolder: null,
    autoCommit: false,
    mainBranch: null,
    pullRequestFlow: false,
    companyId: 'c1' as CompanyId,
    companyName: 'Acme',
    taskCount: 0,
    anchor: `project:${id}`,
    updatedAt: '2026-09-20T10:00:00Z',
    ...overrides
  }
}
