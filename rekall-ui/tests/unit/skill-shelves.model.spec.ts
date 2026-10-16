import { describe, expect, it } from 'vitest'
import { buildSkillShelves, groupLabels } from '@/features/skills/model/skill-shelves'
import type { SkillSource } from '@/features/claude-resources/model/claude-resources'
import { summary } from './claude-resources.fixtures'

const own: SkillSource = { kind: 'own', name: 'You', detail: null }
const skill = (name: string, source: SkillSource | null, editable = true) =>
  summary(`skills/${name}/SKILL.md`, { kind: 'skill', name, source, editable })

const SKILLS = [
  skill('python-standards', own),
  skill('java-standards', own),
  skill('scratch', own),
  skill('pdf', { kind: 'synced', name: 'claude.ai', detail: 'anthropic' }, false),
  skill('docs', { kind: 'synced', name: 'claude.ai', detail: 'anthropic-example' }, false),
  skill('palette', { kind: 'plugin', name: 'ui-ux-pro-max', detail: 'market · v2' }, false),
  skill('plain', null)
]

describe('the skill shelves', () => {
  it('puts the persons own skills first, in the groups they made, with the rest after them', () => {
    const shelves = buildSkillShelves(SKILLS, [
      { skillPath: 'skills/python-standards/SKILL.md', group: 'Languages' },
      { skillPath: 'skills/java-standards/SKILL.md', group: 'Languages' }
    ], '')

    const yours = shelves[0]!
    expect(yours).toMatchObject({ id: 'yours', locked: false, count: 4 })
    expect(yours.buckets.map((bucket) => [bucket.label, bucket.skills.map((s) => s.name)])).toEqual([
      ['Languages', ['python-standards', 'java-standards']],
      [null, ['scratch', 'plain']]
    ])
  })

  it('lists every read-only source as a locked shelf that says who provides it', () => {
    const locked = buildSkillShelves(SKILLS, [], '').filter((shelf) => shelf.locked)

    expect(locked.map((shelf) => [shelf.title, shelf.caption, shelf.count])).toEqual([
      ['claude.ai', 'Anthropic examples', 1],
      ['claude.ai', 'Made by Anthropic', 1],
      ['ui-ux-pro-max', 'market · v2', 1]
    ])
  })

  it('ignores a placement whose skill is gone and a skill with no source that can be edited counts as yours', () => {
    const shelves = buildSkillShelves([skill('plain', null)], [{ skillPath: 'skills/deleted/SKILL.md', group: 'Old' }], '')
    expect(shelves).toHaveLength(1)
    expect(shelves[0]!.buckets).toEqual([expect.objectContaining({ label: null })])
  })

  it('drops the shelves and groups that nothing matches the filter in', () => {
    const shelves = buildSkillShelves(SKILLS, [{ skillPath: 'skills/python-standards/SKILL.md', group: 'Languages' }], 'palette')
    expect(shelves.map((shelf) => shelf.title)).toEqual(['ui-ux-pro-max'])
  })

  it('names the groups in use once each, in order', () => {
    expect(
      groupLabels([
        { skillPath: 'a', group: 'Zed' },
        { skillPath: 'b', group: 'Alpha' },
        { skillPath: 'c', group: 'Zed' }
      ])
    ).toEqual(['Alpha', 'Zed'])
  })
})
