import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import GuardProtectionSection from '@/features/settings/components/GuardProtectionSection.vue'
import type { GuardProtection } from '@/features/settings/model/guardProtection'

/** The section shows what the server read from Claude Code's settings and offers only what applies to it. */
const fetchGuardProtection = vi.fn()
const installGuardProtection = vi.fn()
const removeGuardProtection = vi.fn()

vi.mock('@/features/settings/api/guardProtection.api', () => ({
  fetchGuardProtection: () => fetchGuardProtection(),
  installGuardProtection: () => installGuardProtection(),
  removeGuardProtection: () => removeGuardProtection()
}))

function protection(overrides: Partial<GuardProtection> = {}): GuardProtection {
  return { state: 'NOT_INSTALLED', settingsFile: '/home/dev/.claude/settings.json', problem: null, ...overrides }
}

async function mountSection() {
  const wrapper = mount(GuardProtectionSection)
  await flushPromises()
  return wrapper
}

describe('GuardProtectionSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchGuardProtection.mockReset()
    installGuardProtection.mockReset()
    removeGuardProtection.mockReset()
  })

  it('offers to install when the hooks are not there and lists the rules they enforce', async () => {
    fetchGuardProtection.mockResolvedValue(protection())

    const wrapper = await mountSection()

    expect(wrapper.get('[data-testid="guard-status"]').text()).toBe('Not installed')
    expect(wrapper.find('[data-testid="guard-remove"]').exists()).toBe(false)
    expect(wrapper.findAll('[data-testid="guard-rules"] li')).toHaveLength(3)
  })

  it('shows the installed state after installing and then offers to remove', async () => {
    fetchGuardProtection.mockResolvedValue(protection())
    installGuardProtection.mockResolvedValue(protection({ state: 'INSTALLED' }))
    const wrapper = await mountSection()

    await wrapper.get('[data-testid="guard-install"]').trigger('click')
    await flushPromises()

    expect(installGuardProtection).toHaveBeenCalledTimes(1)
    expect(wrapper.get('[data-testid="guard-status"]').text()).toBe('Installed')
    expect(wrapper.find('[data-testid="guard-remove"]').exists()).toBe(true)
  })

  it('removes the hooks and goes back to not installed', async () => {
    fetchGuardProtection.mockResolvedValue(protection({ state: 'INSTALLED' }))
    removeGuardProtection.mockResolvedValue(protection())
    const wrapper = await mountSection()

    await wrapper.get('[data-testid="guard-remove"]').trigger('click')
    await flushPromises()

    expect(removeGuardProtection).toHaveBeenCalledTimes(1)
    expect(wrapper.get('[data-testid="guard-status"]').text()).toBe('Not installed')
  })

  it('shows why an invalid settings file was left alone and offers no write', async () => {
    fetchGuardProtection.mockResolvedValue(
      protection({ state: 'INVALID_SETTINGS', problem: 'settings.json is not usable (expected value at line 1).' })
    )

    const wrapper = await mountSection()

    expect(wrapper.get('[data-testid="guard-status"]').text()).toBe('Settings file not valid')
    expect(wrapper.get('[data-testid="guard-problem"]').text()).toContain('not usable')
    expect(wrapper.find('[data-testid="guard-install"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="guard-remove"]').exists()).toBe(false)
  })
})
