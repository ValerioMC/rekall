import type { Company, Project } from './catalog'
import type { CompanyId, ProjectId } from './branded'

/**
 * Where a note lives. The scope owns it: the note is listed there, can only be put on tasks
 * inside it, and stays there when it is taken off its last task. Sessions still read a note
 * only through the tasks it is on.
 */
export type NoteScope =
  | Readonly<{ kind: 'GLOBAL' }>
  | Readonly<{ kind: 'COMPANY'; id: CompanyId }>
  | Readonly<{ kind: 'PROJECT'; id: ProjectId }>

export type NoteScopeKind = NoteScope['kind']

export const GLOBAL_SCOPE: NoteScope = { kind: 'GLOBAL' }

export const NOTE_SCOPE_KIND_LABEL: Readonly<Record<NoteScopeKind, string>> = {
  PROJECT: 'Project',
  COMPANY: 'Company',
  GLOBAL: 'Global'
}

/** Whether a task in `project` may carry a note of this scope. */
export function scopeAdmits(scope: NoteScope, project: Pick<Project, 'id' | 'companyId'>): boolean {
  if (scope.kind === 'GLOBAL') return true
  if (scope.kind === 'COMPANY') return scope.id === project.companyId
  return scope.id === project.id
}

export function sameScope(a: NoteScope, b: NoteScope): boolean {
  return a.kind === b.kind && (a.kind === 'GLOBAL' || a.id === (b as { id: string }).id)
}

/** A stable key for grouping and `v-for`. */
export function scopeKey(scope: NoteScope): string {
  return scope.kind === 'GLOBAL' ? 'GLOBAL' : `${scope.kind}:${scope.id}`
}

/** What a reader calls the scope: the project's title, the company's name, or "Global". */
export function scopeName(
  scope: NoteScope,
  projects: readonly Pick<Project, 'id' | 'title'>[],
  companies: readonly Pick<Company, 'id' | 'name'>[]
): string {
  if (scope.kind === 'GLOBAL') return 'Global'
  if (scope.kind === 'COMPANY') return companies.find((company) => company.id === scope.id)?.name ?? 'A company'
  return projects.find((project) => project.id === scope.id)?.title ?? 'A project'
}
