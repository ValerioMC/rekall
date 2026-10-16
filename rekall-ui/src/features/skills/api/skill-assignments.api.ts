import { apiClient, request } from '../../../api/client'
import {
  SkillAssignmentsSchema,
  TagSkillListSchema,
  TaskSkillListSchema
} from './schemas/skill-assignment.schema'
import type { TagSkill, TaskSkill } from '@/features/skills/model/skill-assignment'
import type { TagId, TaskId, TaskStepId } from '@/model/branded'

export async function fetchSkillAssignments(): Promise<{
  taskSkills: TaskSkill[]
  tagSkills: TagSkill[]
}> {
  return request(async () => SkillAssignmentsSchema.parse(await apiClient('/api/skill-assignments')))
}

function replace(path: string, skills: readonly string[]): Promise<unknown> {
  return apiClient(path, { method: 'PUT', body: { skills } })
}

export async function setTaskSkills(taskId: TaskId, skills: readonly string[]): Promise<TaskSkill[]> {
  return request(async () => TaskSkillListSchema.parse(await replace(`/api/tasks/${taskId}/skills`, skills)))
}

export async function setStepSkills(stepId: TaskStepId, skills: readonly string[]): Promise<TaskSkill[]> {
  return request(async () => TaskSkillListSchema.parse(await replace(`/api/steps/${stepId}/skills`, skills)))
}

export async function setTagSkills(tagId: TagId, skills: readonly string[]): Promise<TagSkill[]> {
  return request(async () => TagSkillListSchema.parse(await replace(`/api/tags/${tagId}/skills`, skills)))
}
