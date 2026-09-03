import { z } from 'zod'
import {
  asCompanyId,
  asDocumentId,
  asProjectId,
  asTaskId,
  asTaskStepId,
  asTimeEntryId,
  asWrapupId
} from '@/model/branded'
import { PROJECT_STATUSES, TASK_STATUSES } from '@/model/catalog'

/**
 * Brands are applied by the schema, so an id's kind is decided at the one place the value is
 * checked. A component never has to remember to cast.
 */
const companyId = z.string().uuid().transform(asCompanyId)
const projectId = z.string().uuid().transform(asProjectId)
const taskId = z.string().uuid().transform(asTaskId)
const taskStepId = z.string().uuid().transform(asTaskStepId)
const documentId = z.string().uuid().transform(asDocumentId)
const wrapupId = z.string().uuid().transform(asWrapupId)
const timeEntryId = z.string().uuid().transform(asTimeEntryId)

export const CompanySchema = z.object({
  id: companyId,
  name: z.string(),
  description: z.string().nullable(),
  projectCount: z.number().int(),
  taskCount: z.number().int(),
  updatedAt: z.string()
})

export const TaskRefSchema = z.object({
  id: taskId,
  label: z.string(),
  title: z.string(),
  projectLabel: z.string(),
  projectTitle: z.string(),
  companyName: z.string(),
  anchor: z.string()
})

export const ProjectSchema = z.object({
  id: projectId,
  label: z.string(),
  title: z.string(),
  status: z.enum(PROJECT_STATUSES),
  description: z.string().nullable(),
  blueprintMarkdown: z.string().nullable(),
  repoFolder: z.string().nullable(),
  companyId,
  companyName: z.string(),
  taskCount: z.number().int(),
  anchor: z.string(),
  updatedAt: z.string()
})

export const TaskSchema = z.object({
  id: taskId,
  label: z.string(),
  title: z.string(),
  status: z.enum(TASK_STATUSES),
  description: z.string().nullable(),
  projectId,
  projectLabel: z.string(),
  projectTitle: z.string(),
  companyName: z.string(),
  projectRepoFolder: z.string().nullable(),
  documentCount: z.number().int(),
  stepCount: z.number().int(),
  stepsDone: z.number().int(),
  hasWrapup: z.boolean(),
  anchor: z.string(),
  updatedAt: z.string()
})

export const TaskStepSchema = z.object({
  id: taskStepId,
  taskId,
  title: z.string(),
  bodyMarkdown: z.string().nullable(),
  done: z.boolean(),
  doneAt: z.string().nullable(),
  position: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const DocumentSchema = z.object({
  id: documentId,
  title: z.string(),
  kind: z.string(),
  bodyMarkdown: z.string(),
  tasks: z.array(TaskRefSchema),
  updatedAt: z.string()
})

export const WrapupSchema = z.object({
  id: wrapupId,
  taskId,
  taskLabel: z.string(),
  taskTitle: z.string(),
  projectLabel: z.string(),
  anchor: z.string(),
  bodyMarkdown: z.string(),
  writtenBy: z.enum(['CLAUDE', 'HAND']),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const TimeEntrySchema = z.object({
  id: timeEntryId,
  taskId,
  taskLabel: z.string(),
  taskTitle: z.string(),
  projectLabel: z.string(),
  anchor: z.string(),
  startedAt: z.string(),
  stoppedAt: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const CompanyListSchema = z.array(CompanySchema)
export const ProjectListSchema = z.array(ProjectSchema)
export const TaskListSchema = z.array(TaskSchema)
export const TaskStepListSchema = z.array(TaskStepSchema)
export const DocumentListSchema = z.array(DocumentSchema)
export const WrapupListSchema = z.array(WrapupSchema)
export const TimeEntryListSchema = z.array(TimeEntrySchema)
