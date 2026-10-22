import { describe, expect, it } from 'vitest'
import { credentialNoun, emptyDraft, isComplete, isStarted } from '@/features/connection/lib/credential-draft'

describe('a Jira credential draft', () => {
  it('starts empty, on Jira Cloud, and is not started', () => {
    const draft = emptyDraft('JIRA')
    expect(draft).toEqual({ kind: 'JIRA', deployment: 'CLOUD', email: '', token: '' })
    expect(isStarted(draft)).toBe(false)
  })

  it('is complete on Jira Cloud only with an email and a token', () => {
    expect(isComplete({ kind: 'JIRA', deployment: 'CLOUD', email: 'ada@example.com', token: '' })).toBe(false)
    expect(isComplete({ kind: 'JIRA', deployment: 'CLOUD', email: 'ada', token: 'atl' })).toBe(false)
    expect(isComplete({ kind: 'JIRA', deployment: 'CLOUD', email: ' ada@example.com ', token: 'atl' })).toBe(true)
  })

  it('is complete on Data Center with a personal access token alone', () => {
    expect(isComplete({ kind: 'JIRA', deployment: 'DATA_CENTER', email: '', token: 'NjQ0Mj' })).toBe(true)
    expect(isComplete({ kind: 'JIRA', deployment: 'DATA_CENTER', email: '', token: ' ' })).toBe(false)
  })

  it('counts as started once a field it shows is typed into', () => {
    expect(isStarted({ kind: 'JIRA', deployment: 'CLOUD', email: 'a', token: '' })).toBe(true)
    expect(isStarted({ kind: 'JIRA', deployment: 'CLOUD', email: '', token: 'a' })).toBe(true)
    expect(isStarted({ kind: 'JIRA', deployment: 'DATA_CENTER', email: 'a', token: '' })).toBe(false)
  })

  it('is called a token', () => {
    expect(credentialNoun('JIRA')).toBe('token')
    expect(credentialNoun('GITLAB')).toBe('token')
  })
})
