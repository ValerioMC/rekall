import { describe, expect, it } from 'vitest'
import { rkCommand } from '@/common/format/rk-command'

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
