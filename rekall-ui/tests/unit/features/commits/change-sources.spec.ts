import { describe, expect, it } from 'vitest'
import {
  changeSources,
  otherChanges,
  repositoriesOf,
  reviewFiles,
  scopeGroups,
  scopeOfSource,
  sourcesInScope,
  totalsOf
} from '@/features/commits/lib/change-sources'
import type { ChangedFile, RepositoryChanges, TaskChanges } from '@/features/commits/model/taskChanges'
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

const COMMITS: RepositoryChanges['commits'] = [
  {
    referenceId: 'c1',
    commitHash: 'aaa1111',
    subject: 'first',
    stepId: stepOne,
    loggedAt: '2026-10-01T10:00:00Z',
    available: true,
    truncated: false,
    files: [file('src/b.ts', 'ADDED', 10, 0), file('src/a.ts', 'MODIFIED', 2, 1)]
  },
  {
    referenceId: 'c2',
    commitHash: 'bbb2222',
    subject: 'second',
    stepId: stepTwo,
    loggedAt: '2026-10-01T11:00:00Z',
    available: true,
    truncated: false,
    files: [file('src/a.ts', 'MODIFIED', 3, 2)]
  }
]

const PROJECT: RepositoryChanges = {
  componentId: null,
  name: 'vega',
  commits: COMMITS,
  workingTree: { truncated: false, files: [file('notes.md', 'ADDED', 4, 0)] }
}
const SITE_ID = '6b2f1c3a-0000-4000-8000-000000000001'
const SITE: RepositoryChanges = {
  componentId: SITE_ID,
  name: 'site',
  commits: [
    {
      referenceId: 'c3',
      commitHash: 'ccc3333',
      subject: 'page',
      stepId: stepOne,
      loggedAt: '2026-10-01T11:30:00Z',
      available: true,
      truncated: false,
      files: [file('src/a.ts', 'ADDED', 7, 0)]
    }
  ],
  workingTree: { truncated: false, files: [file('site.txt', 'MODIFIED', 1, 1)] }
}
const CHANGES: TaskChanges = { repositories: [PROJECT] }
const TWO_REPOSITORIES: TaskChanges = { repositories: [PROJECT, SITE] }

describe('changeSources', () => {
  it('lists the commits oldest first and the working tree last', () => {
    expect(changeSources(CHANGES).map((source) => source.id)).toEqual(['c1', 'c2', 'working-tree:project'])
  })

  it('has no working tree source when the tree is clean', () => {
    expect(
      changeSources({ repositories: [{ ...PROJECT, workingTree: null }] }).map((source) => source.kind)
    ).toEqual(['commit', 'commit'])
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
    expect(sourcesInScope(sources, { kind: 'commit', referenceId: 'c1' }).map((source) => source.id)).toEqual(
      ['c1']
    )
    expect(
      sourcesInScope(sources, { kind: 'working-tree', repositoryId: 'project' }).map((source) => source.id)
    ).toEqual(['working-tree:project'])
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
      changeSources({
        repositories: [
          {
            ...PROJECT,
            commits: [
              { ...COMMITS[0]!, files: [file('x.ts', 'MODIFIED', 1, 1)] },
              { ...COMMITS[1]!, files: [file('x.ts', 'DELETED', 0, 4)] }
            ],
            workingTree: null
          }
        ]
      })
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
  const fileOf = (path: string) => reviewFiles(sources).find((entry) => entry.path === path)!

  it('names the sources outside the scope that touched the file, earlier or later than the scope', () => {
    const second = sources.filter((source) => source.id === 'c2')

    expect(
      otherChanges(fileOf('src/a.ts'), sources, second).map((other) => [other.source.id, other.when])
    ).toEqual([['c1', 'earlier']])
    expect(
      otherChanges(
        fileOf('src/a.ts'),
        sources,
        sources.filter((source) => source.id === 'c1')
      ).map((other) => [other.source.id, other.when])
    ).toEqual([['c2', 'later']])
  })

  it('names nothing for a file only the scope touched', () => {
    expect(
      otherChanges(
        fileOf('src/b.ts'),
        sources,
        sources.filter((source) => source.id === 'c1')
      )
    ).toEqual([])
  })

  it('does not name a commit of another repository that changed a file with the same path', () => {
    const both = changeSources(TWO_REPOSITORIES)
    const projectFile = reviewFiles(both).find((entry) => entry.id === 'project::src/a.ts')!

    const others = otherChanges(
      projectFile,
      both,
      both.filter((source) => source.id === 'c1')
    )

    expect(others.map((other) => other.source.id)).toEqual(['c2'])
  })
})

describe('several repositories', () => {
  const sources = changeSources(TWO_REPOSITORIES)

  it('lists every commit oldest first across repositories, then each working tree', () => {
    expect(sources.map((source) => source.id)).toEqual([
      'c1',
      'c2',
      'c3',
      'working-tree:project',
      `working-tree:${SITE_ID}`
    ])
  })

  it('names the repositories in the order the changes listed them', () => {
    expect(repositoriesOf(sources).map((repository) => [repository.id, repository.name])).toEqual([
      ['project', 'vega'],
      [SITE_ID, 'site']
    ])
  })

  it('keeps a file two repositories share apart and sorts the files by repository, then path', () => {
    const files = reviewFiles(sources)

    expect(files.map((entry) => entry.id)).toEqual([
      'project::notes.md',
      'project::src/a.ts',
      'project::src/b.ts',
      `${SITE_ID}::site.txt`,
      `${SITE_ID}::src/a.ts`
    ])
    expect(files.find((entry) => entry.id === `${SITE_ID}::src/a.ts`)?.additions).toBe(7)
  })

  it('reads one repository, or just its working tree, as a scope', () => {
    expect(
      sourcesInScope(sources, { kind: 'repository', repositoryId: SITE_ID }).map((source) => source.id)
    ).toEqual(['c3', `working-tree:${SITE_ID}`])
    expect(
      sourcesInScope(sources, { kind: 'working-tree', repositoryId: SITE_ID }).map((source) => source.id)
    ).toEqual([`working-tree:${SITE_ID}`])
  })

  it('opens a source as its commit, or as the working tree of its own repository', () => {
    expect(scopeOfSource(sources.find((source) => source.id === 'c3')!)).toEqual({
      kind: 'commit',
      referenceId: 'c3'
    })
    expect(scopeOfSource(sources.at(-1)!)).toEqual({ kind: 'working-tree', repositoryId: SITE_ID })
  })
})
