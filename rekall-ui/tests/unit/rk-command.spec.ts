import { describe, expect, it } from 'vitest'
import { rkCommand, rkWrapupCommand } from '@/common/format/rk-command'

/**
 * What the anchor chips put on the clipboard. Four chips read from one function, because four
 * copies of it is how one of them ends up copying a bare anchor nobody can paste.
 */
describe('rkCommand', () => {
  it('hands back the whole line, ready to paste into a session', () => {
    expect(rkCommand('project:stvv task:env-vars-cv')).toBe('/rk project:stvv task:env-vars-cv')
  })

  it('stays empty when there is no anchor, rather than copying a bare /rk', () => {
    expect(rkCommand('')).toBe('')
  })
})

/**
 * The line typed into a live session to run the wrapup. Built in one place so the "Wrapup here"
 * button and anything else that types this in agree on how a directive gets escaped.
 */
describe('rkWrapupCommand', () => {
  it('has no directive when none was given', () => {
    expect(rkWrapupCommand('project:stvv task:env-vars-cv')).toBe(
      '/rk project:stvv task:env-vars-cv wrapup'
    )
  })

  it('quotes a directive onto the end', () => {
    expect(rkWrapupCommand('project:stvv task:env-vars-cv', 'solo il modulo di export')).toBe(
      '/rk project:stvv task:env-vars-cv wrapup "solo il modulo di export"'
    )
  })

  it('treats blank or whitespace-only directives as none', () => {
    expect(rkWrapupCommand('project:stvv task:env-vars-cv', '   ')).toBe(
      '/rk project:stvv task:env-vars-cv wrapup'
    )
  })

  it('escapes quotes and backslashes so the directive cannot close early', () => {
    expect(rkWrapupCommand('project:stvv task:env-vars-cv', 'say "done" not\\done')).toBe(
      '/rk project:stvv task:env-vars-cv wrapup "say \\"done\\" not\\\\done"'
    )
  })

  it('collapses line breaks so the directive cannot submit the terminal line early', () => {
    expect(rkWrapupCommand('project:stvv task:env-vars-cv', 'first line\nsecond line')).toBe(
      '/rk project:stvv task:env-vars-cv wrapup "first line second line"'
    )
  })

  it('stays empty when there is no anchor, whatever the directive', () => {
    expect(rkWrapupCommand('', 'anything')).toBe('')
  })
})
