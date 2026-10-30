import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ChatMessage from '@/features/chat/components/ChatMessage.vue'
import { MarkdownStub } from './support'

function mountMessage(props: { role: 'user' | 'assistant'; text: string; complete: boolean }) {
  return mount(ChatMessage, { props, global: { stubs: { AppMarkdownEditor: MarkdownStub } } })
}

describe('ChatMessage', () => {
  beforeEach(() => vi.useFakeTimers())
  afterEach(() => vi.useRealTimers())

  it('shows a streamed message growing, repainted at most every 120 ms', async () => {
    const message = mountMessage({ role: 'assistant', text: 'Hel', complete: false })
    expect(message.get('[data-testid=markdown]').text()).toBe('Hel')

    await message.setProps({ text: 'Hello' })
    await message.setProps({ text: 'Hello wor' })
    expect(message.get('[data-testid=markdown]').text()).toBe('Hel')

    await vi.advanceTimersByTimeAsync(120)
    expect(message.get('[data-testid=markdown]').text()).toBe('Hello wor')
  })

  it('is busy and carries a caret until the message is complete, then shows all of it at once', async () => {
    const message = mountMessage({ role: 'assistant', text: 'Hel', complete: false })
    expect(message.attributes('aria-busy')).toBe('true')
    expect(message.attributes('aria-live')).toBe('polite')
    expect(message.find('[data-testid=chat-caret]').exists()).toBe(true)

    await message.setProps({ text: 'Hello there', complete: true })

    expect(message.get('[data-testid=markdown]').text()).toBe('Hello there')
    expect(message.attributes('aria-busy')).toBe('false')
    expect(message.find('[data-testid=chat-caret]').exists()).toBe(false)
  })

  it('shows the person\'s message as plain text, markup and all', () => {
    const message = mountMessage({ role: 'user', text: '**not bold**\nsecond line', complete: true })

    expect(message.find('[data-testid=markdown]').exists()).toBe(false)
    expect(message.text()).toContain('**not bold**')
    expect(message.text()).not.toContain('command')
  })

  it('marks a slash command as one', () => {
    const message = mountMessage({ role: 'user', text: '/rk project:vega task:report', complete: true })

    expect(message.text()).toContain('command')
    expect(message.get('p.font-mono').text()).toBe('/rk project:vega task:report')
  })
})
