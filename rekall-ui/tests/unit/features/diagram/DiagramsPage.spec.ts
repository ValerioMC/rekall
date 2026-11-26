import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import DiagramsPage from '@/features/diagram/components/DiagramsPage.vue'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TerminalId } from '@/model/branded'
import { CHAT_ID, chatSession } from '../chat/support'
import { installStorage } from '../../support/memoryStorage'

const push = vi.fn()
const replace = vi.fn()

vi.mock('vue-router', () => ({
  useRoute: () => ({ query: {} }),
  useRouter: () => ({ currentRoute: { value: { name: 'diagrams' } }, push, replace })
}))

vi.mock('@/features/diagram/composables/useDiagramStream', () => ({ useDiagramStream: vi.fn() }))

const Gallery = defineComponent({
  emits: ['watch'],
  setup: (_, { emit }) => () =>
    h('button', { 'data-testid': 'watch', onClick: () => emit('watch', { terminalId: 'chat-1' as TerminalId }) })
})

async function watchGeneration(): Promise<void> {
  const page = mount(DiagramsPage, {
    global: { stubs: { DiagramGallery: Gallery, AppCatalogHeader: true, LibrarySubNav: true, SessionDrawer: true } }
  })
  await page.get('[data-testid=watch]').trigger('click')
  await flushPromises()
}

describe('watching a generation on the Diagrams screen', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    push.mockReset()
    replace.mockReset()
  })

  it('shows a chat full-size', async () => {
    useTerminalStore().upsert(chatSession({ taskId: null }))

    await watchGeneration()

    expect([useConsoleStore().paneFocus, useChatStore().activeChatId]).toEqual(['chat', CHAT_ID])
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('opens a terminal in the drawer, on the screen', async () => {
    useTerminalStore().upsert(chatSession({ taskId: null, kind: 'TERMINAL' }))

    await watchGeneration()

    expect(replace).toHaveBeenCalledWith({ query: { session: CHAT_ID } })
    expect(push).not.toHaveBeenCalled()
  })
})
