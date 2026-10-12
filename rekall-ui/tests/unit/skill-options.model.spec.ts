import { describe, expect, it } from 'vitest'
import { filterSkillOptions, groupSkillOptions, skillOptions } from '@/model/skill-options'
import { summary } from './claude-resources.fixtures'

const own = (name: string, description: string | null = null) =>
  summary(`skills/${name}/SKILL.md`, { kind: 'skill', name, description, source: { kind: 'own', name: 'You', detail: null } })
const plugin = (pluginName: string, name: string) =>
  summary(`plugins/${pluginName}/skills/${name}/SKILL.md`, {
    kind: 'skill',
    name,
    editable: false,
    source: { kind: 'plugin', name: pluginName, detail: null }
  })

describe('skillOptions', () => {
  it('puts the person own skills first and qualifies a plugin skill by its plugin', () => {
    const options = skillOptions([plugin('zeta', 'docx'), own('rk'), plugin('alpha', 'pdf')])

    expect(options.map((option) => option.name)).toEqual(['rk', 'alpha:pdf', 'zeta:docx'])
  })

  it('keeps one option per invocation name', () => {
    expect(skillOptions([own('rk'), own('rk')])).toHaveLength(1)
  })
})

describe('filterSkillOptions', () => {
  const options = skillOptions([own('rk', 'Load a Rekall context'), plugin('anthropic-skills', 'docx')])

  it('keeps an option when every word appears in its name, provider or description', () => {
    expect(filterSkillOptions(options, 'rekall context').map((option) => option.name)).toEqual(['rk'])
    expect(filterSkillOptions(options, 'anthropic docx').map((option) => option.name)).toEqual(['anthropic-skills:docx'])
  })

  it('keeps everything for a blank query and nothing for a word that matches nothing', () => {
    expect(filterSkillOptions(options, '  ')).toHaveLength(2)
    expect(filterSkillOptions(options, 'nothing like this')).toEqual([])
  })
})

describe('groupSkillOptions', () => {
  it('starts a group at each change of provider', () => {
    const groups = groupSkillOptions(skillOptions([own('a'), own('b'), plugin('p', 'c')]))

    expect(groups.map((group) => [group.provider, group.options.length])).toEqual([
      ['Yours', 2],
      ['p', 1]
    ])
  })
})
