import { describe, expect, it } from 'vitest'
import { flattenSections, hitPath, paletteSections, queryTerms } from '@/features/search/lib/palette-entries'
import type { PaletteSources } from '@/features/search/lib/palette-entries'
import type { RekallDocument } from '@/features/notes/model/document'
import type { SearchHit } from '@/features/search/model/search'
import type { DocumentId, TaskId, TaskStepId } from '@/model/branded'
import { project, task } from '../work/work-fixtures'

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
  tasks: [task('t1', { title: 'Report builder' }), task('t2', { title: 'Retry policy', code: 'PR-2' })],
  documents: [note],
  needsReview: new Map([['t2' as TaskId, 2]]),
  textHits: []
}

function groupsOf(query: string, over: Partial<PaletteSources> = {}): string[] {
  return paletteSections(query, { ...sources, ...over }).map((section) => section.group)
}

describe('queryTerms', () => {
  it('drops a pasted /rk and the anchor keys, so an anchor is found by its parts', () => {
    expect(queryTerms('/rk project:vega task:PR-2')).toEqual(['vega', 'pr-2'])
  })
})

describe('paletteSections', () => {
  it('lists every group a query finds something in, in the menu order', () => {
    expect(groupsOf('')).toEqual(['Go to', 'Projects', 'Tasks', 'Notes'])
  })

  it('narrows each group to what the query names, and leaves out an empty one', () => {
    const sections = paletteSections('retry', sources)

    expect(sections.map((section) => section.group)).toEqual(['Tasks'])
    expect(sections[0]!.entries.map((entry) => entry.title)).toEqual(['Retry policy'])
  })

  it('finds a task by its code and a project by its anchor', () => {
    const entries = flattenSections(paletteSections('/rk project:vega', sources))

    expect(entries.map((entry) => entry.key)).toEqual(['project:vega', 'task:t1', 'task:t2'])
  })

  it('says how many claims a task waits on, else its status', () => {
    const details = flattenSections(paletteSections('', sources))
      .filter((entry) => entry.group === 'Tasks')
      .map((entry) => entry.detail)

    expect(details).toEqual(['t1 · In progress', 'PR-2 · 2 to review'])
  })

  it('names the company beside a project, and opens a task on its page', () => {
    const entries = flattenSections(paletteSections('', sources))

    expect(entries.find((entry) => entry.key === 'project:vega')!.detail).toBe('Acme · project:vega')
    expect(entries.find((entry) => entry.key === 'task:t1')!.target).toEqual({ kind: 'path', to: '/task/t1' })
  })

  it('opens a note on its task, in the notes tab', () => {
    const entry = flattenSections(paletteSections('cluster', sources))[0]!

    expect(entry.target).toEqual({ kind: 'path', to: '/task/t1/notes/d1' })
  })

  it('lists the text the server found last, each hit leading to the tab that shows it', () => {
    const textHits = [
      textHit({ kind: 'STEP', stepId: 's1' as TaskStepId, title: 'Wire the batch' }),
      textHit({ kind: 'NOTE', documentId: 'd1' as DocumentId })
    ]
    const entries = flattenSections(paletteSections('settlement batch', { ...sources, textHits }))

    expect(entries.map((entry) => entry.group)).toEqual(['In the text', 'In the text'])
    expect(entries.map((entry) => entry.target.to)).toEqual(['/task/t1/plan', '/task/t1/notes/d1'])
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
