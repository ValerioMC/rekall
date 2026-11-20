import { describe, expect, it } from 'vitest'
import { ancestorsOf, cursorOf, leafOf, pathOf, splitOnMatch } from '@/features/connection/lib/gitlab-path'

describe('gitlab path', () => {
  it('reads a pasted address as its path', () => {
    expect(pathOf('https://gitlab.acme.dev/platform/backend/payments-api.git')).toBe('platform/backend/payments-api')
    expect(pathOf(' /platform/backend/ ')).toBe('platform/backend')
    expect(pathOf('https://gitlab.acme.dev')).toBe('')
  })

  it('finds the group a path names and the name begun inside it', () => {
    expect(cursorOf('platform/backend/pay')).toEqual({ parent: 'platform/backend', fragment: 'pay' })
    expect(cursorOf('platform/backend/')).toEqual({ parent: 'platform/backend', fragment: '' })
    expect(cursorOf('pay')).toEqual({ parent: '', fragment: 'pay' })
    expect(cursorOf('')).toEqual({ parent: '', fragment: '' })
  })

  it('lists the groups above a path from the top, each with the path to jump back to', () => {
    expect(ancestorsOf('platform/backend')).toEqual([
      { name: 'platform', path: 'platform' },
      { name: 'backend', path: 'platform/backend' }
    ])
    expect(ancestorsOf('')).toEqual([])
  })

  it('splits a name around what was typed, in any case', () => {
    expect(splitOnMatch('payments-api', 'MENT')).toEqual(['pay', 'ment', 's-api'])
    expect(splitOnMatch('payments-api', 'zzz')).toEqual(['payments-api', '', ''])
    expect(leafOf('platform/backend/payments-api')).toBe('payments-api')
  })
})
