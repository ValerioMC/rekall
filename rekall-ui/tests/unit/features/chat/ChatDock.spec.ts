import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import ChatDock from '@/features/chat/components/ChatDock.vue'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { installStorage } from '../../support/memoryStorage'

vi.mock('@/features/chat/components/ChatWindow.vue', () => ({
  __esModule: true,
  default: defineComponent({
    setup: () => () => h('section', { 'data-testid': 'chat-window' }, [h('button', { 'data-testid': 'chat-hide' }, 'Hide')])
  })
}))

async function mountDock(shown: boolean) {
  const dock = mount(ChatDock, { props: { shown }, attachTo: document.body })
  await flushPromises()
  return dock
}

describe('ChatDock', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
    document.body.innerHTML = ''
  })

  it('renders no window until it has been shown once', async () => {
    const dock = await mountDock(false)

    expect(dock.find('[data-testid=chat-window]').exists()).toBe(false)
  })

  it('renders the window in its shown state with the width it gives back', async () => {
    useChatDockStore().observe(1200, true)

    const dock = await mountDock(true)
    const window = dock.get('[data-testid=chat-window]')

    expect(window.classes()).toContain('chat-dock')
    expect(window.attributes('data-state')).toBe('shown')
    expect(window.attributes('inert')).toBeUndefined()
    expect(window.attributes('style')).toContain('--chat-dock-width: 440px')
  })

  it('keeps the window mounted and inert once hidden, so its draft survives', async () => {
    const dock = await mountDock(true)

    await dock.setProps({ shown: false })
    const window = dock.get('[data-testid=chat-window]')

    expect(window.attributes('data-state')).toBe('hidden')
    expect(window.attributes('inert')).toBeDefined()
  })

  it('shows the same window again when reopened', async () => {
    const dock = await mountDock(true)
    await dock.setProps({ shown: false })
    const hidden = dock.get('[data-testid=chat-window]').element

    await dock.setProps({ shown: true })

    expect(dock.get('[data-testid=chat-window]').element).toBe(hidden)
    expect(dock.get('[data-testid=chat-window]').attributes('data-state')).toBe('shown')
  })

  it('mounts the window when it is first shown after the dock was created', async () => {
    const dock = await mountDock(false)

    await dock.setProps({ shown: true })
    await flushPromises()

    expect(dock.find('[data-testid=chat-window]').exists()).toBe(true)
  })

  it('sends focus to the note pane when the focused control is hidden away', async () => {
    const note = document.createElement('div')
    note.id = 'note'
    note.tabIndex = -1
    document.body.appendChild(note)
    const dock = await mountDock(true)
    dock.get<HTMLButtonElement>('[data-testid=chat-hide]').element.focus()

    await dock.setProps({ shown: false })

    expect(document.activeElement).toBe(note)
  })

  it('leaves focus alone when it is outside the window', async () => {
    const note = document.createElement('div')
    note.id = 'note'
    note.tabIndex = -1
    const other = document.createElement('button')
    document.body.append(note, other)
    const dock = await mountDock(true)
    other.focus()

    await dock.setProps({ shown: false })

    expect(document.activeElement).toBe(other)
  })
})
