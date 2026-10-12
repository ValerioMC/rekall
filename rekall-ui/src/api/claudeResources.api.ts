import { apiClient, request } from './client'
import { ResourceCatalogSchema, ResourceFileSchema } from './schemas/claude-resources.schema'
import type { ResourceCatalog, ResourceFile } from '@/model/claude-resources'

const FILE = '/api/claude/resources/file'

export async function fetchResourceCatalog(): Promise<ResourceCatalog> {
  return request(async () => ResourceCatalogSchema.parse(await apiClient('/api/claude/resources')))
}

export async function fetchResourceFile(path: string): Promise<ResourceFile> {
  return request(async () => ResourceFileSchema.parse(await apiClient(FILE, { query: { path } })))
}

/** `expectedModifiedAt` is the version the editor opened; null creates a file that must not exist yet. */
export async function saveResourceFile(path: string, body: string, expectedModifiedAt: string | null): Promise<ResourceFile> {
  return request(async () =>
    ResourceFileSchema.parse(await apiClient(FILE, { method: 'PUT', body: { path, body, expectedModifiedAt } }))
  )
}

export async function deleteResourceFile(path: string): Promise<void> {
  await request(async () => {
    await apiClient(FILE, { method: 'DELETE', query: { path } })
  })
}
