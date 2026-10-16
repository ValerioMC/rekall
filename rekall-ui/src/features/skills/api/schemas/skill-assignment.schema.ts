import { z } from 'zod'
import { asTagId, asTaskId, asTaskStepId } from '@/model/branded'

const taskId = z.string().uuid().transform(asTaskId)
const taskStepId = z.string().uuid().transform(asTaskStepId)
const tagId = z.string().uuid().transform(asTagId)

export const TaskSkillSchema = z.object({
  taskId,
  stepId: taskStepId.nullable(),
  skillName: z.string()
})

export const TagSkillSchema = z.object({
  tagId,
  skillName: z.string()
})

export const SkillAssignmentsSchema = z.object({
  taskSkills: z.array(TaskSkillSchema),
  tagSkills: z.array(TagSkillSchema)
})

export const TaskSkillListSchema = z.array(TaskSkillSchema)
export const TagSkillListSchema = z.array(TagSkillSchema)
