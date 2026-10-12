import type { Project } from '@/model/catalog'

export const RESOURCE_KINDS = ['instructions', 'skill', 'command', 'agent', 'memory', 'memoryIndex'] as const
export type ResourceKind = (typeof RESOURCE_KINDS)[number]

export type SkillSourceKind = 'own' | 'synced' | 'plugin'

/** Who provides a skill: the person, claude.ai's sync, or an installed plugin. */
export interface SkillSource {
  readonly kind: SkillSourceKind
  /** The plugin's name, `claude.ai` for a synced skill, `You` for the person's own. */
  readonly name: string
  readonly detail: string | null
}

/** One of the person's skills and the Rekall-only group it is shown under. */
export interface SkillPlacement {
  readonly skillPath: string
  readonly group: string
}

export interface ResourceSummary {
  /** The identity: the file's path under Claude Code's home folder. */
  readonly path: string
  readonly kind: ResourceKind
  readonly name: string
  readonly description: string | null
  readonly memoryType: string | null
  readonly bytes: number
  readonly modifiedAt: string
  readonly editable: boolean
  /** Where a skill comes from; null for every other kind of file. */
  readonly source: SkillSource | null
}

export interface ResourceFile {
  readonly summary: ResourceSummary
  readonly body: string
}

export interface MemoryGroup {
  readonly id: string
  readonly global: boolean
  readonly workingDirectory: string | null
  /** False when the working directory is a guess from the folder's name. */
  readonly resolved: boolean
  /** False when the folder the memories were written for is gone from the disk. */
  readonly folderExists: boolean
  readonly index: ResourceSummary | null
  readonly memories: readonly ResourceSummary[]
}

export interface ResourceCatalog {
  readonly root: string
  readonly instructions: ResourceSummary | null
  readonly skills: readonly ResourceSummary[]
  readonly commands: readonly ResourceSummary[]
  readonly agents: readonly ResourceSummary[]
  readonly memoryGroups: readonly MemoryGroup[]
}

export const RESOURCE_SECTIONS = ['memories', 'skills', 'commands', 'agents'] as const
export type ResourceSection = (typeof RESOURCE_SECTIONS)[number]

export const SECTION_LABEL: Readonly<Record<ResourceSection, string>> = {
  memories: 'Memories',
  skills: 'Skills',
  commands: 'Commands',
  agents: 'Agents'
}

const KIND_LABEL: Readonly<Record<ResourceKind, string>> = {
  instructions: 'Instructions',
  skill: 'Skill',
  command: 'Command',
  agent: 'Agent',
  memory: 'Memory',
  memoryIndex: 'Memory index'
}

export function resourceKindLabel(kind: ResourceKind): string {
  return KIND_LABEL[kind]
}

/** How many entries a section holds, for its tab. */
export function sectionCount(catalog: ResourceCatalog, section: ResourceSection): number {
  if (section === 'memories') return catalog.memoryGroups.reduce((total, group) => total + group.memories.length, 0)
  return catalog[section].length
}

/** The folder's last segment: what a person calls the project. */
export function groupName(group: MemoryGroup): string {
  if (group.global || !group.workingDirectory) return 'Everywhere'
  const segments = group.workingDirectory.split('/').filter((segment) => segment.length > 0)
  return segments[segments.length - 1] ?? group.workingDirectory
}

/** Everything above the last segment, kept apart so the name can carry the row. */
export function groupParent(group: MemoryGroup): string {
  if (group.global || !group.workingDirectory) return 'Memories that apply to every project'
  const at = group.workingDirectory.lastIndexOf('/')
  return at > 0 ? group.workingDirectory.slice(0, at) : '/'
}

function withoutTrailingSlash(path: string): string {
  return path.length > 1 ? path.replace(/\/+$/, '') : path
}

/** The Rekall project whose folder is the group's working directory, or the one above it. */
export function projectOfGroup(group: MemoryGroup, projects: readonly Project[]): Project | null {
  if (group.global || !group.workingDirectory || !group.resolved) return null
  const directory = withoutTrailingSlash(group.workingDirectory)
  const owners = projects
    .filter((project) => project.repoFolder !== null)
    .map((project) => ({ project, folder: withoutTrailingSlash(project.repoFolder ?? '') }))
    .filter(({ folder }) => directory === folder || directory.startsWith(`${folder}/`))
    .sort((a, b) => b.folder.length - a.folder.length)
  return owners[0]?.project ?? null
}

export type MemoryTone = 'event' | 'state' | 'action' | 'data' | 'none'

const MEMORY_TONE: Readonly<Record<string, MemoryTone>> = {
  feedback: 'event',
  user: 'state',
  project: 'action',
  reference: 'data'
}

export function memoryTone(memoryType: string | null): MemoryTone {
  return (memoryType && MEMORY_TONE[memoryType]) || 'none'
}

/** Every entry that contains the query in its name, description or path; all of them for none. */
export function matchesQuery(summary: ResourceSummary, query: string): boolean {
  const needle = query.trim().toLowerCase()
  if (needle === '') return true
  return [summary.name, summary.description ?? '', summary.path].some((text) => text.toLowerCase().includes(needle))
}

export const NEW_MEMORY_TYPES = ['feedback', 'user', 'project', 'reference'] as const
export type NewMemoryType = (typeof NEW_MEMORY_TYPES)[number]

/** A file name: lowercase words joined by dashes, nothing a path would read as a separator. */
export function slugOf(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/^[-._]+|[-._]+$/g, '')
}

export interface NewResource {
  readonly path: string
  readonly body: string
}

/** Where a new file goes and the text it starts with, or null while its name has nothing usable. */
export function newResource(
  section: ResourceSection,
  name: string,
  options: { readonly groupId: string; readonly memoryType: NewMemoryType }
): NewResource | null {
  const slug = slugOf(name)
  if (slug === '') return null
  switch (section) {
    case 'skills':
      return { path: `skills/${slug}/SKILL.md`, body: `---\nname: ${slug}\ndescription: \n---\n\n# ${slug}\n\n` }
    case 'commands':
      return { path: `commands/${slug}.md`, body: `---\ndescription: \n---\n\n` }
    case 'agents':
      return { path: `agents/${slug}.md`, body: `---\nname: ${slug}\ndescription: \n---\n\n` }
    case 'memories': {
      const folder = options.groupId === 'global' ? 'memory' : `projects/${options.groupId}/memory`
      const front = `---\nname: ${slug}\ndescription: \nmetadata:\n  type: ${options.memoryType}\n---\n\n`
      return { path: `${folder}/${slug}.md`, body: front }
    }
  }
}

/** The section a file is listed under, so opening it from elsewhere shows it in its place. */
export function sectionOfPath(path: string): ResourceSection | null {
  if (path.startsWith('skills/') || path.startsWith('plugins/')) return 'skills'
  if (path.startsWith('commands/')) return 'commands'
  if (path.startsWith('agents/')) return 'agents'
  if (path.startsWith('memory/') || path.startsWith('projects/')) return 'memories'
  return null
}
