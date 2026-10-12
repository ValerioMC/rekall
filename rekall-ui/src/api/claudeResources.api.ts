import { apiClient, request } from './client'
import { MemoryAnalysisSchema, ResourceCatalogSchema, ResourceFileSchema, SkillPlacementsSchema } from './schemas/claude-resources.schema'
import type { ResourceCatalog, ResourceFile, SkillPlacement } from '@/model/claude-resources'
import type { MemoryAnalysis } from '@/model/memory-analysis'

const FILE = '/api/claude/resources/file'
const GROUPS = '/api/claude/resources/skill-groups'

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

export async function fetchMemoryAnalysis(): Promise<MemoryAnalysis> {
  return request(async () => MemoryAnalysisSchema.parse(await apiClient('/api/claude/resources/analysis')))
}

export async function fetchSkillPlacements(): Promise<readonly SkillPlacement[]> {
  return request(async () => SkillPlacementsSchema.parse(await apiClient(GROUPS)))
}

/** Puts one of the person's skills under a group in Rekall, or out of its group when none is given. */
export async function placeSkill(skillPath: string, group: string | null): Promise<readonly SkillPlacement[]> {
  return request(async () =>
    SkillPlacementsSchema.parse(await apiClient(`${GROUPS}/placement`, { method: 'PUT', body: { skillPath, group } }))
  )
}

export async function renameSkillGroup(from: string, to: string): Promise<readonly SkillPlacement[]> {
  return request(async () => SkillPlacementsSchema.parse(await apiClient(`${GROUPS}/rename`, { method: 'PUT', body: { from, to } })))
}
