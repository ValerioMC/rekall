import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import RevisionCard from '@/features/feedback/components/RevisionCard.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import type { Feedback } from '@/features/feedback/model/feedback'
import { applied, entry, revision, running } from './fixtures'

function view(feedback: Feedback) {
  return mount(RevisionCard, { props: { entry: feedback }, global: { stubs: { AppMarkdownEditor: true } } })
}

describe('a revision of the description', () => {
  it('names what it is applied to and its number, and shows the change with a way to discard it while it waits', async () => {
    const wrapper = view(revision(3, 'before', 'after', { kind: 'PLAN' }))

    expect(wrapper.get('[data-testid="revision-kind"]').text()).toBe('On the plan')
    expect(wrapper.text()).toContain('#3')
    expect(wrapper.get('[data-testid="revision-state"]').text()).toBe('Waiting for the next run')
    expect(wrapper.find('[data-testid="revision-reply"]').exists()).toBe(false)
    await wrapper.get('[data-testid="revision-discard"]').trigger('click')
    expect(wrapper.emitted('discard')).toEqual([[revision(3, 'before', 'after', { kind: 'PLAN' })]])
  })

  it('says it is applied to the work', () => {
    expect(view(entry(1)).get('[data-testid="revision-kind"]').text()).toBe('On the work')
  })

  it('is sealed once applied: no discard, and the reply shown below it', () => {
    const wrapper = view(applied(1, 'Fixed the totals in the report view.'))

    expect(wrapper.attributes('data-state')).toBe('APPLIED')
    expect(wrapper.find('[data-testid="revision-discard"]').exists()).toBe(false)
    const reply = wrapper.get('[data-testid="revision-reply"]')
    expect(reply.text()).toContain('Session replied')
    expect(reply.getComponent(AppMarkdownEditor).props('modelValue')).toBe('Fixed the totals in the report view.')
  })

  it('says which pass it is in and offers no discard while a session is on it', () => {
    const wrapper = view(running(4, 2))

    expect(wrapper.attributes('data-state')).toBe('RUNNING')
    expect(wrapper.get('[data-testid="revision-state"]').text()).toBe('In pass 2')
    expect(wrapper.find('[data-testid="revision-discard"]').exists()).toBe(false)
  })

  it('collapses an applied revision to one line holding the first line of the reply', async () => {
    const wrapper = view(applied(1, 'Fixed the totals.\nSecond line.'))

    await wrapper.get('[data-testid="revision-collapse"]').trigger('click')

    expect(wrapper.get('[data-testid="revision-summary"]').text()).toBe('Fixed the totals.')
    expect(wrapper.find('[data-testid="revision-reply"]').exists()).toBe(false)
  })

  it('animates a reply that arrives while it is on screen, and not one that was already there', async () => {
    const wrapper = view(entry(1))
    await wrapper.setProps({ entry: applied(1, 'Done.') })
    expect(wrapper.get('[data-testid="revision-reply"]').classes()).toContain('reply-arrived')

    const inPass = view(running(3, 1))
    await inPass.setProps({ entry: applied(3, 'Done.') })
    expect(inPass.get('[data-testid="revision-reply"]').classes()).toContain('reply-arrived')

    expect(view(applied(2, 'Done.')).get('[data-testid="revision-reply"]').classes()).not.toContain('reply-arrived')
  })
})
