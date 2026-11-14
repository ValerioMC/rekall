import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WorkPage from '@/features/work/components/WorkPage.vue'
import { router } from '@/router'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useWorkStore } from '@/features/work/store/work.store'
import { project, task } from './work-fixtures'

function seed() {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project('p1', 'Vega')]
  store.tasks = [
    task('PR-1', { title: 'Write the docs', status: 'TODO' }),
    task('PR-2', { title: 'Fix the login', quick: true })
  ]
  const setTaskStatus = vi.spyOn(store, 'setTaskStatus').mockResolvedValue()
  useWorkStore().view = 'open'
  return { setTaskStatus }
}

async function mountWork() {
  const wrapper = mount(WorkPage, { attachTo: document.body, global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

const rowOf = (wrapper: Awaited<ReturnType<typeof mountWork>>, title: string) =>
  wrapper.findAll('[data-testid="work-row"]').find((candidate) => candidate.text().includes(title))!

describe('the actions on a ledger row', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    document.body.innerHTML = ''
  })

  it('changes the status from the status column without opening the row', async () => {
    const { setTaskStatus } = seed()
    const wrapper = await mountWork()
    const row = rowOf(wrapper, 'Write the docs')

    await row.get('[data-testid="task-status"]').trigger('click')
    const blocked = row.findAll('[data-testid="popover-menu-item"]').find((item) => item.text() === 'Blocked')!
    await blocked.trigger('click')
    await flushPromises()

    expect(setTaskStatus).toHaveBeenCalledWith('PR-1', 'BLOCKED')
    expect(useToastStore().toasts.some((toast) => toast.message === 'Status set to blocked')).toBe(true)
    expect(wrapper.find('[data-testid="work-peek"]').exists()).toBe(false)
  })

  it('marks the stored status as current in the menu', async () => {
    seed()
    const wrapper = await mountWork()
    const row = rowOf(wrapper, 'Write the docs')

    await row.get('[data-testid="task-status"]').trigger('click')

    const current = row.findAll('[data-testid="popover-menu-item"]').filter((item) => item.attributes('aria-current') === 'true')
    expect(current.map((item) => item.text())).toEqual(['To do'])
  })

  it('opens the task from the arrow on the row in one press', async () => {
    seed()
    const push = vi.spyOn(router, 'push').mockResolvedValue(undefined)
    const wrapper = await mountWork()

    await rowOf(wrapper, 'Write the docs').get('[data-testid="work-row-open"]').trigger('click')

    expect(push).toHaveBeenCalledWith({ name: 'task', params: { id: 'PR-1', tab: 'description' } })
    expect(wrapper.find('[data-testid="work-peek"]').exists()).toBe(false)
  })

  it('badges a quick task and no other', async () => {
    seed()
    const wrapper = await mountWork()

    expect(rowOf(wrapper, 'Fix the login').find('[data-testid="work-row-quick"]').text()).toBe('Quick')
    expect(rowOf(wrapper, 'Write the docs').find('[data-testid="work-row-quick"]').exists()).toBe(false)
  })
})
