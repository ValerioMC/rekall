import type { PlanAdherence, ProjectAdherence } from '@/features/adherence/model/adherence'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

export const TASK = 't-1' as TaskId
export const PROJECT = 'p-1' as ProjectId

export function step(number: number, state: TaskStepState, sentBack = 0): TaskStep {
  return {
    id: `s-${number}` as TaskStepId,
    taskId: TASK,
    title: `Step ${number}`,
    state,
    done: state === 'DONE',
    passes: Array.from({ length: sentBack }, () => ({ detailMarkdown: null, sentBackAt: '2026-11-03T10:00:00Z' }))
  } as unknown as TaskStep
}

export function adherence(changes: Partial<PlanAdherence> = {}): PlanAdherence {
  return {
    stepsTotal: 3,
    stepsDone: 3,
    stepsClaimed: 0,
    acceptedFirstTime: 2,
    sentBack: 1,
    passesTotal: 5,
    firstTimeShare: 2 / 3,
    deviations: 1,
    deviationSteps: [2],
    visits: [],
    ...changes
  }
}

export function projectAdherence(changes: Partial<ProjectAdherence> = {}): ProjectAdherence {
  return {
    stepsTotal: 4,
    stepsDone: 4,
    stepsClaimed: 0,
    acceptedFirstTime: 1,
    sentBack: 3,
    passesTotal: 9,
    firstTimeShare: 0.25,
    deviations: 1,
    mostSentBack: [
      { taskId: 'a' as TaskId, label: 'import', title: 'Import', sentBack: 2, passesTotal: 7 },
      { taskId: 'b' as TaskId, label: 'export', title: 'Export', sentBack: 1, passesTotal: 2 }
    ],
    ...changes
  }
}
