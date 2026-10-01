import { scopeKey, scopeName, type NoteScope } from '@/model/note-scope'
import type { Company, Project, RekallDocument } from '@/model/catalog'

export interface NoteGroup {
  readonly key: string
  readonly scope: NoteScope
  readonly name: string
  readonly notes: RekallDocument[]
}

// Narrowest first: a project's notes are the ones most often reached for while working on it.
const RANK: Readonly<Record<NoteScope['kind'], number>> = { PROJECT: 0, COMPANY: 1, GLOBAL: 2 }

/** Notes bucketed by the scope that owns them, projects then companies then global, each by name. */
export function groupNotesByScope(
  notes: readonly RekallDocument[],
  projects: readonly Pick<Project, 'id' | 'title'>[],
  companies: readonly Pick<Company, 'id' | 'name'>[]
): NoteGroup[] {
  const groups = new Map<string, NoteGroup>()
  for (const note of notes) {
    const key = scopeKey(note.scope)
    let group = groups.get(key)
    if (!group) {
      group = { key, scope: note.scope, name: scopeName(note.scope, projects, companies), notes: [] }
      groups.set(key, group)
    }
    group.notes.push(note)
  }
  return [...groups.values()].sort(
    (a, b) => RANK[a.scope.kind] - RANK[b.scope.kind] || a.name.localeCompare(b.name)
  )
}
