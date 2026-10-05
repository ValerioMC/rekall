import {
  createStep as apiCreateStep,
  deleteStep as apiDeleteStep,
  moveStep as apiMoveStep,
  patchStep as apiPatchStep,
  sendBackStep as apiSendBackStep
} from '@/api/steps.api'
import type { TaskStepPatch } from '@/api/steps.api'
import type { TaskStep } from '@/model/catalog'
import type { TaskId, TaskStepId } from '@/model/branded'
import type { ConsoleState } from './console-state'
import type { useTimers } from './useTimers'
import { trackSave, upsertById } from './rows'

/** A task's checklist. Tasks carry their step counts, so every change recounts the task it is on. */
export function useStepWrites(state: ConsoleState, timers: ReturnType<typeof useTimers>) {
  const { tasks, steps, saveState } = state

  function recountSteps(taskId: TaskId): void {
    const own = steps.value.filter((step) => step.taskId === taskId)
    const checklist = own.filter((step) => step.state !== 'DRAFT')
    tasks.value = tasks.value.map((task) =>
      task.id === taskId
        ? {
            ...task,
            stepCount: checklist.length,
            stepsDone: checklist.filter((step) => step.done).length,
            draftStepCount: own.length - checklist.length
          }
        : task
    )
  }

  function upsertStep(step: TaskStep): void {
    steps.value = upsertById(steps.value, step, 'last')
    recountSteps(step.taskId)
  }

  function applyStepEvent(taskId: TaskId, incoming: TaskStep[]): void {
    steps.value = [...steps.value.filter((step) => step.taskId !== taskId), ...incoming]
    recountSteps(taskId)
  }

  async function addStep(taskId: TaskId, title: string, bodyMarkdown?: string): Promise<TaskStep> {
    const created = await apiCreateStep(taskId, title, bodyMarkdown)
    upsertStep(created)
    await timers.resumeTimerOnWork(taskId)
    return created
  }

  /** Accepting a claim stops the timer on the server; editing the text is working on the task. */
  async function saveStep(id: TaskStepId, patch: TaskStepPatch): Promise<void> {
    const before = steps.value.find((candidate) => candidate.id === id)
    const saved = await trackSave(saveState, async () => {
      const patched = await apiPatchStep(id, patch)
      upsertStep(patched)
      return patched
    })
    if (before?.state === 'CLAIMED' && saved.state === 'DONE')
      await timers.rereadTimerStoppedByAccept(saved.taskId)
    if (patch.title !== undefined || patch.bodyMarkdown !== undefined)
      await timers.resumeTimerOnWork(saved.taskId)
  }

  function toggleStep(id: TaskStepId): Promise<void> {
    const step = steps.value.find((candidate) => candidate.id === id)
    if (!step) return Promise.resolve()
    return saveStep(id, { done: !step.done })
  }

  /** A claim sent back keeps what it worked from as a pass; the detail it returns empty is the feedback. */
  async function sendBackStep(id: TaskStepId): Promise<void> {
    upsertStep(await apiSendBackStep(id))
  }

  async function moveStep(id: TaskStepId, position: number): Promise<void> {
    const step = steps.value.find((candidate) => candidate.id === id)
    if (!step) return
    const reordered = await apiMoveStep(id, position)
    steps.value = [...steps.value.filter((candidate) => candidate.taskId !== step.taskId), ...reordered]
  }

  async function removeStep(id: TaskStepId): Promise<void> {
    const step = steps.value.find((candidate) => candidate.id === id)
    if (!step) return
    await apiDeleteStep(id)
    steps.value = steps.value.filter((candidate) => candidate.id !== id)
    recountSteps(step.taskId)
  }

  return {
    applyStepEvent,
    addStep,
    saveStep,
    toggleStep,
    acceptStep: (id: TaskStepId) => saveStep(id, { done: true }),
    reopenStep: (id: TaskStepId) => saveStep(id, { done: false }),
    sendBackStep,
    promoteStep: (id: TaskStepId) => saveStep(id, { draft: false }),
    returnStepToDraft: (id: TaskStepId) => saveStep(id, { draft: true }),
    moveStep,
    removeStep
  }
}
