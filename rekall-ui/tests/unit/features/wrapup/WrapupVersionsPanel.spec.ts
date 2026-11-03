import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WrapupVersionsPanel from '@/features/wrapup/components/WrapupVersionsPanel.vue'
import * as api from '@/features/wrapup/api/wrapup-versions.api'
import { useConsoleStore } from '@/stores/console.store'
import { TASK, detail, version } from './fixtures'

vi.mock('@/features/wrapup/api/wrapup-versions.api')

async function render() {
  const wrapper = mount(WrapupVersionsPanel, {
    props: { taskId: TASK, wrapupUpdatedAt: '2026-11-03T10:00:00Z' },
    global: { stubs: { AppMarkdownEditor: { props: ['modelValue'], template: '<div data-testid="text">{{ modelValue }}</div>' } } }
  })
  await flushPromises()
  return wrapper
}

describe('the versions panel of a wrapup', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('says so when the wrapup has no earlier text', async () => {
    vi.mocked(api.fetchWrapupVersions).mockResolvedValue([])

    const wrapper = await render()

    expect(wrapper.find('[data-testid="wrapup-versions-empty"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="wrapup-version"]').exists()).toBe(false)
  })

  it('lists the versions with author, size and commit, and reads one whole when picked', async () => {
    vi.mocked(api.fetchWrapupVersions).mockResolvedValue([version(2, 'HAND', 'abcdef1234567'), version(1)])
    vi.mocked(api.fetchWrapupVersion).mockResolvedValue(detail(1))
    const wrapper = await render()

    const rows = wrapper.findAll('[data-testid="wrapup-version"]')
    expect(rows).toHaveLength(2)
    expect(rows[0]?.text()).toContain('v2')
    expect(rows[0]?.text()).toContain('you')
    expect(rows[0]?.text()).toContain('22 chars')
    expect(rows[0]?.text()).toContain('at abcdef1')
    await rows[1]?.trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="text"]').text()).toBe('Whole text of state 1')
  })

  it('restores the picked version and tells the pane', async () => {
    vi.mocked(api.fetchWrapupVersions).mockResolvedValue([version(2), version(1)])
    vi.mocked(api.fetchWrapupVersion).mockResolvedValue(detail(1))
    const saveWrapupBody = vi.spyOn(useConsoleStore(), 'saveWrapupBody').mockResolvedValue()
    const wrapper = await render()
    await wrapper.findAll('[data-testid="wrapup-version"]')[1]?.trigger('click')
    await flushPromises()

    await wrapper.get('[data-testid="restore-wrapup-version"]').trigger('click')
    await flushPromises()

    expect(saveWrapupBody).toHaveBeenCalledWith(TASK, 'Whole text of state 1')
    expect(wrapper.emitted('restored')).toHaveLength(1)
  })
})
