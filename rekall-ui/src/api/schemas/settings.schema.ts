import { z } from 'zod'

export const DatabaseEntrySchema = z.object({
  id: z.string(),
  label: z.string(),
  path: z.string(),
  active: z.boolean(),
  reachable: z.boolean(),
  addedAt: z.string(),
  lastUsedAt: z.string()
})

export const DatabaseStatusSchema = z.object({
  status: z.enum(['READY', 'SETUP_NEEDED', 'UNREACHABLE']),
  active: DatabaseEntrySchema.nullable(),
  databases: z.array(DatabaseEntrySchema)
})

export const FolderCheckSchema = z.object({
  resolvedPath: z.string(),
  exists: z.boolean(),
  isDirectory: z.boolean(),
  writable: z.boolean(),
  hasDatabase: z.boolean(),
  usable: z.boolean()
})

export const AddDatabaseResponseSchema = z.object({
  mode: z.enum(['opened', 'created']),
  entry: DatabaseEntrySchema
})
