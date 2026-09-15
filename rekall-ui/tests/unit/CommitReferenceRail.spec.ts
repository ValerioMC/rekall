import { describe, expect, it, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { mount } from '@vue/test-utils'
import CommitReferenceRail from '@/components/console/CommitReferenceRail.vue'
import { fetchCommitReferenceDiff } from '@/api/commitReference.api'
import type { CommitReference } from '@/model/commitReference'
import type { TaskId } from '@/model/branded'

vi.mock('@/api/commitReference.api', () => ({
  fetchCommitReferenceDiff: vi.fn()
}))

const taskId = 't1' as TaskId

function reference(overrides: Partial<CommitReference> = {}): CommitReference {
  return {
    id: 'c1',
    taskId,
    stepId: null,
    stepTitle: null,
    commitHash: 'abc1234def',
    comment: 'Wire up the button',
    inContext: false,
    createdAt: '2026-09-14T10:00:00Z',
    ...overrides
  }
}

function manyReferences(count: number): CommitReference[] {
  return Array.from({ length: count }, (_, index) =>
    reference({ id: `c${index}`, commitHash: `${index}`.padStart(7, '0'), comment: `Commit ${index}` })
  )
}

describe('CommitReferenceRail', () => {
  let pinia: Pinia

  beforeEach(() => {
    vi.mocked(fetchCommitReferenceDiff).mockReset()
    pinia = createPinia()
    setActivePinia(pinia)
  })

  it('starts collapsed, showing only the latest commit and the total count', () => {
    const wrapper = mount(CommitReferenceRail, { props: { references: manyReferences(9) } })

    expect(wrapper.get('[data-testid="commit-rail-toggle"]').attributes('aria-expanded')).toBe('false')
    expect(wrapper.get('[data-testid="commit-rail-latest"]').text()).toContain('Commit 0')
    expect(wrapper.text()).toContain('9 commits')
  })

  it('caps the node stack and reports the rest as an overflow count', () => {
    const wrapper = mount(CommitReferenceRail, { props: { references: manyReferences(9) } })

    expect(wrapper.text()).toContain('+3')
  })

  it('does not show an overflow count when every commit fits the stack', () => {
    const wrapper = mount(CommitReferenceRail, { props: { references: manyReferences(3) } })

    expect(wrapper.text()).not.toContain('+0')
  })

  it('expands the full commit list on toggle, however many commits there are', async () => {
    const wrapper = mount(CommitReferenceRail, { props: { references: manyReferences(9) } })

    await wrapper.get('[data-testid="commit-rail-toggle"]').trigger('click')

    expect(wrapper.get('[data-testid="commit-rail-toggle"]').attributes('aria-expanded')).toBe('true')
    expect(wrapper.findAll('[data-testid="commit-reference-row"]')).toHaveLength(9)
  })

  it('keeps the expanded panel height-capped so it never squeezes the space around it', async () => {
    const wrapper = mount(CommitReferenceRail, { props: { references: manyReferences(20) } })

    await wrapper.get('[data-testid="commit-rail-toggle"]').trigger('click')

    const panel = wrapper.get('[data-testid="commit-rail-panel"]').element.firstElementChild as HTMLElement
    expect(panel.className).toContain('max-h-[260px]')
    expect(panel.className).toContain('overflow-y-auto')
  })

  it('collapses back to the compact row on a second toggle', async () => {
    const wrapper = mount(CommitReferenceRail, { props: { references: manyReferences(3) } })
    const toggle = wrapper.get('[data-testid="commit-rail-toggle"]')

    await toggle.trigger('click')
    await toggle.trigger('click')

    expect(toggle.attributes('aria-expanded')).toBe('false')
  })

  it('fills the nodes of the commits that travel with /rk and counts them', () => {
    const wrapper = mount(CommitReferenceRail, {
      props: { references: [reference({ id: 'c1', inContext: true }), reference({ id: 'c2' })] }
    })

    const nodes = wrapper.findAll('[data-testid="commit-rail-node"]')
    expect(nodes.map((node) => node.attributes('data-in-context'))).toEqual(['true', 'false'])
    expect(wrapper.get('[data-testid="commit-rail-context-count"]').text()).toBe('1 in context')
  })

  it('says nothing about the context when no commit has been chosen', () => {
    const wrapper = mount(CommitReferenceRail, { props: { references: manyReferences(3) } })

    expect(wrapper.find('[data-testid="commit-rail-context-count"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('3 commits')
    expect(wrapper.text()).not.toContain(',')
  })
})
