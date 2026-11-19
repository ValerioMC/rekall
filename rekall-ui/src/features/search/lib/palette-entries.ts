import type { Project, Task } from '@/features/catalog/model/catalog'
import { TASK_STATUS_LABEL } from '@/features/catalog/model/task-status'
import type { RekallDocument } from '@/features/notes/model/document'
import { NAV_GROUPS, screensOf } from '@/model/nav-tab'
import { PALETTE_GROUPS } from '@/features/search/model/palette'
import type {
  PaletteEntry,
  PaletteGroup,
  PaletteMark,
  PaletteOwner,
  PaletteResults,
  PaletteScope,
  PaletteSection,
  PaletteStatus,
  PaletteTarget
} from '@/features/search/model/palette'
import { SEARCH_HIT_LABEL, isTextQuery } from '@/features/search/model/search'
import type { SearchHit } from '@/features/search/model/search'
import type { ProjectId, TaskId } from '@/model/branded'

const ROWS_PER_GROUP: Readonly<Record<PaletteGroup, number>> = {
  Projects: 3,
  Tasks: 4,
  Notes: 3,
  'Go to': 3,
  'In the text': 4
}
const JUMP_ROWS = 5

export interface PaletteSources {
  readonly projects: readonly Project[]
  readonly tasks: readonly Task[]
  readonly documents: readonly RekallDocument[]
  readonly needsReview: ReadonlyMap<TaskId, number>
  readonly textHits: readonly SearchHit[]
  readonly pinnedProjectIds: readonly ProjectId[]
  readonly recentProjectIds: readonly ProjectId[]
}

