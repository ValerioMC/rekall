import type { Project, Task } from '@/features/catalog/model/catalog'
import { TASK_STATUS_LABEL } from '@/features/catalog/model/task-status'
import type { RekallDocument } from '@/features/notes/model/document'
import { NAV_GROUPS, screensOf } from '@/model/nav-tab'
import type { PaletteEntry, PaletteSection, PaletteTarget } from '@/features/search/model/palette'
import { SEARCH_HIT_LABEL } from '@/features/search/model/search'
import type { SearchHit } from '@/features/search/model/search'
import type { TaskId } from '@/model/branded'

const PROJECT_LIMIT = 5
const TASK_LIMIT = 8
const NOTE_LIMIT = 4

export interface PaletteSources {
  readonly projects: readonly Project[]
  readonly tasks: readonly Task[]
  readonly documents: readonly RekallDocument[]
  readonly needsReview: ReadonlyMap<TaskId, number>
  readonly textHits: readonly SearchHit[]
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

function screenEntries(terms: readonly string[]): PaletteEntry[] {
  return screensOf(NAV_GROUPS)
    .filter((screen) => matches(`${screen.label} ${screen.hint}`, terms))
    .map((screen) => ({
      key: `screen:${screen.to}`,
      group: 'Go to',
      title: screen.label,
      detail: screen.hint,
      mark: { kind: 'screen' },
      target: { kind: 'path', to: screen.to }
    }))
}

function projectEntries(projects: readonly Project[], terms: readonly string[]): PaletteEntry[] {
  return projects
    .filter((project) => matches(`${project.title} ${project.label} ${project.companyName} ${project.taskKey}`, terms))
    .slice(0, PROJECT_LIMIT)
    .map((project) => ({
      key: `project:${project.id}`,
      group: 'Projects',
      title: project.title,
      detail: `${project.companyName} · ${project.anchor}`,
      mark: { kind: 'project', projectId: project.id, icon: project.icon, imageVersion: project.iconImageVersion },
      target: { kind: 'path', to: `/projects/${project.id}` }
    }))
}

function taskDetail(task: Task, needs: number): string {
  return needs > 0 ? `${needs} to review` : TASK_STATUS_LABEL[task.status]
}

function taskEntries(
  tasks: readonly Task[],
  needsReview: ReadonlyMap<TaskId, number>,
  terms: readonly string[]
): PaletteEntry[] {
  return tasks
    .filter((task) => matches(`${task.code} ${task.title} ${task.projectLabel} ${task.projectTitle}`, terms))
    .slice(0, TASK_LIMIT)
    .map((task) => {
      const needs = needsReview.get(task.id) ?? 0
      return {
        key: `task:${task.id}`,
        group: 'Tasks',
        title: task.title,
        detail: `${task.code} · ${taskDetail(task, needs)}`,
        mark: { kind: 'task', needsReview: needs > 0 },
        target: { kind: 'path', to: `/task/${task.id}` }
      }
    })
}

function noteEntries(
  documents: readonly RekallDocument[],
  tasks: readonly Task[],
  terms: readonly string[]
): PaletteEntry[] {
  const codeOf = new Map(tasks.map((task) => [task.id, task.code]))
  return documents
    .filter((document) => matches(`${document.title} ${document.anchor}`, terms))
    .slice(0, NOTE_LIMIT)
    .map((document) => ({
      key: `note:${document.id}`,
      group: 'Notes',
      title: document.title,
      detail: codeOf.get(document.taskId) ?? '',
      mark: { kind: 'note' },
      target: { kind: 'path', to: `/task/${document.taskId}/notes/${document.id}` }
    }))
}

function textEntries(hits: readonly SearchHit[]): PaletteEntry[] {
  return hits.flatMap((hit, index): PaletteEntry[] => {
    const to = hitPath(hit)
    if (to === null) return []
    const target: PaletteTarget = { kind: 'hit', hit, to }
    return [
      {
        key: `hit:${index}:${hit.kind}:${hit.stepId ?? hit.documentId ?? hit.taskId}`,
        group: 'In the text',
        title: hit.title,
        detail: hit.excerpt,
        mark: { kind: 'text', label: SEARCH_HIT_LABEL[hit.kind] },
        target
      }
    ]
  })
}

/** The palette's groups for a query, in the order they are listed; a group with nothing to show is left out. */
export function paletteSections(query: string, sources: PaletteSources): readonly PaletteSection[] {
  const terms = queryTerms(query)
  const sections: PaletteSection[] = [
    { group: 'Go to', entries: screenEntries(terms) },
    { group: 'Projects', entries: projectEntries(sources.projects, terms) },
    { group: 'Tasks', entries: taskEntries(sources.tasks, sources.needsReview, terms) },
    { group: 'Notes', entries: noteEntries(sources.documents, sources.tasks, terms) },
    { group: 'In the text', entries: textEntries(sources.textHits) }
  ]
  return sections.filter((section) => section.entries.length > 0)
}

export function optionId(entry: PaletteEntry): string {
  return `palette-option-${entry.key}`
}

export function flattenSections(sections: readonly PaletteSection[]): readonly PaletteEntry[] {
  return sections.flatMap((section) => section.entries)
}
