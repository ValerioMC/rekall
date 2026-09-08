import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ClaudeComposer from '@/components/claude/ClaudeComposer.vue'

describe('ClaudeComposer', () => {
  it('keeps send disabled until there is non-blank text', async () => {
    const wrapper = mount(ClaudeComposer, { props: { disabled: false } })
    const send = wrapper.get('[data-testid="claude-composer-send"]')

    expect((send.element as HTMLButtonElement).disabled).toBe(true)
    await wrapper.get('[data-testid="claude-composer-input"]').setValue('  ')
    expect((send.element as HTMLButtonElement).disabled).toBe(true)
    await wrapper.get('[data-testid="claude-composer-input"]').setValue('hello')
    expect((send.element as HTMLButtonElement).disabled).toBe(false)
  })

  it('emits the trimmed text and clears the field on submit', async () => {
    const wrapper = mount(ClaudeComposer, { props: { disabled: false } })
    await wrapper.get('[data-testid="claude-composer-input"]').setValue('  do the thing  ')
    await wrapper.get('form').trigger('submit')

    expect(wrapper.emitted('send')).toEqual([['do the thing']])
    expect((wrapper.get('[data-testid="claude-composer-input"]').element as HTMLTextAreaElement).value).toBe('')
  })

  it('sends on cmd/ctrl+enter', async () => {
    const wrapper = mount(ClaudeComposer, { props: { disabled: false } })
    await wrapper.get('[data-testid="claude-composer-input"]').setValue('go')
    await wrapper.get('[data-testid="claude-composer-input"]').trigger('keydown', {
      key: 'Enter',
      metaKey: true
    })

    expect(wrapper.emitted('send')).toEqual([['go']])
  })

  it('does not send while disabled', async () => {
    const wrapper = mount(ClaudeComposer, { props: { disabled: true, hint: 'ended' } })
    const input = wrapper.get('[data-testid="claude-composer-input"]')

    expect((input.element as HTMLTextAreaElement).disabled).toBe(true)
    await input.trigger('keydown', { key: 'Enter', metaKey: true })
    expect(wrapper.emitted('send')).toBeUndefined()
  })
})
