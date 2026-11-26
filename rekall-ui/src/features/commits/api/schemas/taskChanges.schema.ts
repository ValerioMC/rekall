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
  deletions: z.number().int()
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

export const TaskChangesIndexSchema = z.object({
  snapshotId: z.string().uuid(),
  repositories: z.array(RepositoryChangesSchema)
})

export const ChangeFileBatchSchema = z.array(z.object({
  file: z.object({ componentId: z.string().uuid().nullable(), path: z.string().min(1) }),
  source: z.discriminatedUnion('kind', [
    z.object({ kind: z.literal('commit'), referenceId: z.string().uuid() }),
    z.object({ kind: z.literal('working-tree') })
  ]),
  hunks: z.array(DiffHunkSchema)
}))
