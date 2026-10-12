import { describe, expect, it } from 'vitest'
import { groupName, groupParent, matchesQuery, memoryTone, newResource, projectOfGroup, sectionCount, slugOf } from '@/model/claude-resources'
import { catalog, group, summary } from './claude-resources.fixtures'
import type { Project } from '@/model/catalog'

const project = (id: string, repoFolder: string | null): Project => ({ id, repoFolder, title: id }) as unknown as Project

describe('naming a memory group', () => {
  it('uses the last folder segment and keeps the rest as the parent', () => {
    const found = group('-a', '/Users/me/Projects/ESA/_kmasters', [])
    expect(groupName(found)).toBe('_kmasters')
    expect(groupParent(found)).toBe('/Users/me/Projects/ESA')
  })

  it('calls the global group everywhere', () => {
    expect(groupName(group('global', null, []))).toBe('Everywhere')
  })
})

describe('finding the Rekall project of a group', () => {
  const vega = project('vega', '/work/vega')
  const api = project('api', '/work/vega/api')

  it('matches the project whose folder it is', () => {
    expect(projectOfGroup(group('g', '/work/vega', []), [vega])).toBe(vega)
  })

  it('prefers the deepest folder when a session started below the project', () => {
    expect(projectOfGroup(group('g', '/work/vega/api/src', []), [vega, api])).toBe(api)
  })

  it('does not confuse a sibling that shares a prefix', () => {
    expect(projectOfGroup(group('g', '/work/vegas', []), [vega])).toBeNull()
  })

  it('ignores trailing slashes and projects without a folder', () => {
    expect(projectOfGroup(group('g', '/work/vega/', []), [project('x', null), project('vega', '/work/vega/')])?.id).toBe('vega')
  })

  it('leaves a guessed path and the global group unmatched', () => {
    expect(projectOfGroup(group('g', '/work/vega', [], { resolved: false }), [vega])).toBeNull()
    expect(projectOfGroup(group('global', null, []), [vega])).toBeNull()
  })
})

describe('listing helpers', () => {
  it('counts memories across groups and the flat sections by length', () => {
    const found = catalog({ memoryGroups: [group('a', '/a', [summary('x'), summary('y')]), group('b', '/b', [summary('z')])], skills: [summary('s', { kind: 'skill' })] })
    expect(sectionCount(found, 'memories')).toBe(3)
    expect(sectionCount(found, 'skills')).toBe(1)
    expect(sectionCount(found, 'agents')).toBe(0)
  })

  it('filters by name, description or path, ignoring case', () => {
    const entry = summary('projects/-a/memory/lean.md', { name: 'Lean visual', description: 'few render rounds' })
    expect(matchesQuery(entry, 'LEAN')).toBe(true)
    expect(matchesQuery(entry, 'render')).toBe(true)
    expect(matchesQuery(entry, '-a/memory')).toBe(true)
    expect(matchesQuery(entry, 'missing')).toBe(false)
    expect(matchesQuery(entry, '   ')).toBe(true)
  })

  it('colours the four memory types and leaves others plain', () => {
    expect(memoryTone('feedback')).toBe('event')
    expect(memoryTone('reference')).toBe('data')
    expect(memoryTone('whatever')).toBe('none')
    expect(memoryTone(null)).toBe('none')
  })
})

describe('starting a new file', () => {
  const options = { groupId: 'global', memoryType: 'feedback' } as const

  it('turns a name into a slug', () => {
    expect(slugOf('  My Great Rule!  ')).toBe('my-great-rule')
    expect(slugOf('../escape')).toBe('escape')
    expect(slugOf('???')).toBe('')
  })

  it('has nothing to create from a name with nothing usable in it', () => {
    expect(newResource('skills', '!!!', options)).toBeNull()
  })

  it('places each section where Claude Code looks', () => {
    expect(newResource('skills', 'Docx Tools', options)?.path).toBe('skills/docx-tools/SKILL.md')
    expect(newResource('commands', 'review', options)?.path).toBe('commands/review.md')
    expect(newResource('agents', 'planner', options)?.path).toBe('agents/planner.md')
  })

  it('files a memory under its project, or under the global folder', () => {
    expect(newResource('memories', 'rule', { groupId: '-work-vega', memoryType: 'project' })?.path).toBe('projects/-work-vega/memory/rule.md')
    expect(newResource('memories', 'rule', options)?.path).toBe('memory/rule.md')
  })

  it('starts a memory with the front matter the index line is built from', () => {
    const made = newResource('memories', 'rule', { groupId: 'global', memoryType: 'user' })
    expect(made?.body).toBe('---\nname: rule\ndescription: \nmetadata:\n  type: user\n---\n\n')
  })
})
