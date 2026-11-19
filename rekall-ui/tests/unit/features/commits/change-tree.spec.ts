import { describe, expect, it } from 'vitest'
import { buildChangeTree, folderIdsAbove, visibleRows, type TreeNode } from '@/features/commits/lib/change-tree'
import type { RepositoryRef, ReviewFile } from '@/features/commits/lib/change-sources'

const repository: RepositoryRef = { id: 'project', name: 'vega', position: 0 }

function file(path: string): ReviewFile {
  return { id: `project::${path}`, repository, path, oldPath: null, kind: 'MODIFIED', binary: false, additions: 1, deletions: 0, hunks: [], sources: [] }
}

function shape(nodes: readonly TreeNode[]): unknown[] {
  return nodes.map((node) => (node.kind === 'file' ? node.name : { [node.name]: shape(node.children) }))
}

describe('buildChangeTree', () => {
  it('joins a chain of folders that hold nothing else into one element', () => {
    const tree = buildChangeTree('project', [file('rekall-core/src/commit/model/a.rs'), file('rekall-core/src/commit/model/b.rs')])

    expect(shape(tree)).toEqual([{ 'rekall-core/src/commit/model': ['a.rs', 'b.rs'] }])
  })

  it('stops joining where a folder branches or holds a file of its own', () => {
    const tree = buildChangeTree('project', [
      file('core/src/commit/model/a.rs'),
      file('core/src/commit/view/b.rs'),
      file('core/src/lib.rs')
    ])

    expect(shape(tree)).toEqual([{ 'core/src': [{ commit: [{ model: ['a.rs'] }, { view: ['b.rs'] }] }, 'lib.rs'] }])
  })

  it('lists folders before files, each by name, and leaves root files at the top', () => {
    const tree = buildChangeTree('project', [file('z.md'), file('b/x.ts'), file('a/y.ts'), file('Makefile')])

    expect(shape(tree)).toEqual([{ a: ['y.ts'] }, { b: ['x.ts'] }, 'Makefile', 'z.md'])
  })

  it('gives a folder every file below it, and an id that holds its repository and path', () => {
    const folder = buildChangeTree('project', [file('a/b/x.ts'), file('a/b/c/y.ts')])[0]!

    expect(folder.kind === 'folder' && folder.files.map((entry) => entry.path)).toEqual(['a/b/c/y.ts', 'a/b/x.ts'].sort())
    expect(folder.id).toBe('project::a/b/')
  })
})

describe('visibleRows', () => {
  const tree = buildChangeTree('project', [file('a/b/x.ts'), file('a/b/c/y.ts'), file('top.ts')])

  it('lists every row with its depth while nothing is closed', () => {
    expect(visibleRows(tree, new Set()).map((row) => [row.node.name, row.depth])).toEqual([['a/b', 0], ['c', 1], ['y.ts', 2], ['x.ts', 1], ['top.ts', 0]])
  })

  it('leaves out what is inside a closed folder', () => {
    expect(visibleRows(tree, new Set(['project::a/b/'])).map((row) => [row.node.name, row.open])).toEqual([['a/b', false], ['top.ts', false]])
  })
})

describe('folderIdsAbove', () => {
  it('names the folders to open so a file shows, outermost first', () => {
    const tree = buildChangeTree('project', [file('a/b/x.ts'), file('a/b/c/y.ts'), file('a/b/c/z.ts'), file('a/b/c/d/w.ts')])

    expect(folderIdsAbove(tree, 'project::a/b/c/d/w.ts')).toEqual(['project::a/b/', 'project::a/b/c/', 'project::a/b/c/d/'])
    expect(folderIdsAbove(tree, 'project::a/b/x.ts')).toEqual(['project::a/b/'])
  })
})
