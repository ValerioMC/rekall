import type { MemoryGroup, ResourceCatalog, ResourceFile, ResourceKind, ResourceSummary } from '@/model/claude-resources'

export function summary(path: string, overrides: Partial<ResourceSummary> = {}): ResourceSummary {
  const kind: ResourceKind = overrides.kind ?? 'memory'
  return {
    path,
    kind,
    name: path.split('/').pop()?.replace('.md', '') ?? path,
    description: null,
    memoryType: null,
    bytes: 10,
    modifiedAt: '2026-10-12T09:00:00.000Z',
    editable: true,
    ...overrides
  }
}

export function group(id: string, workingDirectory: string | null, memories: readonly ResourceSummary[], overrides: Partial<MemoryGroup> = {}): MemoryGroup {
  return {
    id,
    global: workingDirectory === null,
    workingDirectory,
    resolved: true,
    folderExists: true,
    index: null,
    memories,
    ...overrides
  }
}

export function catalog(overrides: Partial<ResourceCatalog> = {}): ResourceCatalog {
  return { root: '/home/me/.claude', instructions: null, skills: [], commands: [], agents: [], memoryGroups: [], ...overrides }
}

export function file(path: string, body: string, overrides: Partial<ResourceSummary> = {}): ResourceFile {
  return { summary: summary(path, overrides), body }
}
