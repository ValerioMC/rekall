import { describe, expect, it } from 'vitest'
import { mount, type VueWrapper } from '@vue/test-utils'
import ChatComposer from '@/features/chat/components/ChatComposer.vue'
import type { ComposerState } from '@/features/chat/lib/chat-status'

function mountComposer(over: { state?: ComposerState; sending?: boolean; stopping?: boolean; error?: string | null; followingNext?: boolean } = {}): VueWrapper {
  return mount(ChatComposer, {
    props: { state: 'idle', sending: false, stopping: false, error: null, followingNext: false, ...over },
    attachTo: document.body
  })
}

async function type(composer: VueWrapper, text: string): Promise<void> {
  await composer.get('[data-testid=chat-input]').setValue(text)
}

describe('ChatComposer', () => {
  it('sends on Enter and keeps the line break on Shift+Enter', async () => {
    const composer = mountComposer()
    await type(composer, 'hello')

    await composer.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter', shiftKey: true })
    expect(composer.emitted('send')).toBeUndefined()

    await composer.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter' })
    expect(composer.emitted('send')).toEqual([['hello']])
    composer.unmount()
  })

  it('does not send while an input method is composing, or an empty field', async () => {
    const composer = mountComposer()

    await composer.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter' })
    await type(composer, 'こんにちは')
    await composer.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter', isComposing: true })

    expect(composer.emitted('send')).toBeUndefined()
    composer.unmount()
  })

  it('keeps the text until the parent clears it', async () => {
    const composer = mountComposer()
    await type(composer, 'hello')
    await composer.get('[data-testid=chat-send]').trigger('click')

    expect((composer.get('[data-testid=chat-input]').element as HTMLTextAreaElement).value).toBe('hello')

    ;(composer.vm as unknown as { clear: () => void }).clear()
    await composer.vm.$nextTick()
    expect((composer.get('[data-testid=chat-input]').element as HTMLTextAreaElement).value).toBe('')
    composer.unmount()
  })

  it('is disabled with its reason once the session ended, and says the next one is coming', async () => {
    const ended = mountComposer({ state: 'ended' })
    const following = mountComposer({ state: 'ended', followingNext: true })

    expect(ended.get('[data-testid=chat-input]').attributes('disabled')).toBeDefined()
    expect(ended.get('[data-testid=chat-hint]').text()).toBe('Session ended')
    expect(following.get('[data-testid=chat-hint]').text()).toContain('Opening the next session')
    ended.unmount()
    following.unmount()
  })

  it('is paused with its reason while a permission waits', () => {
    const composer = mountComposer({ state: 'paused' })

    expect(composer.get('[data-testid=chat-input]').attributes('disabled')).toBeDefined()
    expect(composer.get('[data-testid=chat-hint]').text()).toBe('Paused: waiting for your approval')
    composer.unmount()
  })

  it('cannot send while connecting, but the field stays open', async () => {
    const composer = mountComposer({ state: 'connecting' })
    await type(composer, 'hello')

    expect(composer.get('[data-testid=chat-input]').attributes('disabled')).toBeUndefined()
    expect(composer.get('[data-testid=chat-send]').attributes('disabled')).toBeDefined()
    composer.unmount()
  })

  it('offers Stop only while a turn runs, and says Stopping once asked', async () => {
    const idle = mountComposer()
    const working = mountComposer({ state: 'working' })

    expect(idle.find('[data-testid=chat-stop]').exists()).toBe(false)
    await working.get('[data-testid=chat-stop]').trigger('click')
    expect(working.emitted('stop')).toHaveLength(1)

    await working.setProps({ stopping: true })
    expect(working.get('[data-testid=chat-stop]').text()).toBe('Stopping…')
    expect(working.get('[data-testid=chat-stop]').attributes('disabled')).toBeDefined()
    idle.unmount()
    working.unmount()
  })

  it('lists the commands that work headless for a leading slash, and only fills the field', async () => {
    const composer = mountComposer()
    await type(composer, '/')

    const names = composer.findAll('[role=option]').map((option) => option.find('span').text())
    expect(names).toEqual(['/context', '/model', '/effort', '/compact', '/rk'])
    expect(names).not.toContain('/clear')

    await composer.get('[data-testid=chat-input]').trigger('keydown', { key: 'ArrowDown' })
    await composer.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter' })

    expect((composer.get('[data-testid=chat-input]').element as HTMLTextAreaElement).value).toBe('/model ')
    expect(composer.emitted('send')).toBeUndefined()
    expect(composer.find('[role=listbox]').exists()).toBe(false)
    composer.unmount()
  })

  it('sends a command typed in full, and closes the list on Escape without leaving the window', async () => {
    const composer = mountComposer()
    await type(composer, '/compact')
    await composer.get('[data-testid=chat-input]').trigger('keydown', { key: 'Enter' })
    expect(composer.emitted('send')).toEqual([['/compact']])

    await type(composer, '/co')
    const escape = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true })
    composer.get('[data-testid=chat-input]').element.dispatchEvent(escape)
    await composer.vm.$nextTick()
    expect(composer.find('[role=listbox]').exists()).toBe(false)
    composer.unmount()
  })

  it('shows a failed send under the field', () => {
    const composer = mountComposer({ error: 'Session is gone' })

    expect(composer.get('[role=alert]').text()).toBe('Session is gone')
    composer.unmount()
  })
})
