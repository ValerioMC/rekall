import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import WrapupFreshnessTag from '@/features/wrapup/components/WrapupFreshnessTag.vue'
import * as api from '@/features/wrapup/api/wrapup-versions.api'
import { TASK } from './fixtures'

vi.mock('@/features/wrapup/api/wrapup-versions.api')

async function render() {
  const wrapper = mount(WrapupFreshnessTag, { props: { taskId: TASK, wrapupUpdatedAt: '2026-11-03T10:00:00Z' } })
  await flushPromises()
  return wrapper
}

describe('the freshness tag of a wrapup', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('lists the changed files and the commits when the wrapup is possibly stale', async () => {
    vi.mocked(api.fetchWrapupFreshness).mockResolvedValue({
      state: 'possiblyStale',
      changedPaths: ['rekall-core/src/a.rs', 'rekall-core/src/b.rs'],
      commits: ['abcdef1234567', '1234567abcdef']
    })

    const wrapper = await render()

    const tag = wrapper.get('[data-testid="wrapup-freshness"]')
    expect(tag.text()).toContain('Possibly stale')
    expect(tag.text()).toContain('2 named files changed in 2 commits')
    expect(wrapper.findAll('[data-testid="wrapup-freshness-paths"] li').map((item) => item.text())).toEqual([
      'rekall-core/src/a.rs',
      'rekall-core/src/b.rs'
    ])
    expect(wrapper.get('[data-testid="wrapup-freshness-commits"]').text()).toBe('abcdef1 1234567')
  })

  it.each(['fresh', 'unknown'] as const)('shows nothing when git says %s', async (state) => {
    vi.mocked(api.fetchWrapupFreshness).mockResolvedValue({ state })

    const wrapper = await render()

    expect(wrapper.find('[data-testid="wrapup-freshness"]').exists()).toBe(false)
  })
})
