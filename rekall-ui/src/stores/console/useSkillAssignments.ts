import { setStepSkills, setTagSkills, setTaskSkills } from '@/api/skill-assignments.api'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'
import type { ConsoleState } from './console-state'

/** The skills a task, a step or a tag carries. Each write sends the whole list and keeps what the server answers. */
export function useSkillAssignments(state: ConsoleState) {
  const { taskSkills, tagSkills } = state

  async function assignTaskSkills(taskId: TaskId, names: readonly string[]): Promise<void> {
    const saved = await setTaskSkills(taskId, names)
    taskSkills.value = [
      ...taskSkills.value.filter((skill) => skill.taskId !== taskId || skill.stepId !== null),
      ...saved
    ]
  }

  async function assignStepSkills(stepId: TaskStepId, names: readonly string[]): Promise<void> {
    const saved = await setStepSkills(stepId, names)
    taskSkills.value = [...taskSkills.value.filter((skill) => skill.stepId !== stepId), ...saved]
  }

  async function assignTagSkills(tagId: TagId, names: readonly string[]): Promise<void> {
    const saved = await setTagSkills(tagId, names)
    tagSkills.value = [...tagSkills.value.filter((skill) => skill.tagId !== tagId), ...saved]
  }

  return { assignTaskSkills, assignStepSkills, assignTagSkills }
}
