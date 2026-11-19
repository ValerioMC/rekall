import { PROJECT_REPOSITORY_ID, type ChangeScope } from '@/features/commits/model/changeScope'
import type { ChangedFile, DiffHunk, TaskChanges } from '@/features/commits/model/taskChanges'
import type { TaskStepState } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

/** A repository the task changed, by the place it has among them: the project's own folder first. */
export interface RepositoryRef {
  readonly id: string
  readonly name: string
  readonly position: number
}

/** One place changes come from: a logged commit, or the working tree a session is still writing. */
export interface ChangeSource {
  readonly id: string
  readonly repository: RepositoryRef
  readonly kind: 'commit' | 'working-tree'
  readonly commitHash: string | null
  readonly subject: string
  readonly stepId: TaskStepId | null
  readonly loggedAt: string | null
  readonly available: boolean
  readonly truncated: boolean
  readonly files: readonly ChangedFile[]
}

/** A hunk and the source that made it. */
export interface AttributedHunk {
  readonly hunk: DiffHunk
  readonly source: ChangeSource
}

/** One file as the review shows it: the hunks of every source in scope that touched it, oldest first. */
export interface ReviewFile {
  /** The repository and the path together: two repositories can both have a `README.md`. */
  readonly id: string
  readonly repository: RepositoryRef
  readonly path: string
  readonly oldPath: string | null
  readonly kind: ChangedFile['kind']
  readonly binary: boolean
  readonly additions: number
  readonly deletions: number
  readonly hunks: readonly AttributedHunk[]
  readonly sources: readonly ChangeSource[]
}

/** Every commit of every repository, oldest first, then each repository's working tree when it holds changes. */
export function changeSources(changes: TaskChanges): readonly ChangeSource[] {
  const commits: ChangeSource[] = []
  const workingTrees: ChangeSource[] = []
  changes.repositories.forEach((repository, position) => {
    const ref: RepositoryRef = {
      id: repository.componentId ?? PROJECT_REPOSITORY_ID,
      name: repository.name,
      position
    }
    commits.push(
      ...repository.commits.map<ChangeSource>((commit) => ({
        id: commit.referenceId,
        repository: ref,
        kind: 'commit',
        commitHash: commit.commitHash,
        subject: commit.subject,
        stepId: commit.stepId,
        loggedAt: commit.loggedAt,
        available: commit.available,
        truncated: commit.truncated,
        files: commit.files
      }))
    )
    if (repository.workingTree !== null) {
      workingTrees.push({
        id: `working-tree:${ref.id}`,
        repository: ref,
        kind: 'working-tree',
        commitHash: null,
        subject: 'Working tree',
        stepId: null,
        loggedAt: null,
        available: true,
        truncated: repository.workingTree.truncated,
        files: repository.workingTree.files
      })
    }
  })
  return [
    ...commits.sort((a, b) => Date.parse(a.loggedAt ?? '') - Date.parse(b.loggedAt ?? '')),
    ...workingTrees
  ]
}

/** The repositories the sources come from, in the order the task's changes listed them. */
export function repositoriesOf(sources: readonly ChangeSource[]): readonly RepositoryRef[] {
  const byId = new Map<string, RepositoryRef>()
  for (const source of sources) byId.set(source.repository.id, source.repository)
  return [...byId.values()].sort((a, b) => a.position - b.position)
}

/** The scope that reads just this source: its commit, or its repository's working tree. */
export function scopeOfSource(source: ChangeSource): ChangeScope {
  return source.kind === 'working-tree'
    ? { kind: 'working-tree', repositoryId: source.repository.id }
    : { kind: 'commit', referenceId: source.id }
}

export function sourcesInScope(sources: readonly ChangeSource[], scope: ChangeScope): readonly ChangeSource[] {
  if (scope.kind === 'all') return sources
  if (scope.kind === 'step')
    return sources.filter((source) => source.kind === 'commit' && source.stepId === scope.stepId)
  if (scope.kind === 'repository')
    return sources.filter((source) => source.repository.id === scope.repositoryId)
  if (scope.kind === 'working-tree')
    return sources.filter(
      (source) => source.kind === 'working-tree' && source.repository.id === scope.repositoryId
    )
  return sources.filter((source) => source.id === scope.referenceId)
}

