import { z } from 'zod'

export const TaskBranchSchema = z.object({
  branch: z.string(),
  commitsAhead: z.number().int(),
  changedFiles: z.array(z.string()),
  mergesCleanly: z.boolean().nullable(),
  worktreeFolder: z.string().nullable(),
  hasUnsavedChanges: z.boolean(),
  mergeCommand: z.string()
})

export const OptionalTaskBranchSchema = TaskBranchSchema.nullable()
