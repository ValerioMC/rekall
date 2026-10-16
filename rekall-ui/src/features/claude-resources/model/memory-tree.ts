import { groupName, matchesQuery, projectOfGroup, type MemoryGroup, type ResourceSummary } from '@/features/claude-resources/model/claude-resources'
import type { Project } from '@/features/catalog/model/catalog'

/** A project's memories, with the memories of the folders inside it nested below. */
export interface MemoryNode {
  readonly group: MemoryGroup
  readonly project: Project | null
  /** The memories that match the filter. */
  readonly memories: readonly ResourceSummary[]
  readonly children: readonly MemoryNode[]
  /** The folder this one sits in, to show only what comes after it. */
  readonly parentDirectory: string | null
}

interface Entry {
  readonly group: MemoryGroup
  readonly project: Project | null
  readonly memories: readonly ResourceSummary[]
}

function directoryOf(group: MemoryGroup): string | null {
  if (group.global || !group.resolved || !group.folderExists || !group.workingDirectory) return null
  return group.workingDirectory.length > 1 ? group.workingDirectory.replace(/\/+$/, '') : group.workingDirectory
}

/** The folder a group's memories are nested under: the nearest one above it that has memories too. */
function parentOf(entry: Entry, entries: readonly Entry[]): Entry | null {
  const own = directoryOf(entry.group)
  if (own === null) return null
  let nearest: Entry | null = null
  let nearestLength = -1
  for (const candidate of entries) {
    const above = directoryOf(candidate.group)
    if (above !== null && own.startsWith(`${above}/`) && above.length > nearestLength) {
      nearest = candidate
      nearestLength = above.length
    }
  }
  return nearest
}

/** The everywhere group, then groups that are Rekall projects, then the rest by name; gone folders last. */
function rank(node: MemoryNode): number {
  if (node.group.global) return 0
  if (!node.group.folderExists) return 3
  return node.project ? 1 : 2
}

function byName(a: MemoryNode, b: MemoryNode): number {
  return groupName(a.group).localeCompare(groupName(b.group))
}

function nodeOf(entry: Entry, entries: readonly Entry[], parentDirectory: string | null): MemoryNode | null {
  const directory = directoryOf(entry.group)
  const children = entries
    .filter((candidate) => parentOf(candidate, entries) === entry)
    .map((candidate) => nodeOf(candidate, entries, directory))
    .filter((child): child is MemoryNode => child !== null)
    .sort(byName)
  if (entry.memories.length === 0 && children.length === 0) return null
  return { group: entry.group, project: entry.project, memories: entry.memories, children, parentDirectory }
}

/** Groups nested by the folder they sit in, keeping those with a match or a match below them. */
export function buildMemoryTree(groups: readonly MemoryGroup[], projects: readonly Project[], query: string): readonly MemoryNode[] {
  const entries: Entry[] = groups.map((group) => ({
    group,
    project: projectOfGroup(group, projects),
    memories: group.memories.filter((memory) => matchesQuery(memory, query))
  }))
  return entries
    .filter((entry) => parentOf(entry, entries) === null)
    .map((entry) => nodeOf(entry, entries, null))
    .filter((node): node is MemoryNode => node !== null)
    .sort((a, b) => rank(a) - rank(b) || byName(a, b))
}

/** How many memories a node holds with everything nested below it. */
export function memoriesBelow(node: MemoryNode): number {
  return node.children.reduce((total, child) => total + memoriesBelow(child), node.memories.length)
}

/** The ids of the nodes above the one holding `path`, outermost first, with that node last. */
export function trailTo(path: string | null, nodes: readonly MemoryNode[]): readonly string[] {
  if (path === null) return []
  for (const node of nodes) {
    const holds = node.group.index?.path === path || node.memories.some((memory) => memory.path === path)
    if (holds) return [node.group.id]
    const below = trailTo(path, node.children)
    if (below.length > 0) return [node.group.id, ...below]
  }
  return []
}

/** The part of a folder after the one it is nested in. */
export function relativeDirectory(node: MemoryNode): string | null {
  const directory = node.group.workingDirectory
  if (!directory || !node.parentDirectory || !directory.startsWith(`${node.parentDirectory}/`)) return null
  return directory.slice(node.parentDirectory.length + 1)
}
