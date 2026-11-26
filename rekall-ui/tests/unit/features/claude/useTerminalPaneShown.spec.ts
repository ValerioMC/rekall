import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { defineComponent, type ComputedRef } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { useTerminalPaneShown } from '@/features/claude/composables/useTerminalPaneShown'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'
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
      shown = useTerminalPaneShown()
      return () => null
    }
  })
  mount(Host, { global: { plugins: [router] } })
  await flushPromises()
  return shown!
}

describe('useTerminalPaneShown', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    useConsoleStore().tasks = [task('t1', 'vega')]
  })

  it('is true on the console with the terminal pane on a task', async () => {
    useConsoleStore().selectTask('t1' as TaskId)
    useConsoleStore().openTerminal()

    expect((await shownAt('/console')).value).toBe(true)
  })

  it('is false away from the console', async () => {
    useConsoleStore().selectTask('t1' as TaskId)
    useConsoleStore().openTerminal()

    expect((await shownAt('/task/t1')).value).toBe(false)
  })

  it('is false on the console with another pane', async () => {
    useConsoleStore().selectTask('t1' as TaskId)

    expect((await shownAt('/console')).value).toBe(false)
  })

  it('is false with no task selected', async () => {
    useConsoleStore().paneFocus = 'terminal'

    expect((await shownAt('/console')).value).toBe(false)
  })
})
