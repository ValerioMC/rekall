import { describe, expect, it } from 'vitest'
import { flattenSections, hitPath, paletteResults, queryTerms } from '@/features/search/lib/palette-entries'
import type { PaletteSources } from '@/features/search/lib/palette-entries'
import type { PaletteScope } from '@/features/search/model/palette'
import type { RekallDocument } from '@/features/notes/model/document'
import type { SearchHit } from '@/features/search/model/search'
import type { DocumentId, ProjectId, TaskId, TaskStepId } from '@/model/branded'
import { project, task } from '../project/project-task-fixtures'

const vega = 'vega' as ProjectId
const orion = 'orion' as ProjectId

const note: RekallDocument = {
  id: 'd1' as DocumentId,
  title: 'cluster.md',
  kind: 'notes',
  bodyMarkdown: '',
  taskId: 't1' as TaskId,
  contextMode: 'FULL',
  anchor: 'note:00000000',
  updatedAt: '2026-09-01T10:00:00Z'
}

const textHit = (overrides: Partial<SearchHit>): SearchHit => ({
  kind: 'DESCRIPTION',
  taskId: 't1' as TaskId,
  stepId: null,
  documentId: null,
  title: 'Settlement',
  where: 'project:vega task:t1',
  excerpt: 'The nightly settlement batch.',
  ...overrides
})

const sources: PaletteSources = {
  projects: [project('vega', 'Vega Platform'), project('orion', 'Orion')],
  tasks: [
    task('t1', { title: 'Report builder', projectId: vega, projectLabel: 'vega' }),
    task('t2', { title: 'Retry policy', code: 'PR-2', projectId: vega, projectLabel: 'vega', anchor: 'project:vega task:PR-2' })
  ],
  documents: [note],
  needsReview: new Map([['t2' as TaskId, 2]]),
  textHits: [],
  pinnedProjectIds: [],
  recentProjectIds: []
}

function groupsOf(query: string, over: Partial<PaletteSources> = {}, scope: PaletteScope = 'all'): string[] {
  return paletteResults(query, { ...sources, ...over }, scope).sections.map((section) => section.group)
}

function entriesOf(query: string, over: Partial<PaletteSources> = {}, scope: PaletteScope = 'all') {
  return flattenSections(paletteResults(query, { ...sources, ...over }, scope).sections)
}

describe('queryTerms', () => {
  it('drops a pasted /rk and the anchor keys, so an anchor is found by its parts', () => {
    expect(queryTerms('/rk project:vega task:PR-2')).toEqual(['vega', 'pr-2'])
  })
})

describe('paletteResults with nothing typed', () => {
  it('opens on the pinned projects, then the recent ones, each once, and the tasks waiting on a review', () => {
    const results = paletteResults('', { ...sources, pinnedProjectIds: [orion], recentProjectIds: [vega, orion] }, 'all')

    expect(results.sections.map((section) => [section.label, section.entries.map((entry) => entry.title)])).toEqual([
      ['Your projects', ['Orion', 'Vega Platform']],
      ['Waiting on you', ['Retry policy']]
    ])
  })

  it('leaves out an archived project among the recent ones', () => {
    const archived = project('old', 'Old', { status: 'ARCHIVED' })

    const titles = entriesOf('', { projects: [...sources.projects, archived], recentProjectIds: ['old' as ProjectId, vega] }).map((e) => e.title)

    expect(titles).toContain('Vega Platform')
    expect(titles).not.toContain('Old')
  })

  it('falls back to every group when there is nothing of the reader to lead with', () => {
    expect(groupsOf('', { needsReview: new Map() })).toEqual(['Projects', 'Tasks', 'Notes', 'Go to'])
  })

  it('lists a whole group when it is the scope, so every project can be browsed', () => {
    expect(entriesOf('', {}, 'Projects').map((entry) => entry.title)).toEqual(['Vega Platform', 'Orion'])
  })
})

