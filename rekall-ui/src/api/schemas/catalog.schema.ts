import { z } from 'zod'
import {
  asDocumentId,
  asEnvironmentId,
  asProjectId,
  asTaskId
} from '@/model/branded'
import { PROJECT_STATUSES, TASK_STATUSES } from '@/model/catalog'

/**
 * Brands are applied by the schema, so an id's kind is decided at the one place the value is
 * checked. A component never has to remember to cast.
 */
const projectId = z.string().uuid().transform(asProjectId)
const taskId = z.string().uuid().transform(asTaskId)
const environmentId = z.string().uuid().transform(asEnvironmentId)
const documentId = z.string().uuid().transform(asDocumentId)

export const ProjectSchema = z.object({
  id: projectId,
  name: z.string(),
  status: z.enum(PROJECT_STATUSES),
  description: z.string().nullable(),
  taskCount: z.number().int(),
  updatedAt: z.string()
})

export const TaskSchema = z.object({
  id: taskId,
  name: z.string(),
  status: z.enum(TASK_STATUSES),
  description: z.string().nullable(),
  projectId,
  projectName: z.string(),
  environmentId: environmentId.nullable(),
  environmentLabel: z.string().nullable(),
  updatedAt: z.string()
})

export const EnvironmentSchema = z.object({
  id: environmentId,
  label: z.string(),
  namespace: z.string().nullable(),
  kubeconfigPath: z.string().nullable(),
  updatedAt: z.string()
})

export const DocumentSchema = z.object({
  id: documentId,
  title: z.string(),
  kind: z.string(),
  bodyMarkdown: z.string(),
  owner: z.string(),
  position: z.number().int(),
  updatedAt: z.string()
})

export const ProjectListSchema = z.array(ProjectSchema)
export const TaskListSchema = z.array(TaskSchema)
export const EnvironmentListSchema = z.array(EnvironmentSchema)
export const DocumentListSchema = z.array(DocumentSchema)
