import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createProjectCheck,
  createStepCheck,
  deleteCheck,
  fetchChecks,
  fetchTaskCheckResults,
  reorderProjectChecks,
  reorderStepChecks,
  updateCheck
} from '@/features/check/api/checks.api'
import { movedOne, type Direction } from '@/features/project/lib/section-order'
import { replaceById, upsertById } from '@/stores/console/rows'
import type { CheckInput, CheckResult, DoneCheck } from '@/features/check/model/check'
import type { DoneCheckId, ProjectId, TaskId, TaskStepId } from '@/model/branded'

/**
 * The checks of the projects the console has opened, and the results kept for the tasks it has
 * shown. Each list is read when its page opens and again whenever the event stream names it.
 */
export const useChecksStore = defineStore('checks', () => {
  const checksByProject = ref<Readonly<Record<string, readonly DoneCheck[]>>>({})
  const resultsByTask = ref<Readonly<Record<string, readonly CheckResult[]>>>({})

  function scope(projectId: ProjectId, stepId: TaskStepId | null): DoneCheck[] {
    return (checksByProject.value[projectId] ?? [])
      .filter((check) => check.stepId === stepId)
      .sort((first, second) => first.position - second.position)
  }

  function projectChecksOf(projectId: ProjectId): DoneCheck[] {
    return scope(projectId, null)
  }

  function stepChecksOf(projectId: ProjectId, stepId: TaskStepId): DoneCheck[] {
    return scope(projectId, stepId)
  }

  function resultsOf(taskId: TaskId): readonly CheckResult[] {
    return resultsByTask.value[taskId] ?? []
  }

  function putChecks(projectId: ProjectId, checks: readonly DoneCheck[]): void {
    checksByProject.value = { ...checksByProject.value, [projectId]: checks }
  }

  function withChecks(projectId: ProjectId, apply: (known: readonly DoneCheck[]) => DoneCheck[]): void {
    putChecks(projectId, apply(checksByProject.value[projectId] ?? []))
  }

  async function load(projectId: ProjectId): Promise<void> {
    putChecks(projectId, await fetchChecks(projectId))
  }

  async function loadResults(taskId: TaskId): Promise<void> {
    resultsByTask.value = { ...resultsByTask.value, [taskId]: await fetchTaskCheckResults(taskId) }
  }

  async function createForProject(projectId: ProjectId, input: CheckInput): Promise<DoneCheck> {
    const created = await createProjectCheck(projectId, input)
    withChecks(projectId, (known) => upsertById(known, created, 'last'))
    return created
  }

  async function createForStep(projectId: ProjectId, stepId: TaskStepId, input: CheckInput): Promise<DoneCheck> {
    const created = await createStepCheck(stepId, input)
    withChecks(projectId, (known) => upsertById(known, created, 'last'))
    return created
  }

  async function update(id: DoneCheckId, input: CheckInput): Promise<DoneCheck> {
    const saved = await updateCheck(id, input)
    withChecks(saved.projectId, (known) => replaceById(known, saved))
    return saved
  }

  /** Shown at once; the server's answer has the last word, and a refusal puts the old order back. */
  async function move(projectId: ProjectId, stepId: TaskStepId | null, id: DoneCheckId, direction: Direction): Promise<void> {
    const before = scope(projectId, stepId)
    const ids = movedOne(before.map((check) => check.id), id, direction)
    const reordered = ids.flatMap((moved, position) => before.filter((check) => check.id === moved).map((check) => ({ ...check, position })))
    withChecks(projectId, (known) => replacedBy(known, reordered))
    try {
      const saved = stepId ? await reorderStepChecks(stepId, ids) : await reorderProjectChecks(projectId, ids)
      withChecks(projectId, (known) => replacedBy(known, saved))
    } catch (error) {
      withChecks(projectId, (known) => replacedBy(known, before))
      throw error
    }
  }

  async function remove(projectId: ProjectId, id: DoneCheckId): Promise<void> {
    await deleteCheck(id)
    withChecks(projectId, (known) => known.filter((check) => check.id !== id))
  }

  async function applyProjectEvent(projectId: ProjectId): Promise<void> {
    if (projectId in checksByProject.value) await load(projectId)
  }

  async function applyResultEvent(taskId: TaskId): Promise<void> {
    if (taskId in resultsByTask.value) await loadResults(taskId)
  }

  return {
    projectChecksOf,
    stepChecksOf,
    resultsOf,
    load,
    loadResults,
    createForProject,
    createForStep,
    update,
    move,
    remove,
    applyProjectEvent,
    applyResultEvent
  }
})

function replacedBy(known: readonly DoneCheck[], rows: readonly DoneCheck[]): DoneCheck[] {
  return rows.reduce<DoneCheck[]>((all, row) => replaceById(all, row), [...known])
}
