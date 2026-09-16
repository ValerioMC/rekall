import { describe, expect, it, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import CommitReferenceList from '@/components/console/CommitReferenceList.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import { fetchCommitReferenceDiff } from '@/api/commitReference.api'
import { useConsoleStore } from '@/stores/console.store'
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
    inContext: false,
    createdAt: '2026-09-14T10:00:00Z',
    ...overrides
  }
}

function render(references: CommitReference[]) {
  return mount(CommitReferenceList, {
    props: { references },
    global: { stubs: { AppConfirm: true } }
  })
}

describe('CommitReferenceList', () => {
  let pinia: Pinia

  beforeEach(() => {
    vi.mocked(fetchCommitReferenceDiff).mockReset()
    pinia = createPinia()
    setActivePinia(pinia)
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

  it('offers to add a commit to the context, and says so once it is in', () => {
    const off = render([reference()])
    const offToggle = off.get('[data-testid="commit-reference-context-toggle"]')
    expect(offToggle.text()).toBe('add to context')
    expect(offToggle.attributes('aria-pressed')).toBe('false')

    const on = render([reference({ inContext: true })])
    const onToggle = on.get('[data-testid="commit-reference-context-toggle"]')
    expect(onToggle.text()).toBe('in context')
    expect(onToggle.attributes('aria-pressed')).toBe('true')
    expect(onToggle.attributes('data-in-context')).toBe('true')
  })

  it('chooses the commit for the context through the store, without opening the diff', async () => {
    const store = useConsoleStore()
    store.setCommitReferenceInContext = vi.fn().mockResolvedValue(undefined)
    const wrapper = render([reference({ id: 'c1' })])

    await wrapper.get('[data-testid="commit-reference-context-toggle"]').trigger('click')
    await flushPromises()

    expect(store.setCommitReferenceInContext).toHaveBeenCalledWith('c1', true)
    expect(fetchCommitReferenceDiff).not.toHaveBeenCalled()
    expect(wrapper.find('[data-testid="commit-reference-diff"]').exists()).toBe(false)
  })

  it('unchooses a commit that is already in the context', async () => {
    const store = useConsoleStore()
    store.setCommitReferenceInContext = vi.fn().mockResolvedValue(undefined)
    const wrapper = render([reference({ id: 'c1', inContext: true })])

    await wrapper.get('[data-testid="commit-reference-context-toggle"]').trigger('click')
    await flushPromises()

    expect(store.setCommitReferenceInContext).toHaveBeenCalledWith('c1', false)
  })

  it('asks for confirmation before deleting a commit reference', async () => {
    const wrapper = render([reference()])

    await wrapper.get('[data-testid="commit-reference-delete"]').trigger('click')

    expect(wrapper.findComponent(AppConfirm).exists()).toBe(true)
  })

  it('deletes the commit reference through the store once confirmed', async () => {
    const store = useConsoleStore()
    store.deleteCommitReference = vi.fn().mockResolvedValue(undefined)
    const wrapper = render([reference({ id: 'c1' })])

    await wrapper.get('[data-testid="commit-reference-delete"]').trigger('click')
    await wrapper.findComponent(AppConfirm).vm.$emit('confirm')
    await flushPromises()

    expect(store.deleteCommitReference).toHaveBeenCalledWith('c1')
    expect(wrapper.findComponent(AppConfirm).exists()).toBe(false)
  })

  it('does not delete anything when the confirmation is cancelled', async () => {
    const store = useConsoleStore()
    store.deleteCommitReference = vi.fn().mockResolvedValue(undefined)
    const wrapper = render([reference()])

    await wrapper.get('[data-testid="commit-reference-delete"]').trigger('click')
    await wrapper.findComponent(AppConfirm).vm.$emit('cancel')

    expect(store.deleteCommitReference).not.toHaveBeenCalled()
    expect(wrapper.findComponent(AppConfirm).exists()).toBe(false)
  })
})
