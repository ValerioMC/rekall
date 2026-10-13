import type { StepLink, StepVisit, Task, TaskStep, TaskStepState } from '@/model/catalog'
import type { ProjectId, StepLinkId, StepVisitId, TaskId, TaskStepId } from '@/model/branded'

/** A release flow: build, deploy, test, a fix that loops back to deploy. */
export const TASK = 'task-release' as TaskId

export function flowStep(
  id: string,
  position: number,
  state: TaskStepState = 'OPEN',
  links: readonly StepLink[] = [],
  visits: readonly StepVisit[] = []
): TaskStep {
  return {
    id: id as TaskStepId,
    taskId: TASK,
    title: id.charAt(0).toUpperCase() + id.slice(1),
    bodyMarkdown: null,
    state,
    done: state === 'DONE',
    runningAt: null,
    claimedAt: null,
    doneAt: null,
    position,
    passes: [],
    links,
    visits,
    createdAt: '2026-10-01T09:00:00Z',
    updatedAt: '2026-10-01T09:00:00Z'
  }
}

export function route(from: string, to: string, outcome: string): StepLink {
  return {
    id: `${from}-${outcome}` as StepLinkId,
    fromStepId: from as TaskStepId,
    toStepId: to as TaskStepId,
    outcome,
    position: 0
  }
}

export function visit(step: string, ordinal: number, outcome: string | null, open = false): StepVisit {
  return {
    id: `${step}-${ordinal}` as StepVisitId,
    stepId: step as TaskStepId,
    ordinal,
    enteredAt: `2026-10-01T09:0${ordinal}:00Z`,
    leftAt: open ? null : `2026-10-01T09:0${ordinal}:30Z`,
    outcome
  }
}

/** Five moves in: build, deploy, test failed, fix, and back on deploy. */
export function releaseFlow(): TaskStep[] {
  return [
    flowStep('build', 0, 'CLAIMED', [route('build', 'deploy', 'built')], [visit('build', 1, 'built')]),
    flowStep(
      'deploy',
      1,
      'RUNNING',
      [route('deploy', 'test', 'deployed')],
      [visit('deploy', 2, 'deployed'), visit('deploy', 5, null, true)]
    ),
    flowStep(
      'test',
      2,
      'CLAIMED',
      [route('test', 'fix', 'app-error'), route('test', 'ship', 'passed')],
      [visit('test', 3, 'app-error')]
    ),
    flowStep('fix', 3, 'CLAIMED', [route('fix', 'deploy', 'fixed')], [visit('fix', 4, 'fixed')]),
    flowStep('ship', 4)
  ]
}

export function releaseTask(): Task {
  return {
    id: TASK,
    label: 'release',
    title: 'Release',
    status: 'IN_PROGRESS',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'acme',
    projectRepoFolder: '/work/vega',
    documentCount: 0,
    stepCount: 5,
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
    anchor: 'project:vega task:release',
    updatedAt: '2026-10-01T10:00:00Z'
  }
}
