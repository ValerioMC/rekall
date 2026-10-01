import { describe, expect, it } from 'vitest'
import { groupNotesByScope } from '@/common/catalog/note-groups'
import type { RekallDocument } from '@/model/catalog'
import type { NoteScope } from '@/model/note-scope'
import type { CompanyId, DocumentId, ProjectId } from '@/model/branded'

const vega = 'p1' as ProjectId
const beacon = 'p2' as ProjectId
const acme = 'c1' as CompanyId

const note = (id: string, scope: NoteScope): RekallDocument => ({
  id: id as DocumentId,
  title: `${id}.md`,
  kind: 'notes',
  bodyMarkdown: '',
  tasks: [],
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
      [
        { id: vega, title: 'Vega' },
        { id: beacon, title: 'Beacon' }
      ],
      [{ id: acme, name: 'Acme' }]
    )

    expect(groups.map((group) => [group.name, group.notes.map((n) => n.id)])).toEqual([
      ['Beacon', ['b']],
      ['Vega', ['v1', 'v2']],
      ['Acme', ['a']],
      ['Global', ['g']]
    ])
  })
})
