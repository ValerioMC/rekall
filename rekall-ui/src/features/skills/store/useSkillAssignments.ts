import { setStepSkills, setTagSkills, setTaskSkills } from '@/features/skills/api/skill-assignments.api'
import type { SkillAssignment } from '@/features/skills/model/skill-assignment'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'

/** The skills a task, a step or a tag carries. Each write sends the whole list and keeps what the server answers. */
export function useSkillAssignments(state: ConsoleState) {
  const { taskSkills, tagSkills } = state

  async function assignTaskSkills(taskId: TaskId, skills: readonly SkillAssignment[]): Promise<void> {
    const saved = await setTaskSkills(taskId, skills)
    taskSkills.value = [
      ...taskSkills.value.filter((skill) => skill.taskId !== taskId || skill.stepId !== null),
      ...saved
    ]
  }

  async function assignStepSkills(stepId: TaskStepId, skills: readonly SkillAssignment[]): Promise<void> {
    const saved = await setStepSkills(stepId, skills)
    taskSkills.value = [...taskSkills.value.filter((skill) => skill.stepId !== stepId), ...saved]
  }

  async function assignTagSkills(tagId: TagId, skills: readonly SkillAssignment[]): Promise<void> {
    const saved = await setTagSkills(tagId, skills)
    tagSkills.value = [...tagSkills.value.filter((skill) => skill.tagId !== tagId), ...saved]
  }

  return { assignTaskSkills, assignStepSkills, assignTagSkills }
}
