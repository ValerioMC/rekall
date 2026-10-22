import { describe, expect, it } from 'vitest'
import { credentialNoun, emptyDraft, isComplete, isStarted } from '@/features/connection/lib/credential-draft'

describe('a Jira credential draft', () => {
  it('starts empty and is not started', () => {
    const draft = emptyDraft('JIRA')
    expect(draft).toEqual({ kind: 'JIRA', email: '', token: '' })
    expect(isStarted(draft)).toBe(false)
  })

  it('is complete only with an email and a token', () => {
    expect(isComplete({ kind: 'JIRA', email: 'ada@example.com', token: '' })).toBe(false)
    expect(isComplete({ kind: 'JIRA', email: 'ada', token: 'atl' })).toBe(false)
    expect(isComplete({ kind: 'JIRA', email: ' ada@example.com ', token: 'atl' })).toBe(true)
  })

  it('counts as started once either field is typed into', () => {
    expect(isStarted({ kind: 'JIRA', email: 'a', token: '' })).toBe(true)
    expect(isStarted({ kind: 'JIRA', email: '', token: 'a' })).toBe(true)
  })

  it('is called an API token', () => {
    expect(credentialNoun('JIRA')).toBe('API token')
    expect(credentialNoun('GITLAB')).toBe('token')
  })
})
