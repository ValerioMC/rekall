import { describe, expect, it } from 'vitest'
import { groupNotesByScope } from '@/features/notes/lib/note-groups'
import type { RekallDocument, TaskRef } from '@/features/catalog/model/catalog'
import type { NoteScope } from '@/features/notes/model/note-scope'
import type { CompanyId, DocumentId, ProjectId, TaskId } from '@/model/branded'

const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId
const acme = 'c1' as CompanyId

const project = (id: ProjectId, title: string) => ({
  id,
  title,
  icon: 'rocket',
  iconImageVersion: null,
  companyName: 'Acme'
})

const note = (id: string, scope: NoteScope, onTasks = 0): RekallDocument => ({
  id: id as DocumentId,
  title: `${id}.md`,
  kind: 'notes',
  bodyMarkdown: '',
  tasks: Array.from({ length: onTasks }, (_, index): TaskRef => ({
    id: `t${index}` as TaskId,
    label: `task-${index}`,
    title: `Task ${index}`,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'Acme',
    anchor: `project:vega task:task-${index}`
  })),
  contextMode: 'FULL',
  scope,
  anchor: `note:${id}`,
  updatedAt: '2026-09-01T10:00:00Z'
})

describe('notes grouped by the scope that owns them', () => {
  it('puts projects first, then companies, then global, each by name', () => {
    const groups = groupNotesByScope(
      [
        note('g', { kind: 'GLOBAL' }),
        note('v1', { kind: 'PROJECT', id: vega }),
        note('a', { kind: 'COMPANY', id: acme }),
        note('b', { kind: 'PROJECT', id: beacon }),
        note('v2', { kind: 'PROJECT', id: vega })
      ],
      [project(vega, 'Vega'), project(beacon, 'Beacon')],
      [{ id: acme, name: 'Acme' }]
    )

    expect(groups.map((group) => [group.name, group.notes.map((n) => n.id)])).toEqual([
      ['Beacon', ['b']],
      ['Vega', ['v1', 'v2']],
      ['Acme', ['a']],
      ['Global', ['g']]
    ])
  })

  it('shelves the notes on no task apart from the ones on a task', () => {
    const [group] = groupNotesByScope(
      [note('loose', { kind: 'PROJECT', id: vega }), note('placed', { kind: 'PROJECT', id: vega }, 2)],
      [project(vega, 'Vega')],
      []
    )

    expect(group!.placed.map((n) => n.id)).toEqual(['placed'])
    expect(group!.loose.map((n) => n.id)).toEqual(['loose'])
    expect(group!.notes).toHaveLength(2)
  })

  it('captions a project with its company and the wider scopes with what they are', () => {
    const groups = groupNotesByScope(
      [note('v', { kind: 'PROJECT', id: vega }), note('a', { kind: 'COMPANY', id: acme }), note('g', { kind: 'GLOBAL' })],
      [project(vega, 'Vega')],
      [{ id: acme, name: 'Acme' }]
    )

    expect(groups.map((group) => group.caption)).toEqual(['Acme', 'Company notes', 'Every project'])
    expect(groups[0]!.icon).toBe('rocket')
  })
})
