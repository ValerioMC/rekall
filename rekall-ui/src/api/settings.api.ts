import { apiClient, request } from './client'
import {
  AddDatabaseResponseSchema,
  DatabaseEntrySchema,
  DatabaseStatusSchema,
  FolderCheckSchema
} from './schemas/settings.schema'
import type { DatabaseEntry, DatabaseStatus, FolderCheck } from '@/model/settings'

export interface AddDatabaseInput {
  path: string
  label?: string
}

export interface AddDatabaseResult {
  mode: 'opened' | 'created'
  entry: DatabaseEntry
}

export async function fetchDatabaseStatus(): Promise<DatabaseStatus> {
  return request(async () => DatabaseStatusSchema.parse(await apiClient('/api/settings/databases')))
}

/** Read-only: never registers or creates anything, however the path resolves. */
export async function checkFolder(path: string): Promise<FolderCheck> {
  return request(async () =>
    FolderCheckSchema.parse(await apiClient('/api/settings/databases/check', { query: { path } }))
  )
}

export async function addDatabase(input: AddDatabaseInput): Promise<AddDatabaseResult> {
  return request(async () =>
    AddDatabaseResponseSchema.parse(
      await apiClient('/api/settings/databases', { method: 'POST', body: input })
    )
  )
}

export async function activateDatabase(id: string): Promise<DatabaseEntry> {
  return request(async () =>
    DatabaseEntrySchema.parse(await apiClient(`/api/settings/databases/${id}/activate`, { method: 'POST' }))
  )
}

export async function renameDatabase(id: string, label: string): Promise<DatabaseEntry> {
  return request(async () =>
    DatabaseEntrySchema.parse(
      await apiClient(`/api/settings/databases/${id}`, { method: 'PATCH', body: { label } })
    )
  )
}

export async function forgetDatabase(id: string): Promise<void> {
  await request(() => apiClient(`/api/settings/databases/${id}`, { method: 'DELETE' }))
}
