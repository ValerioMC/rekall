import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { defineComponent, type ComputedRef } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { useChatPaneShown } from '@/features/chat/composables/useChatPaneShown'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId, TerminalId } from '@/model/branded'
import { task } from '../workingset/fixtures'

const Page = { render: () => null }

async function shownAt(path: string): Promise<ComputedRef<boolean>> {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/task/:id', name: 'task', component: Page },
      { path: '/console', name: 'console', component: Page }
    ]
  })
  await router.push(path)
  let shown: ComputedRef<boolean> | undefined
  const Host = defineComponent({
    setup() {
      shown = useChatPaneShown()
      return () => null
    }
  })
  mount(Host, { global: { plugins: [router] } })
  await flushPromises()
  return shown!
}

describe('useChatPaneShown', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    useConsoleStore().tasks = [task('t1', 'vega')]
  })

  it('is true on the console with the chat pane on a task', async () => {
    useConsoleStore().selectTask('t1' as TaskId)
    useConsoleStore().openChat()

    expect((await shownAt('/console')).value).toBe(true)
  })

  it('is true on the console with a chat on no task', async () => {
    useConsoleStore().openProjectChat('chat-9' as TerminalId)

    expect((await shownAt('/console')).value).toBe(true)
  })

  it('is false away from the console', async () => {
    useConsoleStore().openProjectChat('chat-9' as TerminalId)

    expect((await shownAt('/task/t1')).value).toBe(false)
  })

  it('is false on the console with another pane', async () => {
    useConsoleStore().selectTask('t1' as TaskId)
    useConsoleStore().openTerminal()

    expect((await shownAt('/console')).value).toBe(false)
  })

  it('is false with neither a task nor a chat to show', async () => {
    useConsoleStore().paneFocus = 'chat'

    expect((await shownAt('/console')).value).toBe(false)
  })
})
