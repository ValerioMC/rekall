import { describe, expect, it } from 'vitest'
import { folderName } from '@/features/project/lib/component-folder'

describe('the name a component starts with', () => {
  it('is the last segment of the folder', () => {
    expect(folderName('/Users/you/Projects/rekall/rekall-ui')).toBe('rekall-ui')
  })

  it('ignores a trailing separator and the padding around the path', () => {
    expect(folderName('  /work/billing/  ')).toBe('billing')
  })

  it('reads a Windows path the same way', () => {
    expect(folderName('C:\\work\\billing')).toBe('billing')
  })

  it('is empty while nothing is typed, and the whole path for the root', () => {
    expect(folderName('')).toBe('')
    expect(folderName('/')).toBe('/')
  })
})
