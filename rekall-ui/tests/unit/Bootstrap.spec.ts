import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import Bootstrap from '@/Bootstrap.vue'

/**
 * The boot-time branch: which of the three top-level screens mounts. The screens themselves are
 * stubbed, because what is under test here is the branching, not their contents — those have
 * their own specs.
 */
const fetchDatabaseStatus = vi.fn()

vi.mock('@/api/settings.api', () => ({ fetchDatabaseStatus: () => fetchDatabaseStatus() }))

vi.mock('@/App.vue', () => ({
  default: { name: 'AppStub', template: '<div data-testid="app-stub" />' }
}))
vi.mock('@/components/setup/FirstRunSetup.vue', () => ({
  default: { name: 'FirstRunSetupStub', template: '<div data-testid="first-run-stub" />' }
}))
vi.mock('@/components/setup/DatabaseUnreachable.vue', () => ({
  default: {
    name: 'DatabaseUnreachableStub',
    props: ['status'],
    template: '<div data-testid="unreachable-stub" />'
  }
}))

describe('Bootstrap', () => {
  it('mounts the console when the database is ready', async () => {
    fetchDatabaseStatus.mockResolvedValue({ status: 'READY', active: null, databases: [] })

    const wrapper = mount(Bootstrap)
    await flushPromises()

    expect(wrapper.find('[data-testid="app-stub"]').exists()).toBe(true)
  })

  it('mounts the first-run wizard when nothing is configured yet', async () => {
    fetchDatabaseStatus.mockResolvedValue({ status: 'SETUP_NEEDED', active: null, databases: [] })

    const wrapper = mount(Bootstrap)
    await flushPromises()

    expect(wrapper.find('[data-testid="first-run-stub"]').exists()).toBe(true)
  })

  it('mounts the recovery screen when the configured database cannot be reached', async () => {
    fetchDatabaseStatus.mockResolvedValue({
      status: 'UNREACHABLE',
      active: { id: '1', label: 'Local', path: '/a', active: true, reachable: false, addedAt: 't', lastUsedAt: 't' },
      databases: []
    })

    const wrapper = mount(Bootstrap)
    await flushPromises()

    expect(wrapper.find('[data-testid="unreachable-stub"]').exists()).toBe(true)
  })

  it('shows a plain message if the status call itself cannot be reached', async () => {
    fetchDatabaseStatus.mockRejectedValue(new Error('network error'))

    const wrapper = mount(Bootstrap)
    await flushPromises()

    expect(wrapper.text()).toContain("couldn't be reached")
  })
})
