import { describe, expect, it } from 'vitest'
import { deliveryOf, hasDelivery } from '@/features/project/lib/task-delivery'
import { task } from './project-task-fixtures'

describe('deliveryOf', () => {
  it('has nothing for a task with neither issue nor request', () => {
    expect(deliveryOf(task('RK-1'))).toEqual({ issue: null, request: null })
    expect(hasDelivery(task('RK-1'))).toBe(false)
  })

  it('writes a numbered issue with a hash and a Jira key as it is', () => {
    expect(deliveryOf(task('RK-1', { trackerIssueKey: '87' })).issue?.label).toBe('#87')
    expect(deliveryOf(task('RK-1', { trackerIssueKey: 'OPS-4' })).issue?.label).toBe('OPS-4')
  })

  it('names an open pull request and a merged merge request', () => {
    const open = task('RK-1', { pullRequestNumber: 12, pullRequestUrl: 'https://github.com/a/b/pull/12' })
    const merged = task('RK-1', {
      pullRequestNumber: 3,
      pullRequestUrl: 'https://gitlab.com/a/b/-/merge_requests/3',
      pullRequestMergedAt: '2026-09-20T10:00:00Z'
    })

    expect(deliveryOf(open).request).toMatchObject({ label: 'PR #12', state: 'open' })
    expect(deliveryOf(merged).request).toMatchObject({ label: 'MR !3', state: 'merged' })
  })

  it('shows a request that could not be opened as failed, with why', () => {
    const failed = task('RK-1', { pullRequestFailure: 'no remote' })

    expect(deliveryOf(failed).request).toMatchObject({ label: 'PR', state: 'failed', failure: 'no remote' })
    expect(hasDelivery(failed)).toBe(true)
  })
})
