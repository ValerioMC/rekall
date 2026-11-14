import { WORKING_TREE_SOURCE_ID, type ChangeScope } from '@/features/commits/model/changeScope'
import type { ChangedFile, DiffHunk, TaskChanges } from '@/features/commits/model/taskChanges'
import type { TaskStepState } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

/** One place changes come from: a logged commit, or the working tree a session is still writing. */
export interface ChangeSource {
  readonly id: string
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
  readonly path: string
  readonly oldPath: string | null
  readonly kind: ChangedFile['kind']
  readonly binary: boolean
  readonly additions: number
  readonly deletions: number
  readonly hunks: readonly AttributedHunk[]
  readonly sources: readonly ChangeSource[]
}

/** The commits oldest first, then the working tree when it holds changes. */
export function changeSources(changes: TaskChanges): readonly ChangeSource[] {
  const commits = changes.commits.map<ChangeSource>((commit) => ({
    id: commit.referenceId,
    kind: 'commit',
    commitHash: commit.commitHash,
    subject: commit.subject,
    stepId: commit.stepId,
    loggedAt: commit.loggedAt,
    available: commit.available,
    truncated: commit.truncated,
    files: commit.files
  }))
  if (changes.workingTree === null) return commits
  const workingTree: ChangeSource = {
    id: WORKING_TREE_SOURCE_ID,
    kind: 'working-tree',
    commitHash: null,
    subject: 'Working tree',
    stepId: null,
    loggedAt: null,
    available: true,
    truncated: changes.workingTree.truncated,
    files: changes.workingTree.files
  }
  return [...commits, workingTree]
}

export function sourcesInScope(sources: readonly ChangeSource[], scope: ChangeScope): readonly ChangeSource[] {
  if (scope.kind === 'all') return sources
  if (scope.kind === 'step') return sources.filter((source) => source.kind === 'commit' && source.stepId === scope.stepId)
  if (scope.kind === 'working-tree') return sources.filter((source) => source.kind === 'working-tree')
  return sources.filter((source) => source.id === scope.referenceId)
}

/** The files of `sources` by path, each with its hunks in source order. */
export function reviewFiles(sources: readonly ChangeSource[]): readonly ReviewFile[] {
  const byPath = new Map<string, { file: ChangedFile; last: ChangedFile; sources: ChangeSource[]; hunks: AttributedHunk[]; additions: number; deletions: number }>()
  for (const source of sources) {
    for (const file of source.files) {
      const entry = byPath.get(file.path) ?? { file, last: file, sources: [], hunks: [], additions: 0, deletions: 0 }
      entry.last = file
      entry.sources.push(source)
      entry.hunks.push(...file.hunks.map((hunk) => ({ hunk, source })))
      entry.additions += file.additions
      entry.deletions += file.deletions
      byPath.set(file.path, entry)
    }
  }
  return [...byPath.values()]
    .map<ReviewFile>(({ file, last, sources: touching, hunks, additions, deletions }) => ({
      path: file.path,
      oldPath: file.oldPath,
      kind: last.kind === 'DELETED' ? 'DELETED' : file.kind,
      binary: file.binary,
      additions,
      deletions,
      hunks,
      sources: touching
    }))
    .sort((a, b) => a.path.localeCompare(b.path))
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

/** The sources outside the scope that also changed `path`, and whether each came before or after the ones in it. */
export function otherChanges(path: string, all: readonly ChangeSource[], inScope: readonly ChangeSource[]): readonly OtherChange[] {
  const firstInScope = all.findIndex((source) => inScope.includes(source))
  return all
    .map((source, index) => ({ source, index }))
    .filter(({ source }) => !inScope.includes(source) && source.files.some((file) => file.path === path))
    .map(({ source, index }) => ({ source, when: index < firstInScope ? ('earlier' as const) : ('later' as const) }))
}
