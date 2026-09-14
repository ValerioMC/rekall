import { describe, expect, it, vi, beforeEach } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import CommitReferenceList from '@/components/console/CommitReferenceList.vue'
import { fetchCommitReferenceDiff } from '@/api/commitReference.api'
import type { CommitReference } from '@/model/commitReference'
import type { TaskId, TaskStepId } from '@/model/branded'

vi.mock('@/api/commitReference.api', () => ({
  fetchCommitReferenceDiff: vi.fn()
}))

const taskId = 't1' as TaskId
const stepId = 's1' as TaskStepId

function reference(overrides: Partial<CommitReference> = {}): CommitReference {
  return {
    id: 'c1',
    taskId,
    stepId: null,
    stepTitle: null,
    commitHash: 'abc1234def',
    comment: 'Wire up the button',
    createdAt: '2026-09-14T10:00:00Z',
    ...overrides
  }
}

describe('CommitReferenceList', () => {
  beforeEach(() => {
    vi.mocked(fetchCommitReferenceDiff).mockReset()
  })

  it('shows the short hash, the comment, and nothing expanded by default', () => {
    const wrapper = mount(CommitReferenceList, { props: { references: [reference()] } })

    expect(wrapper.get('[data-testid="commit-reference-hash"]').text()).toBe('abc1234')
    expect(wrapper.text()).toContain('Wire up the button')
    expect(wrapper.find('[data-testid="commit-reference-diff"]').exists()).toBe(false)
  })

  it('tags the row with its step, or "task" when it has none, only when asked to', () => {
    const withStep = mount(CommitReferenceList, {
      props: { references: [reference({ stepId, stepTitle: 'Wire the ledger' })], showStepTag: true }
    })
    expect(withStep.get('[data-testid="commit-reference-step-tag"]').text()).toBe('Wire the ledger')

    const withoutTag = mount(CommitReferenceList, { props: { references: [reference()] } })
    expect(withoutTag.find('[data-testid="commit-reference-step-tag"]').exists()).toBe(false)
  })

  it('fetches and shows the diff the first time a row is opened', async () => {
    vi.mocked(fetchCommitReferenceDiff).mockResolvedValue('diff --git a/x b/x\n+added\n-removed\n context')
    const wrapper = mount(CommitReferenceList, { props: { references: [reference()] } })

    await wrapper.get('[data-testid="commit-reference-toggle"]').trigger('click')
    await flushPromises()

    expect(fetchCommitReferenceDiff).toHaveBeenCalledWith('c1')
    expect(wrapper.get('[data-testid="commit-diff-view"]').text()).toContain('+added')
    expect(wrapper.get('[data-testid="commit-diff-view"]').text()).toContain('-removed')
  })

  it('does not refetch the diff when a row is collapsed and reopened', async () => {
    vi.mocked(fetchCommitReferenceDiff).mockResolvedValue('+one line')
    const wrapper = mount(CommitReferenceList, { props: { references: [reference()] } })
    const toggle = wrapper.get('[data-testid="commit-reference-toggle"]')

    await toggle.trigger('click')
    await flushPromises()
    await toggle.trigger('click')
    await toggle.trigger('click')
    await flushPromises()

    expect(fetchCommitReferenceDiff).toHaveBeenCalledTimes(1)
  })

  it('says so when a commit has no diff recorded', async () => {
    vi.mocked(fetchCommitReferenceDiff).mockResolvedValue(null)
    const wrapper = mount(CommitReferenceList, { props: { references: [reference()] } })

    await wrapper.get('[data-testid="commit-reference-toggle"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="commit-diff-empty"]').text()).toContain('No diff recorded')
  })

  it('reports when the diff could not be loaded', async () => {
    vi.mocked(fetchCommitReferenceDiff).mockRejectedValue(new Error('network down'))
    const wrapper = mount(CommitReferenceList, { props: { references: [reference()] } })

    await wrapper.get('[data-testid="commit-reference-toggle"]').trigger('click')
    await flushPromises()

    expect(wrapper.text()).toContain('Could not load the diff.')
  })
})
