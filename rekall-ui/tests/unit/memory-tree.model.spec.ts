import { describe, expect, it } from 'vitest'
import { buildMemoryTree, memoriesBelow, relativeDirectory, trailTo } from '@/model/memory-tree'
import { group, summary } from './claude-resources.fixtures'

const memory = (project: string, name: string) => summary(`projects/${project}/memory/${name}.md`, { name })

const GROUPS = [
  group('-work-esa', '/work/esa', [memory('-work-esa', 'esa-rule')]),
  group('-work-esa-core', '/work/esa/core', [memory('-work-esa-core', 'core-rule')]),
  group('-work-esa-core-apps-validator', '/work/esa/core/apps/validator', [memory('-work-esa-core-apps-validator', 'native')]),
  group('-work-other', '/work/other', [memory('-work-other', 'other-rule')]),
  group('global', null, [summary('memory/global.md', { name: 'global-rule' })])
]

describe('the memory tree', () => {
  it('nests a project under the nearest folder above it that has memories', () => {
    const tree = buildMemoryTree(GROUPS, [], '')

    expect(tree.map((node) => node.group.id)).toEqual(['global', '-work-esa', '-work-other'])
    const esa = tree[1]!
    expect(esa.children.map((child) => child.group.id)).toEqual(['-work-esa-core'])
    expect(esa.children[0]!.children.map((child) => child.group.id)).toEqual(['-work-esa-core-apps-validator'])
  })

  it('does not take a folder that merely starts with the same letters for its parent', () => {
    const tree = buildMemoryTree(
      [group('a', '/work/vega', [memory('a', 'x')]), group('b', '/work/vega-web', [memory('b', 'y')])],
      [],
      ''
    )
    expect(tree.map((node) => node.group.id)).toEqual(['a', 'b'])
  })

  it('counts the memories of everything nested below a node', () => {
    const esa = buildMemoryTree(GROUPS, [], '')[1]!
    expect(esa.memories).toHaveLength(1)
    expect(memoriesBelow(esa)).toBe(3)
  })

  it('shows only the folder after the one it is nested in', () => {
    const core = buildMemoryTree(GROUPS, [], '')[1]!.children[0]!
    expect(relativeDirectory(core)).toBe('core')
    expect(relativeDirectory(core.children[0]!)).toBe('apps/validator')
    expect(relativeDirectory(buildMemoryTree(GROUPS, [], '')[1]!)).toBeNull()
  })

  it('keeps a parent with no match of its own when something inside it matches', () => {
    const tree = buildMemoryTree(GROUPS, [], 'native')

    expect(tree.map((node) => node.group.id)).toEqual(['-work-esa'])
    expect(tree[0]!.memories).toHaveLength(0)
    expect(memoriesBelow(tree[0]!)).toBe(1)
  })

  it('leaves a folder that is gone or guessed at the top, never inside another', () => {
    const tree = buildMemoryTree(
      [group('a', '/work/esa', [memory('a', 'x')]), group('b', '/work/esa/old', [memory('b', 'y')], { folderExists: false })],
      [],
      ''
    )
    expect(tree.map((node) => node.group.id)).toEqual(['a', 'b'])
  })

  it('names the groups from the open memory up to the top, so every fold on the way opens', () => {
    const tree = buildMemoryTree(GROUPS, [], '')
    expect(trailTo('projects/-work-esa-core-apps-validator/memory/native.md', tree)).toEqual(['-work-esa', '-work-esa-core', '-work-esa-core-apps-validator'])
    expect(trailTo('nowhere.md', tree)).toEqual([])
    expect(trailTo(null, tree)).toEqual([])
  })
})
