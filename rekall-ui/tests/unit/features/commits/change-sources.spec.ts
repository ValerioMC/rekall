import { describe, expect, it } from 'vitest'
import { changeSources, otherChanges, reviewFiles, scopeGroups, sourcesInScope, totalsOf } from '@/features/commits/lib/change-sources'
import type { ChangedFile, TaskChanges } from '@/features/commits/model/taskChanges'
import type { TaskStepId } from '@/model/branded'

const stepOne = 's1' as TaskStepId
const stepTwo = 's2' as TaskStepId

function file(path: string, kind: ChangedFile['kind'], additions: number, deletions: number): ChangedFile {
  return {
    path,
    oldPath: null,
    kind,
    binary: false,
    additions,
    deletions,
    hunks: [{ oldStart: 1, oldLines: deletions, newStart: 1, newLines: additions, context: '', lines: [] }]
  }
}

const CHANGES: TaskChanges = {
  commits: [
    { referenceId: 'c1', commitHash: 'aaa1111', subject: 'first', stepId: stepOne, loggedAt: '2026-10-01T10:00:00Z', available: true, truncated: false, files: [file('src/b.ts', 'ADDED', 10, 0), file('src/a.ts', 'MODIFIED', 2, 1)] },
    { referenceId: 'c2', commitHash: 'bbb2222', subject: 'second', stepId: stepTwo, loggedAt: '2026-10-01T11:00:00Z', available: true, truncated: false, files: [file('src/a.ts', 'MODIFIED', 3, 2)] }
  ],
  workingTree: { truncated: false, files: [file('notes.md', 'ADDED', 4, 0)] }
}

describe('changeSources', () => {
  it('lists the commits oldest first and the working tree last', () => {
    expect(changeSources(CHANGES).map((source) => source.id)).toEqual(['c1', 'c2', 'working-tree'])
  })

  it('has no working tree source when the tree is clean', () => {
    expect(changeSources({ ...CHANGES, workingTree: null }).map((source) => source.kind)).toEqual(['commit', 'commit'])
  })
})

describe('sourcesInScope', () => {
  const sources = changeSources(CHANGES)

  it('takes every source for all changes', () => {
    expect(sourcesInScope(sources, { kind: 'all' })).toHaveLength(3)
  })

  it('takes the commits logged against one step, never the working tree', () => {
    expect(sourcesInScope(sources, { kind: 'step', stepId: stepTwo }).map((source) => source.id)).toEqual(['c2'])
  })

  it('takes one commit or the working tree by itself', () => {
    expect(sourcesInScope(sources, { kind: 'commit', referenceId: 'c1' }).map((source) => source.id)).toEqual(['c1'])
    expect(sourcesInScope(sources, { kind: 'working-tree' }).map((source) => source.id)).toEqual(['working-tree'])
  })
})

describe('reviewFiles', () => {
  const files = reviewFiles(changeSources(CHANGES))

  it('merges a file touched twice into one entry holding both hunks in order', () => {
    const merged = files.find((entry) => entry.path === 'src/a.ts')

    expect(merged?.additions).toBe(5)
    expect(merged?.deletions).toBe(3)
    expect(merged?.hunks.map((attributed) => attributed.source.id)).toEqual(['c1', 'c2'])
  })

  it('sorts the files by path', () => {
    expect(files.map((entry) => entry.path)).toEqual(['notes.md', 'src/a.ts', 'src/b.ts'])
  })

  it('reads a file deleted by its last change as deleted', () => {
    const removed = reviewFiles(
      changeSources({ ...CHANGES, commits: [{ ...CHANGES.commits[0]!, files: [file('x.ts', 'MODIFIED', 1, 1)] }, { ...CHANGES.commits[1]!, files: [file('x.ts', 'DELETED', 0, 4)] }], workingTree: null })
    )

    expect(removed[0]?.kind).toBe('DELETED')
  })
})

describe('totalsOf', () => {
  it('counts files, lines and the commits that made them, not the working tree', () => {
    expect(totalsOf(reviewFiles(changeSources(CHANGES)))).toEqual({ files: 3, additions: 19, deletions: 3, commits: 2 })
  })
})

describe('scopeGroups', () => {
  const sources = changeSources(CHANGES)
  const steps = [
    { id: stepOne, number: 1, title: 'One', state: 'DONE' as const },
    { id: stepTwo, number: 2, title: 'Two', state: 'OPEN' as const }
  ]

  it('groups commits under their step in checklist order and leaves out a step with none', () => {
    const groups = scopeGroups(sources, steps)

    expect(groups.map((group) => [group.step?.number, group.sources.map((source) => source.id)])).toEqual([
      [1, ['c1']],
      [2, ['c2']]
    ])
  })

  it('puts a commit logged against no known step in the task group, last, and never the working tree', () => {
    const groups = scopeGroups(sources, [steps[0]!])

    expect(groups.map((group) => group.step?.number ?? null)).toEqual([1, null])
    expect(groups[1]?.sources.map((source) => source.id)).toEqual(['c2'])
  })
})

describe('otherChanges', () => {
  const sources = changeSources(CHANGES)

  it('names the sources outside the scope that touched the file, earlier or later than the scope', () => {
    const second = sources.filter((source) => source.id === 'c2')

    expect(otherChanges('src/a.ts', sources, second).map((other) => [other.source.id, other.when])).toEqual([['c1', 'earlier']])
    expect(otherChanges('src/a.ts', sources, sources.filter((source) => source.id === 'c1')).map((other) => [other.source.id, other.when])).toEqual([['c2', 'later']])
  })

  it('names nothing for a file only the scope touched', () => {
    expect(otherChanges('src/b.ts', sources, sources.filter((source) => source.id === 'c1'))).toEqual([])
  })
})
