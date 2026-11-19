import { describe, expect, it } from 'vitest'
import { repositoryNameOf } from '@/features/commits/lib/repository-name'

describe('repositoryNameOf', () => {
  it('names a repository after its folder, whatever trails the path', () => {
    expect(repositoryNameOf('/work/rekall-rust')).toBe('rekall-rust')
    expect(repositoryNameOf('/work/rekall-rust/')).toBe('rekall-rust')
  })

  it('falls back to a plain name when the project has no folder', () => {
    expect(repositoryNameOf(null)).toBe('Project folder')
  })
})