/** The files of `sources` by repository and path, each with its hunks in source order. */
export function reviewFiles(sources: readonly ChangeSource[]): readonly ReviewFile[] {
  const byId = new Map<
    string,
    {
      id: string
      repository: RepositoryRef
      file: ChangedFile
      last: ChangedFile
      sources: ChangeSource[]
      hunks: AttributedHunk[]
      additions: number
      deletions: number
    }
  >()
  for (const source of sources) {
    for (const file of source.files) {
      const id = fileIdOf(source.repository, file.path)
      const entry = byId.get(id) ?? {
        id,
        repository: source.repository,
        file,
        last: file,
        sources: [],
        hunks: [],
        additions: 0,
        deletions: 0
      }
      entry.last = file
      entry.sources.push(source)
      entry.hunks.push(...file.hunks.map((hunk) => ({ hunk, source })))
      entry.additions += file.additions
      entry.deletions += file.deletions
      byId.set(id, entry)
    }
  }
  return [...byId.values()]
    .map<ReviewFile>(({ id, repository, file, last, sources: touching, hunks, additions, deletions }) => ({
      id,
      repository,
      path: file.path,
      oldPath: file.oldPath,
      kind: last.kind === 'DELETED' ? 'DELETED' : file.kind,
      binary: file.binary,
      additions,
      deletions,
      hunks,
      sources: touching
    }))
    .sort((a, b) => a.repository.position - b.repository.position || a.path.localeCompare(b.path))
}

function fileIdOf(repository: RepositoryRef, path: string): string {
  return `${repository.id}::${path}`
}

export interface ChangeTotals {
  readonly files: number
  readonly additions: number
  readonly deletions: number
  readonly commits: number
}

export function totalsOf(files: readonly ReviewFile[]): ChangeTotals {
  const commits = new Set(files.flatMap((file) => file.sources.filter((s) => s.kind === 'commit').map((s) => s.id)))
  return {
    files: files.length,
    additions: files.reduce((sum, file) => sum + file.additions, 0),
    deletions: files.reduce((sum, file) => sum + file.deletions, 0),
    commits: commits.size
  }
}

export interface StepRef {
  readonly id: TaskStepId
  readonly number: number
  readonly title: string
  readonly state: TaskStepState
}

/** The commits of one step in the order they were logged; `step` is null for the commits logged against the task itself. */
export interface ScopeGroup {
  readonly step: StepRef | null
  readonly sources: readonly ChangeSource[]
}

/** The commit sources grouped by the step they were logged against, in checklist order, then the task's own. */
export function scopeGroups(sources: readonly ChangeSource[], steps: readonly StepRef[]): readonly ScopeGroup[] {
  const commits = sources.filter((source) => source.kind === 'commit')
  const stepGroups = steps
    .map<ScopeGroup>((step) => ({ step, sources: commits.filter((source) => source.stepId === step.id) }))
    .filter((group) => group.sources.length > 0)
  const known = new Set(steps.map((step) => step.id))
  const ownCommits = commits.filter((source) => source.stepId === null || !known.has(source.stepId))
  return ownCommits.length > 0 ? [...stepGroups, { step: null, sources: ownCommits }] : stepGroups
}

export interface OtherChange {
  readonly source: ChangeSource
  readonly when: 'earlier' | 'later'
}

/** The sources outside the scope that also changed `file` in its repository, and whether each came before or after the ones in it. */
export function otherChanges(
  file: ReviewFile,
  all: readonly ChangeSource[],
  inScope: readonly ChangeSource[]
): readonly OtherChange[] {
  const firstInScope = all.findIndex((source) => inScope.includes(source))
  return all
    .map((source, index) => ({ source, index }))
    .filter(
      ({ source }) =>
        !inScope.includes(source) &&
        source.repository.id === file.repository.id &&
        source.files.some((changed) => changed.path === file.path)
    )
    .map(({ source, index }) => ({
      source,
      when: index < firstInScope ? ('earlier' as const) : ('later' as const)
    }))
}
