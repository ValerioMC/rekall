import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import FeedbackEntry from '@/features/feedback/components/FeedbackEntry.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import type { Feedback } from '@/features/feedback/model/feedback'
import { applied, entry, revision, running } from './fixtures'

function view(feedback: Feedback) {
  return mount(FeedbackEntry, { props: { entry: feedback }, global: { stubs: { AppMarkdownEditor: true } } })
}

describe('a message of the thread', () => {
  it('names its kind and number and offers edit and delete while it is pending', () => {
    const wrapper = view(entry(3, { kind: 'PLAN' }))

    expect(wrapper.get('[data-testid="feedback-kind"]').text()).toBe('Feedback on the plan')
    expect(wrapper.text()).toContain('#3')
    expect(wrapper.get('[data-testid="feedback-state"]').text()).toBe('Waiting for the next run')
    expect(wrapper.find('[data-testid="feedback-edit"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="feedback-delete"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="feedback-reply"]').exists()).toBe(false)
  })

  it('is sealed once applied: no edit, no delete, and the reply shown below it', () => {
    const wrapper = view(applied(1, 'Fixed the totals in the report view.'))

    expect(wrapper.attributes('data-state')).toBe('APPLIED')
    expect(wrapper.find('[data-testid="feedback-edit"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="feedback-delete"]').exists()).toBe(false)
    const reply = wrapper.get('[data-testid="feedback-reply"]')
    expect(reply.text()).toContain('Session replied')
    expect(reply.getComponent(AppMarkdownEditor).props('modelValue')).toBe('Fixed the totals in the report view.')
  })

  it('says which pass an entry is in and offers neither edit nor delete while it is', () => {
    const wrapper = view(running(4, 2))

    expect(wrapper.attributes('data-state')).toBe('RUNNING')
    expect(wrapper.get('[data-testid="feedback-state"]').text()).toBe('In pass 2')
    expect(wrapper.find('[data-testid="feedback-edit"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="feedback-delete"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="feedback-follow-up"]').exists()).toBe(false)
  })

  it('offers Add detail on an applied entry only, and names the entry a follow-up continues', async () => {
    const wrapper = view(applied(1, 'Done.'))
    await wrapper.get('[data-testid="feedback-follow-up"]').trigger('click')

    expect(wrapper.emitted('followUp')).toEqual([[applied(1, 'Done.')]])
    expect(view(entry(2)).find('[data-testid="feedback-follow-up"]').exists()).toBe(false)
    expect(view(entry(3, { followsSeq: 1 })).get('[data-testid="feedback-follows"]').text()).toBe('Follows #1')
  })

  it('collapses an applied entry to one line holding the first line of the reply', async () => {
    const wrapper = view(applied(1, 'Fixed the totals.\nSecond line.'))

    await wrapper.get('[data-testid="feedback-collapse"]').trigger('click')

    expect(wrapper.get('[data-testid="feedback-summary"]').text()).toBe('Fixed the totals.')
    expect(wrapper.find('[data-testid="feedback-reply"]').exists()).toBe(false)
  })

  it('edits a pending entry in an editor and saves the new text', async () => {
    const wrapper = view(entry(1))

    await wrapper.get('[data-testid="feedback-edit"]').trigger('click')
    wrapper.getComponent(AppMarkdownEditor).vm.$emit('update:modelValue', 'Better words')
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-testid="feedback-save"]').trigger('click')

    expect(wrapper.emitted('edit')).toEqual([[entry(1), 'Better words']])
    expect(wrapper.find('[data-testid="feedback-editing"]').exists()).toBe(false)
  })

  it('asks to delete a pending entry', async () => {
    const wrapper = view(entry(1))

    await wrapper.get('[data-testid="feedback-delete"]').trigger('click')

    expect(wrapper.emitted('remove')).toEqual([[entry(1)]])
  })

  it('shows a pending revision as a diff with discard, and neither edit nor delete', async () => {
    const wrapper = mount(FeedbackEntry, { props: { entry: revision(1, 'before', 'after') }, global: { stubs: { AppMarkdownEditor: true } } })

    expect(wrapper.find('[data-testid="revision-entry"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="feedback-edit"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="feedback-delete"]').exists()).toBe(false)
    await wrapper.get('[data-testid="revision-discard"]').trigger('click')
    expect(wrapper.emitted('discard')).toEqual([[revision(1, 'before', 'after')]])
  })

  it('animates a reply that arrives while it is on screen, and not one that was already there', async () => {
    const wrapper = view(entry(1))
    await wrapper.setProps({ entry: applied(1, 'Done.') })
    expect(wrapper.get('[data-testid="feedback-reply"]').classes()).toContain('reply-arrived')

    const inPass = view(running(3, 1))
    await inPass.setProps({ entry: applied(3, 'Done.') })
    expect(inPass.get('[data-testid="feedback-reply"]').classes()).toContain('reply-arrived')

    expect(view(applied(2, 'Done.')).get('[data-testid="feedback-reply"]').classes()).not.toContain('reply-arrived')
  })
})
