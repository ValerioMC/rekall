import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import PassSummary from '@/features/feedback/components/PassSummary.vue'
import { groupByPass } from '@/features/feedback/lib/feedback-passes'
import type { CommitReference } from '@/features/commits/model/commitReference'
import { appliedInPass, running, TASK } from './fixtures'

function commit(hash: string, comment: string, createdAt: string): CommitReference {
  return { id: hash, taskId: TASK, stepId: null, stepTitle: null, commitHash: hash, comment, inContext: false, createdAt } as CommitReference
}

describe('what the last pass did', () => {
  it('lists each answered entry with the first line of its reply', () => {
    const pass = groupByPass([appliedInPass(1, 1, 'Fixed the totals.\nMore detail.', { kind: 'PLAN' })]).passes[0] ?? null

    const wrapper = mount(PassSummary, { props: { pass, commits: [] } })

    expect(wrapper.get('[data-testid="pass-summary-title"]').text()).toBe('What pass 1 did')
    const row = wrapper.get('[data-testid="pass-summary-entry"]')
    expect(row.text()).toContain('#1')
    expect(row.text()).toContain('On the plan')
    expect(row.text()).toContain('Fixed the totals.')
    expect(row.text()).not.toContain('More detail.')
  })

  it('marks an entry the session never answered', () => {
    const pass = groupByPass([appliedInPass(1, 1, 'Done.'), running(2, 1)]).passes[0] ?? null

    const wrapper = mount(PassSummary, { props: { pass, commits: [] } })

    const rows = wrapper.findAll('[data-testid="pass-summary-entry"]')
    expect(rows[0]?.find('[data-testid="pass-summary-no-reply"]').exists()).toBe(false)
    expect(rows[1]?.get('[data-testid="pass-summary-no-reply"]').text()).toBe('No reply')
  })

  it('keeps only the commits logged since the pass was taken', () => {
    const pass = groupByPass([appliedInPass(1, 1, 'Done.', { takenAt: '2026-11-04T10:00:00Z' })]).passes[0] ?? null
    const commits = [commit('aaaaaaa111', 'older', '2026-11-04T09:59:00Z'), commit('bbbbbbb222', 'during the pass', '2026-11-04T10:05:00Z')]

    const wrapper = mount(PassSummary, { props: { pass, commits } })

    expect(wrapper.findAll('[data-testid="pass-summary-commit"]').map((row) => row.text())).toEqual(['bbbbbbbduring the pass'])
  })

  it('shows all the commits and no entries when no pass ever ran', () => {
    const commits = [commit('aaaaaaa111', 'first', '2026-11-04T09:00:00Z'), commit('bbbbbbb222', 'second', '2026-11-04T10:00:00Z')]

    const wrapper = mount(PassSummary, { props: { pass: null, commits } })

    expect(wrapper.get('[data-testid="pass-summary-title"]').text()).toBe('What the session did')
    expect(wrapper.find('[data-testid="pass-summary-entries"]').exists()).toBe(false)
    expect(wrapper.findAll('[data-testid="pass-summary-commit"]')).toHaveLength(2)
  })

  it('caps the commits shown and counts the rest, and says when there are none', () => {
    const many = Array.from({ length: 7 }, (_, index) => commit(`ccccccc${index}`, `commit ${index}`, '2026-11-04T10:00:00Z'))

    const capped = mount(PassSummary, { props: { pass: null, commits: many } })
    expect(capped.findAll('[data-testid="pass-summary-commit"]')).toHaveLength(5)
    expect(capped.get('[data-testid="pass-summary-more"]').text()).toBe('and 2 more')

    const none = mount(PassSummary, { props: { pass: null, commits: [] } })
    expect(none.get('[data-testid="pass-summary-no-commits"]').text()).toBe('No commit logged yet.')
  })

  it('wraps a long reply instead of cutting it off', () => {
    const pass = groupByPass([appliedInPass(1, 1, 'A reply long enough to run past the width of the review bar')]).passes[0] ?? null

    const wrapper = mount(PassSummary, { props: { pass, commits: [] } })

    const reply = wrapper.get('[data-testid="pass-summary-reply"]')
    expect(reply.text()).toBe('A reply long enough to run past the width of the review bar')
    expect(reply.classes()).not.toContain('truncate')
  })
})
