import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import GuardProtectionSection from '@/features/settings/components/GuardProtectionSection.vue'
import type { GuardProtection } from '@/features/settings/model/guardProtection'
import type { SessionProvider } from '@/features/claude/model/session-provider'

const fetchGuardProtection = vi.fn()
const installGuardProtection = vi.fn()
const removeGuardProtection = vi.fn()

vi.mock('@/features/settings/api/guardProtection.api', () => ({
  fetchGuardProtection: (provider: SessionProvider) => fetchGuardProtection(provider),
  installGuardProtection: (provider: SessionProvider) => installGuardProtection(provider),
  removeGuardProtection: (provider: SessionProvider) => removeGuardProtection(provider)
}))

function protection(overrides: Partial<GuardProtection> = {}): GuardProtection {
  return {
    state: 'NOT_INSTALLED',
    settingsFile: '/home/dev/settings.json',
    configured: false,
    problem: null,
    ...overrides
  }
}

async function mountSection() {
  const wrapper = mount(GuardProtectionSection)
  await flushPromises()
  return {
    wrapper,
    claude: wrapper.get('[data-testid="guard-provider-CLAUDE"]'),
    codex: wrapper.get('[data-testid="guard-provider-CODEX"]')
  }
}

describe('GuardProtectionSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchGuardProtection.mockReset().mockResolvedValue(protection())
    installGuardProtection.mockReset()
    removeGuardProtection.mockReset()
  })

  it('loads both providers and lists the shared rules', async () => {
    const { wrapper, claude, codex } = await mountSection()

    expect(fetchGuardProtection.mock.calls).toEqual([['CLAUDE'], ['CODEX']])
    expect(claude.get('[data-testid="guard-status"]').text()).toBe('Not installed')
    expect(codex.get('[data-testid="guard-action"]').text()).toBe('Install')
    expect(wrapper.findAll('[data-testid="guard-rules"] li')).toHaveLength(3)
  })

  it('installs only Codex, shows the trust instructions, and refreshes just that provider', async () => {
    installGuardProtection.mockResolvedValue(
      protection({ state: 'TRUST_REQUIRED', configured: true, problem: 'Trust the Rekall hooks in /hooks.' })
    )
    const { claude, codex } = await mountSection()

    await codex.get('[data-testid="guard-action"]').trigger('click')
    await flushPromises()

    expect(installGuardProtection.mock.calls).toEqual([['CODEX']])
    expect(claude.get('[data-testid="guard-status"]').text()).toBe('Not installed')
    expect(codex.get('[data-testid="guard-problem"]').text()).toContain('/hooks')
    expect(codex.get('[data-testid="guard-remove"]').text()).toBe('Remove')

    fetchGuardProtection.mockResolvedValue(protection({ state: 'INSTALLED', configured: true }))
    await codex.get('[data-testid="guard-action"]').trigger('click')
    await flushPromises()

    expect(fetchGuardProtection).toHaveBeenLastCalledWith('CODEX')
    expect(codex.get('[data-testid="guard-status"]').text()).toBe('Installed')
  })

  it('offers repair for stale Claude settings and removal once repaired', async () => {
    fetchGuardProtection.mockImplementation((provider: SessionProvider) =>
      Promise.resolve(
        provider === 'CLAUDE' ? protection({ state: 'NEEDS_REPAIR', configured: true }) : protection()
      )
    )
    installGuardProtection.mockResolvedValue(protection({ state: 'INSTALLED', configured: true }))
    removeGuardProtection.mockResolvedValue(protection())
    const { claude, codex } = await mountSection()

    expect(claude.get('[data-testid="guard-action"]').text()).toBe('Repair')
    await claude.get('[data-testid="guard-action"]').trigger('click')
    await flushPromises()
    await claude.get('[data-testid="guard-remove"]').trigger('click')
    await flushPromises()

    expect(installGuardProtection.mock.calls).toEqual([['CLAUDE']])
    expect(removeGuardProtection.mock.calls).toEqual([['CLAUDE']])
    expect(claude.get('[data-testid="guard-status"]').text()).toBe('Not installed')
    expect(codex.get('[data-testid="guard-status"]').text()).toBe('Not installed')
  })

  it.each([
    ['INVALID_SETTINGS', 'Invalid settings'],
    ['UNSUPPORTED', 'Unavailable'],
    ['DISABLED', 'Disabled in Codex']
  ] as const)('shows the %s reason and offers refresh', async (state, label) => {
    fetchGuardProtection.mockResolvedValue(protection({ state, problem: 'Configuration needs attention.' }))
    const { codex } = await mountSection()

    expect(codex.get('[data-testid="guard-status"]').text()).toBe(label)
    expect(codex.get('[data-testid="guard-problem"]').text()).toContain('needs attention')
    expect(codex.get('[data-testid="guard-action"]').text()).toBe('Refresh')
  })
})
