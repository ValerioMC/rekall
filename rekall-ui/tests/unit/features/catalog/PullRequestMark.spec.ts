import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import PullRequestMark from '@/features/catalog/components/PullRequestMark.vue'

type Held = {
  pullRequestNumber: number | null
  pullRequestUrl: string | null
  pullRequestFailure: string | null
  pullRequestMergedAt: string | null
}

const NONE: Held = { pullRequestNumber: null, pullRequestUrl: null, pullRequestFailure: null, pullRequestMergedAt: null }

const render = (held: Partial<Held>) => mount(PullRequestMark, { props: { task: { ...NONE, ...held } } })

describe('the pull request mark', () => {
  it('shows nothing for a task that holds no pull request and no failure', () => {
    expect(render({}).html()).toBe('<!--v-if-->')
  })

  it('links the pull request that is open', () => {
    const mark = render({ pullRequestNumber: 5, pullRequestUrl: 'https://github.com/acme/app/pull/5' })

    const link = mark.get('[data-testid="pull-request-link"]')
    expect(link.attributes('href')).toBe('https://github.com/acme/app/pull/5')
    expect(link.text()).toContain('PR #5')
    expect(mark.find('[data-testid="pull-request-merged"]').exists()).toBe(false)
  })

  it('names a GitLab request a merge request', () => {
    const link = render({ pullRequestNumber: 7, pullRequestUrl: 'https://gitlab.com/acme/app/-/merge_requests/7' }).get('[data-testid="pull-request-link"]')

    expect(link.text()).toContain('MR !7')
    expect(link.attributes('aria-label')).toContain('merge request')
  })

  it('says Merged beside the link once the request is merged', () => {
    const mark = render({ pullRequestNumber: 5, pullRequestUrl: 'https://github.com/acme/app/pull/5', pullRequestMergedAt: '2026-11-12T10:00:00Z' })

    expect(mark.get('[data-testid="pull-request-merged"]').text()).toBe('Merged')
    expect(mark.find('[data-testid="pull-request-link"]').exists()).toBe(true)
  })

  it('says Not opened with the stored reason when no request was opened', () => {
    const mark = render({ pullRequestFailure: 'The push of rekall/fix was refused: rejected.' })

    expect(mark.get('[data-testid="pull-request-not-opened"]').attributes('title')).toContain('refused')
    expect(mark.find('[data-testid="pull-request-link"]').exists()).toBe(false)
  })

  it('says Not merged with the host reason when an open request was refused', () => {
    const mark = render({ pullRequestNumber: 5, pullRequestUrl: 'https://github.com/acme/app/pull/5', pullRequestFailure: 'The pull request was not merged: checks failing' })

    expect(mark.get('[data-testid="pull-request-not-merged"]').attributes('title')).toContain('checks failing')
    expect(mark.find('[data-testid="pull-request-not-opened"]').exists()).toBe(false)
  })
})