describe('paletteResults for a query', () => {
  it('narrows each group to what the query names, in the group order, and leaves out an empty one', () => {
    expect(groupsOf('vega')).toEqual(['Projects', 'Tasks'])
    expect(entriesOf('retry').map((entry) => entry.title)).toEqual(['Retry policy'])
  })

  it('finds a task by its code and a project by its anchor', () => {
    expect(entriesOf('/rk project:vega').map((entry) => entry.key)).toEqual(['project:vega', 'task:t1', 'task:t2'])
    expect(entriesOf('pr-2').map((entry) => entry.key)).toEqual(['task:t2'])
  })

  it('names the company beside a project, with its anchor', () => {
    const entry = entriesOf('vega').find((candidate) => candidate.key === 'project:vega')!

    expect(entry.subtitle).toBe('Acme')
    expect(entry.anchor).toBe('project:vega')
    expect(entry.target).toEqual({ kind: 'path', to: '/projects/vega' })
  })

  it('shows a task with its project, its anchor and how many claims wait on it, else its status', () => {
    const [plain, waiting] = entriesOf('', { needsReview: new Map([['t2' as TaskId, 2]]) }, 'Tasks')

    expect(plain).toMatchObject({ subtitle: 'vega', anchor: 'task:t1', status: { label: 'In progress', tone: 'neutral' } })
    expect(plain!.mark).toMatchObject({ kind: 'project', projectId: 'vega' })
    expect(waiting).toMatchObject({ anchor: 'task:PR-2', status: { label: '2 to review', tone: 'progress' } })
    expect(plain!.target).toEqual({ kind: 'path', to: '/task/t1' })
  })

  it('opens a note on its task, in the notes tab, and says which project it is in', () => {
    const entry = entriesOf('cluster')[0]!

    expect(entry.target).toEqual({ kind: 'path', to: '/task/t1/notes/d1' })
    expect(entry.subtitle).toBe('on Report builder')
    expect(entry.owner).toMatchObject({ projectId: 'vega', label: 'vega' })
  })

  it('lists the text the server found last, each hit leading to the tab that shows it', () => {
    const textHits = [
      textHit({ kind: 'STEP', stepId: 's1' as TaskStepId, title: 'Wire the batch' }),
      textHit({ kind: 'NOTE', documentId: 'd1' as DocumentId })
    ]
    const entries = entriesOf('settlement batch', { textHits })

    expect(entries.map((entry) => entry.group)).toEqual(['In the text', 'In the text'])
    expect(entries.map((entry) => entry.target.to)).toEqual(['/task/t1/plan', '/task/t1/notes/d1'])
    expect(entries[0]).toMatchObject({ excerpt: 'The nightly settlement batch.', anchor: 'task:t1' })
    expect(entries[0]!.owner).toMatchObject({ label: 'vega' })
  })
})

describe('the scopes', () => {
  const manyTasks = Array.from({ length: 7 }, (_, index) => task(`m${index}`, { title: `Batch job ${index}`, projectId: vega }))

  it('shows a few rows of each group on All, and says how many the group found', () => {
    const results = paletteResults('batch', { ...sources, tasks: manyTasks }, 'all')

    expect(results.sections).toHaveLength(1)
    expect(results.sections[0]!.entries).toHaveLength(4)
    expect(results.sections[0]!.total).toBe(7)
  })

  it('shows one group in full when it is the scope', () => {
    const results = paletteResults('batch', { ...sources, tasks: manyTasks }, 'Tasks')

    expect(results.sections[0]!.entries).toHaveLength(7)
  })

  it('counts what each group found, and has no text count before the text is searched', () => {
    const typed = paletteResults('/rk project:vega', sources, 'all').counts
    expect(typed).toEqual({ Projects: 1, Tasks: 2, Notes: 0, 'Go to': 0, 'In the text': null })

    const searched = paletteResults('vega settlement', { ...sources, textHits: [textHit({})] }, 'all').counts
    expect(searched['In the text']).toBe(1)
  })
})

describe('hitPath', () => {
  it('sends each kind of hit to the tab that shows that text', () => {
    expect(hitPath(textHit({ kind: 'WRAPUP' }))).toBe('/task/t1/wrapup')
    expect(hitPath(textHit({ kind: 'DESCRIPTION' }))).toBe('/task/t1/description')
    expect(hitPath(textHit({ kind: 'STEP', stepId: 's1' as TaskStepId }))).toBe('/task/t1/plan')
  })

  it('has no place for a hit with no task', () => {
    expect(hitPath(textHit({ taskId: null }))).toBeNull()
    expect(hitPath(textHit({ kind: 'NOTE', taskId: null, documentId: 'd1' as DocumentId }))).toBeNull()
  })
})
