import { z } from 'zod'

export const DirectoryEntrySchema = z.object({
  name: z.string(),
  path: z.string(),
  directory: z.boolean(),
  hidden: z.boolean()
})

export const PathSegmentSchema = z.object({
  name: z.string(),
  path: z.string()
})

export const DirectoryListingSchema = z.object({
  path: z.string(),
  parent: z.string().nullable(),
  home: z.string(),
  segments: z.array(PathSegmentSchema),
  entries: z.array(DirectoryEntrySchema),
  readable: z.boolean(),
  truncated: z.boolean()
})