/** The words of a query, with the `/rk` prefix and the `project:` or `task:` keys dropped: an anchor is found by its parts. */
export function queryTerms(query: string): readonly string[] {
  return query
    .toLowerCase()
    .replace(/^\s*\/rk\b/, ' ')
    .replace(/\b(company|project|task|note):/g, ' ')
    .replace(/\//g, ' ')
    .split(/\s+/)
    .filter((term) => term.length > 0)
}

function matches(haystack: string, terms: readonly string[]): boolean {
  const lowered = haystack.toLowerCase()
  return terms.every((term) => lowered.includes(term))
}

export function hitPath(hit: SearchHit): string | null {
  if (hit.kind === 'NOTE' && hit.documentId) {
    return hit.taskId ? `/task/${hit.taskId}/notes/${hit.documentId}` : null
  }
  if (!hit.taskId) return null
  if (hit.kind === 'WRAPUP') return `/task/${hit.taskId}/wrapup`
  if (hit.kind === 'DESCRIPTION') return `/task/${hit.taskId}/description`
  return `/task/${hit.taskId}/plan`
}

function projectMark(project: Project): PaletteMark {
  return { kind: 'project', projectId: project.id, icon: project.icon, imageVersion: project.iconImageVersion }
}

function ownerOf(project: Project | undefined): PaletteOwner | null {
  if (!project) return null
  return { projectId: project.id, icon: project.icon, imageVersion: project.iconImageVersion, label: project.label }
}

function taskAnchor(task: Task): string {
  return task.anchor.split(' ').find((part) => part.startsWith('task:')) ?? `task:${task.code}`
}

function taskStatus(task: Task, needs: number): PaletteStatus {
  if (needs > 0) return { label: `${needs} to review`, tone: 'progress' }
  return { label: TASK_STATUS_LABEL[task.status], tone: task.status === 'DONE' ? 'done' : 'neutral' }
}

function screenEntries(terms: readonly string[]): PaletteEntry[] {
  return screensOf(NAV_GROUPS)
    .filter((screen) => matches(`${screen.label} ${screen.hint}`, terms))
    .map((screen) => ({
      key: `screen:${screen.to}`,
      group: 'Go to',
      title: screen.label,
      subtitle: screen.hint,
      anchor: null,
      status: null,
      excerpt: null,
      owner: null,
      mark: { kind: 'screen' },
      target: { kind: 'path', to: screen.to }
    }))
}

function projectEntry(project: Project): PaletteEntry {
  return {
    key: `project:${project.id}`,
    group: 'Projects',
    title: project.title,
    subtitle: project.companyName,
    anchor: project.anchor,
    status: null,
    excerpt: null,
    owner: null,
    mark: projectMark(project),
    target: { kind: 'path', to: `/projects/${project.id}` }
  }
}

function projectEntries(projects: readonly Project[], terms: readonly string[]): PaletteEntry[] {
  return projects
    .filter((project) => matches(`${project.title} ${project.label} ${project.companyName} ${project.taskKey}`, terms))
    .map(projectEntry)
}

function taskEntry(task: Task, project: Project | undefined, needs: number): PaletteEntry {
  return {
    key: `task:${task.id}`,
    group: 'Tasks',
    title: task.title,
    subtitle: task.projectLabel,
    anchor: taskAnchor(task),
    status: taskStatus(task, needs),
    excerpt: null,
    owner: null,
    mark: project ? projectMark(project) : { kind: 'task' },
    target: { kind: 'path', to: `/task/${task.id}` }
  }
}

function taskEntries(
  tasks: readonly Task[],
  projectOf: ReadonlyMap<ProjectId, Project>,
  needsReview: ReadonlyMap<TaskId, number>,
  terms: readonly string[]
): PaletteEntry[] {
  return tasks
    .filter((task) => matches(`${task.code} ${task.title} ${task.projectLabel} ${task.projectTitle}`, terms))
    .map((task) => taskEntry(task, projectOf.get(task.projectId), needsReview.get(task.id) ?? 0))
}

function noteEntries(
  documents: readonly RekallDocument[],
  taskOf: ReadonlyMap<TaskId, Task>,
  projectOf: ReadonlyMap<ProjectId, Project>,
  terms: readonly string[]
): PaletteEntry[] {
  return documents
    .filter((document) => matches(`${document.title} ${document.anchor}`, terms))
    .map((document) => {
      const task = taskOf.get(document.taskId)
      return {
        key: `note:${document.id}`,
        group: 'Notes',
        title: document.title,
        subtitle: task ? `on ${task.title}` : '',
        anchor: document.anchor,
        status: null,
        excerpt: null,
        owner: task ? ownerOf(projectOf.get(task.projectId)) : null,
        mark: { kind: 'note' },
        target: { kind: 'path', to: `/task/${document.taskId}/notes/${document.id}` }
      }
    })
}

function textEntries(
  hits: readonly SearchHit[],
  taskOf: ReadonlyMap<TaskId, Task>,
  projectOf: ReadonlyMap<ProjectId, Project>
): PaletteEntry[] {
  return hits.flatMap((hit, index): PaletteEntry[] => {
    const to = hitPath(hit)
    if (to === null) return []
    const task = hit.taskId ? taskOf.get(hit.taskId) : undefined
    const target: PaletteTarget = { kind: 'hit', hit, to }
    return [
      {
        key: `hit:${index}:${hit.kind}:${hit.stepId ?? hit.documentId ?? hit.taskId}`,
        group: 'In the text',
        title: hit.title,
        subtitle: '',
        anchor: task ? taskAnchor(task) : null,
        status: null,
        excerpt: hit.excerpt,
        owner: task ? ownerOf(projectOf.get(task.projectId)) : null,
        mark: { kind: 'text', label: SEARCH_HIT_LABEL[hit.kind] },
        target
      }
    ]
  })
}

function section(group: PaletteGroup, label: string, found: readonly PaletteEntry[], limit: number): PaletteSection {
  return { group, label, entries: found.slice(0, limit), total: found.length }
}

function jumpSections(
  sources: PaletteSources,
  projectOf: ReadonlyMap<ProjectId, Project>,
  tasks: readonly PaletteEntry[]
): PaletteSection[] {
  const ids = [...new Set([...sources.pinnedProjectIds, ...sources.recentProjectIds])]
  const mine = ids.flatMap((id) => projectOf.get(id) ?? []).filter((project) => project.status !== 'ARCHIVED')
  const waiting = tasks.filter((entry) => entry.status?.label.endsWith('to review'))
  return [
    section('Projects', 'Your projects', mine.map(projectEntry), JUMP_ROWS),
    section('Tasks', 'Waiting on you', waiting, JUMP_ROWS)
  ].filter((candidate) => candidate.total > 0)
}

function countsOf(found: Readonly<Record<PaletteGroup, readonly PaletteEntry[]>>, query: string): PaletteResults['counts'] {
  const counts = Object.fromEntries(PALETTE_GROUPS.map((group) => [group, found[group].length])) as Record<PaletteGroup, number | null>
  if (!isTextQuery(query)) counts['In the text'] = null
  return counts
}

/**
 * What the palette lists for a query and a scope. Nothing typed on `all` opens on the reader's own
 * projects and what waits on them; otherwise every group shows a few rows, and a scope shows its
 * group in full. A group with nothing to show is left out.
 */
export function paletteResults(query: string, sources: PaletteSources, scope: PaletteScope): PaletteResults {
  const terms = queryTerms(query)
  const projectOf = new Map(sources.projects.map((project) => [project.id, project]))
  const taskOf = new Map(sources.tasks.map((task) => [task.id, task]))
  const found: Record<PaletteGroup, readonly PaletteEntry[]> = {
    Projects: projectEntries(sources.projects, terms),
    Tasks: taskEntries(sources.tasks, projectOf, sources.needsReview, terms),
    Notes: noteEntries(sources.documents, taskOf, projectOf, terms),
    'Go to': screenEntries(terms),
    'In the text': textEntries(sources.textHits, taskOf, projectOf)
  }
  const counts = countsOf(found, query)
  if (scope !== 'all') {
    return { sections: [section(scope, scope, found[scope], found[scope].length)].filter((one) => one.total > 0), counts }
  }
  if (terms.length === 0) {
    const jump = jumpSections(sources, projectOf, found.Tasks)
    if (jump.length > 0) return { sections: jump, counts }
  }
  const sections = PALETTE_GROUPS.map((group) => section(group, group, found[group], ROWS_PER_GROUP[group]))
  return { sections: sections.filter((one) => one.total > 0), counts }
}

export function optionId(entry: PaletteEntry): string {
  return `palette-option-${entry.key}`
}

export function flattenSections(sections: readonly PaletteSection[]): readonly PaletteEntry[] {
  return sections.flatMap((one) => one.entries)
}
