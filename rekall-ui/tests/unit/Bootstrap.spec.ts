import { afterEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import Bootstrap from '@/Bootstrap.vue'
import { router } from '@/router'
import { fetchWrapups } from '@/features/wrapup/api/wrapups.api'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { TaskId } from '@/model/branded'
import { FakeEventSource, chatSession } from './features/chat/support'

/**
 * The boot-time branch: which of the three top-level screens mounts. The screens themselves are
 * stubbed, because what is under test here is the branching, not their contents — those have
 * their own specs. The routed screens (the console among them) render through `router-view` now,
 * so the ready case needs the router installed and the catalog fetches stubbed, the same way
 * `store.load()` is stubbed everywhere else it runs in a test.
 */
const fetchDatabaseStatus = vi.fn()

vi.mock('@/features/settings/api/settings.api', () => ({ fetchDatabaseStatus: () => fetchDatabaseStatus() }))

vi.mock('@/features/catalog/api/catalog.api', () => ({
  fetchCompanies: vi.fn(async () => []),
  fetchProjects: vi.fn(async () => []),
  fetchTasks: vi.fn(async () => []),
  fetchTags: vi.fn(async () => [])
}))
vi.mock('@/features/notes/api/documents.api', () => ({ fetchAllDocuments: vi.fn(async () => []) }))
vi.mock('@/features/claude/api/terminal.api', () => ({ fetchTerminals: vi.fn(async () => []) }))
vi.mock('@/features/wrapup/api/wrapups.api', () => ({ fetchWrapups: vi.fn(async () => []) }))
vi.mock('@/features/steps/api/steps.api', () => ({ fetchSteps: vi.fn(async () => []) }))
vi.mock('@/features/workingset/api/working-set.api', () => ({ fetchWorkingSet: vi.fn(async () => ({ tasks: [], places: [] })) }))
vi.mock('@/features/time/api/time-entries.api', () => ({ fetchTimeEntries: vi.fn(async () => []) }))
vi.mock('@/features/skills/api/skill-assignments.api', () => ({
  fetchSkillAssignments: vi.fn(async () => ({ taskSkills: [], tagSkills: [] })),
  setTaskSkills: vi.fn(),
  setStepSkills: vi.fn(),
  setTagSkills: vi.fn()
}))

vi.mock('@/features/commits/api/commitReference.api', () => ({
  fetchCommitReferences: vi.fn(async () => [])
}))

vi.mock('@/App.vue', () => ({
  default: { name: 'AppStub', template: '<div data-testid="app-stub" />' }
}))
vi.mock('@/features/settings/components/FirstRunSetup.vue', () => ({
  default: { name: 'FirstRunSetupStub', template: '<div data-testid="first-run-stub" />' }
}))
vi.mock('@/features/settings/components/DatabaseUnreachable.vue', () => ({
  default: {
    name: 'DatabaseUnreachableStub',
    props: ['status'],
    template: '<div data-testid="unreachable-stub" />'
  }
}))

/**
 * Unmounted after every test, and not only for tidiness: this component listens on `window`,
 * so one left mounted would answer the next test's events alongside the one under it.
 */
const mounted: ReturnType<typeof mount>[] = []

async function mountBootstrap() {
  setActivePinia(createPinia())
  await router.push('/console')
  const wrapper = mount(Bootstrap, { global: { plugins: [router] } })
  mounted.push(wrapper)
  await flushPromises()
  return wrapper
}

afterEach(() => {
  mounted.splice(0).forEach((wrapper) => wrapper.unmount())
  vi.unstubAllGlobals()
})

describe('Bootstrap', () => {
  it('mounts the console when the database is ready', async () => {
    fetchDatabaseStatus.mockResolvedValue({ status: 'READY', active: null, databases: [] })

    const wrapper = await mountBootstrap()
    await flushPromises()

    expect(wrapper.find('[data-testid="app-stub"]').exists()).toBe(true)
  })

  it('draws the toasts itself, so every screen shows them', async () => {
    fetchDatabaseStatus.mockResolvedValue({ status: 'READY', active: null, databases: [] })
    const wrapper = await mountBootstrap()

    useToastStore().notify('1 unused image found')
    await flushPromises()

    expect(wrapper.get('[data-testid="toast"]').text()).toContain('1 unused image found')
  })

  /**
   * The loop ends outside this window: a session writes the wrapup through MCP, and what was on
   * screen when it did knows nothing about it. Coming back to the window is when the answer is
   * wanted, and reloading the page to get it is not an answer.
   */
  it('reads everything again when the window comes back to the front', async () => {
    fetchDatabaseStatus.mockResolvedValue({ status: 'READY', active: null, databases: [] })
    await mountBootstrap()
    vi.mocked(fetchWrapups).mockClear()

    window.dispatchEvent(new Event('focus'))
    await flushPromises()

    expect(fetchWrapups).toHaveBeenCalledOnce()
  })

  /** A pane holds the draft being typed. Replacing the record underneath it loses a paragraph. */
  it('leaves the screen alone while something on it is waiting to be saved', async () => {
    fetchDatabaseStatus.mockResolvedValue({ status: 'READY', active: null, databases: [] })
    await mountBootstrap()
    useConsoleStore().saveState = 'unsaved'
    vi.mocked(fetchWrapups).mockClear()

    window.dispatchEvent(new Event('focus'))
    await flushPromises()

    expect(fetchWrapups).not.toHaveBeenCalled()
  })

  it('mounts the first-run wizard when nothing is configured yet', async () => {
    fetchDatabaseStatus.mockResolvedValue({ status: 'SETUP_NEEDED', active: null, databases: [] })

    const wrapper = await mountBootstrap()

    expect(wrapper.find('[data-testid="first-run-stub"]').exists()).toBe(true)
  })

  it('mounts the recovery screen when the configured database cannot be reached', async () => {
    fetchDatabaseStatus.mockResolvedValue({
      status: 'UNREACHABLE',
      active: { id: '1', label: 'Local', path: '/a', active: true, reachable: false, addedAt: 't', lastUsedAt: 't' },
      databases: []
    })

    const wrapper = await mountBootstrap()

    expect(wrapper.find('[data-testid="unreachable-stub"]').exists()).toBe(true)
  })

  it('shows a plain message if the status call itself cannot be reached', async () => {
    fetchDatabaseStatus.mockRejectedValue(new Error('network error'))

    const wrapper = await mountBootstrap()

    expect(wrapper.text()).toContain("couldn't be reached")
  })

  it('follows the selected task\'s chat from any screen', async () => {
    FakeEventSource.instances = []
    vi.stubGlobal('EventSource', FakeEventSource)
    fetchDatabaseStatus.mockResolvedValue({ status: 'READY', active: null, databases: [] })
    await mountBootstrap()

    useTerminalStore().upsert(chatSession())
    useConsoleStore().selectedTaskId = 't-1' as TaskId
    await flushPromises()

    expect(FakeEventSource.instances.map((source) => source.url)).toContain('/api/chat/chat-1/events?after=0')
  })
})
