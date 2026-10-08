import { scopeKey, scopeName, type NoteScope } from '@/model/note-scope'
import type { Company, Project, RekallDocument } from '@/model/catalog'

type GroupProject = Pick<Project, 'id' | 'title' | 'icon' | 'iconImageVersion' | 'companyName'>

/**
 * The navigator's card for one scope: what owns the notes, and its notes split into the ones on
 * a task and the loose ones that wait in the scope, the way a project card shelves its backlog.
 */
export interface NoteGroup {
  readonly key: string
  readonly scope: NoteScope
  readonly name: string
  /** The line under the name: the project's company, or what kind of scope this is. */
  readonly caption: string
  readonly icon: string
  readonly iconImageVersion: string | null
  readonly notes: readonly RekallDocument[]
  readonly placed: readonly RekallDocument[]
  readonly loose: readonly RekallDocument[]
}

// Narrowest first: a project's notes are the ones most often reached for while working on it.
const RANK: Readonly<Record<NoteScope['kind'], number>> = { PROJECT: 0, COMPANY: 1, GLOBAL: 2 }

function scopeCaption(scope: NoteScope, project: GroupProject | undefined): string {
  if (scope.kind === 'PROJECT') return project?.companyName ?? ''
  return scope.kind === 'COMPANY' ? 'Company notes' : 'Every project'
}

interface Bucket {
  readonly scope: NoteScope
  readonly placed: RekallDocument[]
  readonly loose: RekallDocument[]
}

function toGroup(
  bucket: Bucket,
  projects: readonly GroupProject[],
  companies: readonly Pick<Company, 'id' | 'name'>[]
): NoteGroup {
  const { scope } = bucket
  const project = scope.kind === 'PROJECT' ? projects.find((candidate) => candidate.id === scope.id) : undefined
  return {
    key: scopeKey(scope),
    scope,
    name: scopeName(scope, projects, companies),
    caption: scopeCaption(scope, project),
    icon: project?.icon ?? 'folder',
    iconImageVersion: project?.iconImageVersion ?? null,
    notes: [...bucket.placed, ...bucket.loose],
    placed: bucket.placed,
    loose: bucket.loose
  }
}

/** Notes bucketed by the scope that owns them, projects then companies then global, each by name. */
export function groupNotesByScope(
  notes: readonly RekallDocument[],
  projects: readonly GroupProject[],
  companies: readonly Pick<Company, 'id' | 'name'>[]
): NoteGroup[] {
  const buckets = new Map<string, Bucket>()
  for (const note of notes) {
    const key = scopeKey(note.scope)
    let bucket = buckets.get(key)
    if (!bucket) {
      bucket = { scope: note.scope, placed: [], loose: [] }
      buckets.set(key, bucket)
    }
    if (note.tasks.length) bucket.placed.push(note)
    else bucket.loose.push(note)
  }
  return [...buckets.values()]
    .map((bucket) => toGroup(bucket, projects, companies))
    .sort((a, b) => RANK[a.scope.kind] - RANK[b.scope.kind] || a.name.localeCompare(b.name))
}
