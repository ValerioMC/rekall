import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ClaudeCodeSection from '@/components/settings/ClaudeCodeSection.vue'
import type { ClaudeInstallation } from '@/model/claude'

/**
 * The Claude Code section, whose whole job is to tell the truth about a registration nobody can
 * see from inside the application, and to offer the one action that fixes it. Every state is
 * worth a test because each one is a different sentence and a different button.
 */
const fetchClaudeInstallation = vi.fn()
const installClaudeIntegration = vi.fn()

vi.mock('@/api/claude.api', () => ({
  fetchClaudeInstallation: () => fetchClaudeInstallation(),
  installClaudeIntegration: () => installClaudeIntegration()
}))

const ENDPOINT = 'http://localhost:47355/mcp'

function installation(overrides: Partial<ClaudeInstallation> = {}): ClaudeInstallation {
  return {
    status: 'NOT_CONNECTED',
    endpoint: ENDPOINT,
    registeredUrl: null,
    folderScoped: [],
    commandInstalled: false,
    cliPath: '/Users/someone/.local/bin/claude',
    manualCommand: `claude mcp add --scope user --transport http rekall ${ENDPOINT}`,
    ...overrides
  }
}

async function mountSection() {
  const wrapper = mount(ClaudeCodeSection)
  await flushPromises()
  return wrapper
}

describe('ClaudeCodeSection', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    fetchClaudeInstallation.mockReset()
    installClaudeIntegration.mockReset()
    fetchClaudeInstallation.mockResolvedValue(installation())
  })

  it('offers to connect when Claude Code knows nothing about this instance', async () => {
    const wrapper = await mountSection()

    expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('Not connected')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Connect')
    expect(wrapper.text()).toContain(ENDPOINT)
  })

  it('registers in one click and then says the running session will not see it', async () => {
    installClaudeIntegration.mockResolvedValue(
      installation({ status: 'CONNECTED', registeredUrl: ENDPOINT, commandInstalled: true })
    )
    const wrapper = await mountSection()

    await wrapper.get('[data-testid="claude-install"]').trigger('click')
    await flushPromises()

    expect(installClaudeIntegration).toHaveBeenCalledOnce()
    expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('Connected')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Reinstall')
    expect(wrapper.get('[data-testid="claude-restart-hint"]').text()).toContain('needs restarting')
  })

  it('names the address a stale registration points at, rather than only calling it wrong', async () => {
    fetchClaudeInstallation.mockResolvedValue(
      installation({ status: 'OUTDATED', registeredUrl: 'http://localhost:8080/mcp', commandInstalled: true })
    )
    const wrapper = await mountSection()

    expect(wrapper.get('[data-testid="claude-status"]').text()).toBe('Out of date')
    expect(wrapper.text()).toContain('http://localhost:8080/mcp')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Repair')
  })

  it('with no CLI to run, hands over the command instead of a button that cannot work', async () => {
    fetchClaudeInstallation.mockResolvedValue(installation({ status: 'CLI_MISSING', cliPath: null }))
    const wrapper = await mountSection()

    expect(wrapper.find('[data-testid="claude-install"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="claude-copy"]').exists()).toBe(true)
    expect(wrapper.text()).toContain(`claude mcp add --scope user --transport http rekall ${ENDPOINT}`)
  })

  it('says a folder keeping its own setup is what has to be repaired, without naming scopes', async () => {
    fetchClaudeInstallation.mockResolvedValue(
      installation({
        status: 'OUTDATED',
        registeredUrl: ENDPOINT,
        commandInstalled: true,
        folderScoped: ['/Users/someone/Projects/rekall']
      })
    )
    const wrapper = await mountSection()

    expect(wrapper.text()).toContain('One folder keeps a setup of its own')
    expect(wrapper.get('[data-testid="claude-install"]').text()).toBe('Repair')
  })

  it('leaves the section usable when the configuration cannot be read', async () => {
    fetchClaudeInstallation.mockRejectedValue(new Error('nope'))
    const wrapper = await mountSection()

    expect(wrapper.text()).toContain("Couldn't read the Claude Code configuration")
    expect(wrapper.find('[data-testid="claude-install"]').exists()).toBe(false)
  })
})
