import { z } from 'zod'
import { asTaskStepId } from '@/model/branded'

const DiffLineSchema = z.object({
  kind: z.enum(['CONTEXT', 'ADDED', 'REMOVED']),
  oldNumber: z.number().int().nullable(),
  newNumber: z.number().int().nullable(),
  text: z.string()
})

const DiffHunkSchema = z.object({
  oldStart: z.number().int(),
  oldLines: z.number().int(),
  newStart: z.number().int(),
  newLines: z.number().int(),
  context: z.string(),
  lines: z.array(DiffLineSchema)
})

const ChangedFileSchema = z.object({
  path: z.string(),
  oldPath: z.string().nullable(),
  kind: z.enum(['ADDED', 'MODIFIED', 'DELETED', 'RENAMED']),
  binary: z.boolean(),
  additions: z.number().int(),
  deletions: z.number().int(),
  hunks: z.array(DiffHunkSchema)
})

const CommitChangesSchema = z.object({
  referenceId: z.string().uuid(),
  commitHash: z.string(),
  subject: z.string(),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  loggedAt: z.string(),
  available: z.boolean(),
  truncated: z.boolean(),
  files: z.array(ChangedFileSchema)
})

const RepositoryChangesSchema = z.object({
  componentId: z.string().uuid().nullable(),
  name: z.string(),
  commits: z.array(CommitChangesSchema),
  workingTree: z.object({ truncated: z.boolean(), files: z.array(ChangedFileSchema) }).nullable()
})

export const TaskChangesSchema = z.object({ repositories: z.array(RepositoryChangesSchema) })
