import { apiClient, request } from './client'
import { DirectoryListingSchema } from './schemas/filesystem.schema'
import type { DirectoryListing } from '@/model/filesystem'

/**
 * Lists a folder on this machine. `base` is the folder the picker stands in, `path` what was
 * typed on top of it (`src/`, `~/Downloads`, `/etc`); the server resolves one against the other,
 * and with neither it lists the home folder.
 */
export async function fetchDirectory(base: string | null, path = ''): Promise<DirectoryListing> {
  const query: Record<string, string> = {}
  if (base) query.base = base
  if (path) query.path = path
  return request(async () =>
    DirectoryListingSchema.parse(
      // No retry: a folder that is missing now is missing on the next attempt too, and a typed
      // path changes faster than three attempts would take.
      await apiClient('/api/filesystem/directory', { query, retry: 0 })
    )
  )
}
