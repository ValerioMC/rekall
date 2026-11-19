import { describe, expect, it } from 'vitest'
import { findInHelp, helpSegments, helpText } from '@/features/settings/lib/help-template'

describe('help templates', () => {
  it('uses the typed value, and the default marked replaceable when none is typed', () => {
    expect(helpSegments('/rk task:{task} plan', { project: '', task: '  LC-7 ' })).toEqual([
      { text: '/rk task:', replaceable: false },
      { text: 'LC-7', replaceable: false },
      { text: ' plan', replaceable: false }
    ])
    expect(helpSegments('project:{project}', { project: '', task: '' })).toEqual([
      { text: 'project:', replaceable: false },
      { text: 'vega', replaceable: true }
    ])
  })

  it('renders a template as the text to copy', () => {
    expect(helpText('/rk task:{task} step:3 done', { project: '', task: 'LC-7' })).toBe('/rk task:LC-7 step:3 done')
  })

  it('narrows each context to what mentions the query and drops the rest', () => {
    const found = findInHelp('rekall_note')

    expect(found.map((context) => context.id)).toEqual(['notes', 'outside'])
    expect(found[0]!.commands.map((command) => command.line)).toEqual(['/rk task:{task} note "the export formats we settled on"'])
    expect(found[0]!.directTools).toEqual([])
  })
})
