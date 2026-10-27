import type { StepLink, StepVisit, Task, TaskStep, TaskStepState } from '@/features/catalog/model/catalog'
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
    flowX: null,
    flowY: null,
    maxVisits: null,
    connectionId: null,
    passes: [],
    links,
    visits,
    createdAt: '2026-10-01T09:00:00Z',
    updatedAt: '2026-10-01T09:00:00Z'
  }
}

/** A route to a step, or to the end of the flow when `to` is null. */
export function route(from: string, to: string | null, outcome: string): StepLink {
  return {
    id: `${from}-${outcome}` as StepLinkId,
    fromStepId: from as TaskStepId,
    toStepId: to as TaskStepId | null,
    outcome,
    position: 0
  }
}

export function visit(step: string, ordinal: number, outcome: string | null, open = false, report: string | null = null): StepVisit {
  return {
    id: `${step}-${ordinal}` as StepVisitId,
    stepId: step as TaskStepId,
    ordinal,
    enteredAt: `2026-10-01T09:0${ordinal}:00Z`,
    leftAt: open ? null : `2026-10-01T09:0${ordinal}:30Z`,
    outcome,
    report
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

/** The release flow with its last step routed to the end and the work finished there. */
export function finishedFlow(): TaskStep[] {
  return releaseFlow().map((step) => {
    if (step.id === 'deploy') {
      return {
        ...step,
        state: 'CLAIMED' as const,
        visits: step.visits.map((v) => ({
          ...v,
          leftAt: v.leftAt ?? '2026-10-01T09:06:00Z',
          outcome: v.outcome ?? 'deployed'
        }))
      }
    }
    if (step.id === 'ship') {
      return {
        ...step,
        state: 'CLAIMED' as const,
        links: [route('ship', null, 'shipped')],
        visits: [visit('ship', 6, 'shipped')]
      }
    }
    return step
  })
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
    graphLaidAt: null,
    releaseSkipped: false,
    kind: 'TASK',
    epicId: null,
    anchor: 'project:vega task:release',
    updatedAt: '2026-10-01T10:00:00Z'
  }
}
