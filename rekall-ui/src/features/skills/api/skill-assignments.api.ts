import { apiClient, request } from '../../../api/client'
import {
  SkillAssignmentsSchema,
  TagSkillListSchema,
  TaskSkillListSchema
} from './schemas/skill-assignment.schema'
import type { SkillAssignment, TagSkill, TaskSkill } from '@/features/skills/model/skill-assignment'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

export async function fetchSkillAssignments(): Promise<{
  taskSkills: TaskSkill[]
  tagSkills: TagSkill[]
}> {
  return request(async () => SkillAssignmentsSchema.parse(await apiClient('/api/skill-assignments')))
}

function replace(path: string, skills: readonly SkillAssignment[]): Promise<unknown> {
  const body = { skills: skills.map((skill) => ({ name: skill.name, arguments: skill.arguments })) }
  return apiClient(path, { method: 'PUT', body })
}

export async function setTaskSkills(
  taskId: TaskId,
  skills: readonly SkillAssignment[]
): Promise<TaskSkill[]> {
  return request(async () => TaskSkillListSchema.parse(await replace(`/api/tasks/${taskId}/skills`, skills)))
}

export async function setStepSkills(
  stepId: TaskStepId,
  skills: readonly SkillAssignment[]
): Promise<TaskSkill[]> {
  return request(async () => TaskSkillListSchema.parse(await replace(`/api/steps/${stepId}/skills`, skills)))
}

export async function setTagSkills(tagId: TagId, skills: readonly SkillAssignment[]): Promise<TagSkill[]> {
  return request(async () => TagSkillListSchema.parse(await replace(`/api/tags/${tagId}/skills`, skills)))
}
